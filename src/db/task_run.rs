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
pub async fn append_log(
    db: &DatabaseConnection,
    run_id: i32,
    chunk: &str,
) -> Result<i64, DbErr> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"UPDATE task_run
               SET log = COALESCE(log, '') || $1,
                   last_heartbeat_at = now()
               WHERE id = $2
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

    let mut reaped = Vec::with_capacity(stale.len());
    for run in stale {
        let task_id = run.task_id;
        let run_id = run.id;

        let mut active_run: task_run::ActiveModel = run.into();
        active_run.task_run_status = Set(TaskRunStatus::Aborted);
        active_run.finished_at = Set(Some(Utc::now().fixed_offset()));
        active_run.error_message = Set(Some(format!(
            "no heartbeat from executor for {stale_after_secs}s (reaped)"
        )));
        active_run.update(db).await?;

        // Only pull the parent task back to `queued` if it's still
        // marked `running`; any other transition (completed/exhausted/
        // invalid/aborted) has already moved the task out of the running
        // lane and we don't want to undo it.
        if let Some(parent) = task::Entity::find_by_id(task_id).one(db).await?
            && parent.task_status == TaskStatus::Running
        {
            let mut active: task::ActiveModel = parent.into();
            active.task_status = Set(TaskStatus::Queued);
            active.update(db).await?;
        }

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
