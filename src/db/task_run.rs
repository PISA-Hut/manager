use crate::entity::sea_orm_active_enums::{TaskRunStatus, TaskStatus};
use crate::entity::{task, task_run};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DbBackend, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionError, TransactionTrait,
};
use sea_orm_migration::prelude::{LockBehavior, LockType};

/// Append a chunk to `task_run.log` without rewriting the entire column,
/// and bump `last_heartbeat_at` so the reaper treats this run as alive.
/// Returns the byte length of `task_run.log` *after* the append, which
/// is the inclusive end-offset of the newly written chunk. Callers
/// piggy-back this on the SSE envelope so a Log Drawer subscriber that
/// races the snapshot fetch can dedupe overlapping chunks instead of
/// dropping them and silently truncating its view.
///
/// Gated on `task_run_status = 'running'` so a chunk that arrives after
/// the run has been finalised (Stop, abort, reaper) cannot revive its
/// heartbeat or grow the log. The pre-flight status check in the HTTP
/// handler is racy with concurrent finalisation; this is the lock.
/// Surfaces as `RecordNotFound` when the row exists but is not running,
/// which the handler maps to 410 Gone just like a missing row.
pub async fn append_log(db: &DatabaseConnection, run_id: i32, chunk: &str) -> Result<i64, DbErr> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE task_run
               SET log = COALESCE(log, '') || $1,
                   last_heartbeat_at = now()
               WHERE id = $2
                 AND task_run_status = 'running'
               RETURNING octet_length(log)::bigint AS end_offset"#,
            [chunk.into(), run_id.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("task_run {run_id}")))?;
    let end_offset: i64 = row.try_get_by("end_offset")?;
    Ok(end_offset)
}

/// Mark task_runs that are still in `running` but haven't sent a log chunk
/// or heartbeat in `stale_after` seconds as `aborted`, and flip their
/// parent task back to `queued` so the scheduler can retry. Returns the
/// list of rows that were reaped so callers can log them.
///
/// Uses started_at as a fallback when a run has never sent a heartbeat
/// (brand-new run from an executor on an old build).
///
/// The status flip is an atomic conditional UPDATE re-checking the
/// running+stale predicate so a run that legitimately transitions to
/// completed/failed/aborted between the SELECT and the UPDATE is not
/// clobbered. Same idea for the parent task: only requeue if it's
/// still `running`. Rows that lost the race are simply skipped.
pub async fn reap_stale_runs(
    db: &DatabaseConnection,
    stale_after_secs: i64,
) -> Result<Vec<i32>, DbErr> {
    let cutoff = Utc::now().fixed_offset() - Duration::seconds(stale_after_secs);

    let stale = task_run::Entity::find()
        .filter(task_run::Column::TaskRunStatus.eq(TaskRunStatus::Running))
        .filter(
            task_run::Column::LastHeartbeatAt
                .lt(cutoff)
                .or(task_run::Column::LastHeartbeatAt
                    .is_null()
                    .and(task_run::Column::StartedAt.lt(cutoff))),
        )
        .all(db)
        .await?;

    if stale.is_empty() {
        return Ok(Vec::new());
    }

    let error_message = format!("no heartbeat from executor for {stale_after_secs}s (reaped)");

    let mut reaped = Vec::with_capacity(stale.len());
    for run in stale {
        let task_id = run.task_id;
        let run_id = run.id;

        // Atomic CAS: only flip to `aborted` if the row is still
        // `running` AND still stale by the same predicate. If anyone
        // (including the executor's own finish path) raced us to
        // finalise it, RETURNING comes back empty and we move on.
        let reaped_row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"UPDATE task_run
                   SET task_run_status = 'aborted',
                       finished_at = now(),
                       error_message = $1
                   WHERE id = $2
                     AND task_run_status = 'running'
                     AND (
                         last_heartbeat_at < $3
                         OR (last_heartbeat_at IS NULL AND started_at < $3)
                     )
                   RETURNING id"#,
                [error_message.clone().into(), run_id.into(), cutoff.into()],
            ))
            .await?;

        if reaped_row.is_none() {
            continue;
        }

        // Only pull the parent task back to `queued` if it's still
        // marked `running`; any other transition (completed/invalid/
        // aborted) has already moved the task out of the running lane
        // and we don't want to undo it.
        db.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE task
               SET task_status = 'queued'
               WHERE id = $1
                 AND task_status = 'running'"#,
            [task_id.into()],
        ))
        .await?;

        reaped.push(run_id);
    }

    Ok(reaped)
}

/// Mark a running task_run as aborted (by SLURM scancel or user stop).
/// Parent task goes to `aborted` — the run is done, and the user must
/// deliberately Run again to leave the state.
pub async fn abort_task(
    db: &DatabaseConnection,
    task_id: i32,
    reason: String,
    log: Option<String>,
    concrete_scenarios_executed: i32,
) -> Result<Option<task::Model>, DbErr> {
    let result = db
        .transaction(|txn| {
            Box::pin(async move {
                let task = task::Entity::find_by_id(task_id)
                    .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
                    .one(txn)
                    .await?;
                let Some(task_model) = task else {
                    return Ok(None);
                };
                // Idempotent: if the task already left `running` (stopped
                // from the UI, etc.), don't rewrite it.
                if task_model.task_status != TaskStatus::Running {
                    return Ok(Some(task_model));
                }

                let mut active_task: task::ActiveModel = task_model.into();
                active_task.task_status = Set(TaskStatus::Aborted);
                let updated_task = active_task.update(txn).await?;

                if let Some(run) = task_run::Entity::find()
                    .filter(task_run::Column::TaskId.eq(task_id))
                    .filter(task_run::Column::TaskRunStatus.eq(TaskRunStatus::Running))
                    .order_by_desc(task_run::Column::Attempt)
                    .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
                    .one(txn)
                    .await?
                {
                    let mut active_run: task_run::ActiveModel = run.into();
                    active_run.task_run_status = Set(TaskRunStatus::Aborted);
                    active_run.finished_at = Set(Some(Utc::now().fixed_offset()));
                    active_run.error_message = Set(Some(reason));
                    active_run.concrete_scenarios_executed = Set(concrete_scenarios_executed);
                    if log.is_some() {
                        active_run.log = Set(log);
                    }
                    active_run.update(txn).await?;
                }

                Ok(Some(updated_task))
            })
        })
        .await;

    match result {
        Ok(v) => Ok(v),
        Err(TransactionError::Connection(e)) => Err(e),
        Err(TransactionError::Transaction(e)) => Err(e),
    }
}
