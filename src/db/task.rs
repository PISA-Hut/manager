use crate::entity::plan;
use crate::entity::sea_orm_active_enums::TaskRunStatus;
use crate::entity::sea_orm_active_enums::TaskStatus;
use crate::entity::task;
use crate::entity::task_run;
use chrono::Utc;

use sea_orm::*;
use sea_orm_migration::prelude::{LockBehavior, LockType};

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<task::Model>, DbErr> {
    task::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    plan_id: i32,
    av_id: i32,
    sampler_id: i32,
    simulator_id: i32,
) -> Result<task::Model, DbErr> {
    let active = task::ActiveModel {
        plan_id: Set(plan_id),
        av_id: Set(av_id),
        sampler_id: Set(sampler_id),
        simulator_id: Set(simulator_id),
        task_status: Set(TaskStatus::Queued),
        attempt_count: Set(0),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn claim_task_with_filters(
    db: &DatabaseConnection,
    executor_id: i32,
    task_id: Option<i32>,
    map_id: Option<i32>,
    scenario_id: Option<i32>,
    av_id: Option<i32>,
    simulator_id: Option<i32>,
    sampler_id: Option<i32>,
) -> Result<Option<(task::Model, i32)>, DbErr> {
    let result = db
        .transaction(|txn| {
            Box::pin(async move {
                let task = task::Entity::find()
                    .join(JoinType::InnerJoin, task::Relation::Plan.def())
                    .filter(task::Column::TaskStatus.eq(TaskStatus::Queued))
                    .apply_if(task_id, |q, task_id| q.filter(task::Column::Id.eq(task_id)))
                    .apply_if(map_id, |q, map_id| q.filter(plan::Column::MapId.eq(map_id)))
                    .apply_if(scenario_id, |q, scenario_id| {
                        q.filter(plan::Column::ScenarioId.eq(scenario_id))
                    })
                    .apply_if(av_id, |q, av_id| q.filter(task::Column::AvId.eq(av_id)))
                    .apply_if(simulator_id, |q, simulator_id| {
                        q.filter(task::Column::SimulatorId.eq(simulator_id))
                    })
                    .apply_if(sampler_id, |q, sampler_id| {
                        q.filter(task::Column::SamplerId.eq(sampler_id))
                    })
                    .order_by_desc(task::Column::CreatedAt)
                    .limit(1)
                    .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
                    .one(txn)
                    .await?;

                let Some(task) = task else {
                    return Ok(None);
                };

                // Count existing runs to derive attempt_count and next attempt
                let run_count = task_run::Entity::find()
                    .filter(task_run::Column::TaskId.eq(task.id))
                    .count(txn)
                    .await? as i32;

                let mut active: task::ActiveModel = task.clone().into();
                active.task_status = Set(TaskStatus::Running);
                active.attempt_count = Set(run_count + 1);
                let updated = active.update(txn).await?;

                let next_attempt = run_count + 1;

                let active_run = task_run::ActiveModel {
                    task_id: Set(updated.id),
                    executor_id: Set(executor_id),
                    attempt: Set(next_attempt),
                    task_run_status: Set(TaskRunStatus::Running),
                    started_at: Set(Some(Utc::now().fixed_offset())),
                    ..Default::default()
                };
                let inserted_run = active_run.insert(txn).await?;

                Ok(Some((updated, inserted_run.id)))
            })
        })
        .await;

    match result {
        Ok(v) => Ok(v),
        Err(TransactionError::Connection(e)) => Err(e),
        Err(TransactionError::Transaction(e)) => Err(e),
    }
}

pub async fn complete_task(
    db: &DatabaseConnection,
    task_id: i32,
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
                let Some(task) = task else {
                    return Ok(None);
                };

                // Ignore if task is no longer running (e.g. stopped from web UI)
                if task.task_status != TaskStatus::Running {
                    return Ok(Some(task));
                }

                let mut active_task: task::ActiveModel = task.into();
                active_task.task_status = Set(TaskStatus::Completed);
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
                    active_run.task_run_status = Set(TaskRunStatus::Completed);
                    active_run.finished_at = Set(Some(Utc::now().fixed_offset()));
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

pub async fn fail_task(
    db: &DatabaseConnection,
    task_id: i32,
    reason: String,
    log: Option<String>,
    concrete_scenarios_executed: i32,
    useless_streak_limit: usize,
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

                // Ignore if task is no longer running (e.g. stopped from web UI)
                if task_model.task_status != TaskStatus::Running {
                    return Ok(Some(task_model));
                }

                // Permanent failure only when this failing run and the
                // previous `useless_streak_limit - 1` runs all finished with
                // zero concrete scenarios executed. Any run — regardless of
                // its terminal status — that managed to finish at least one
                // concrete scenario resets the streak. Error messages are
                // no longer compared.
                let prior_streak_target = useless_streak_limit.saturating_sub(1);
                let recent_runs = task_run::Entity::find()
                    .filter(task_run::Column::TaskId.eq(task_id))
                    .filter(task_run::Column::TaskRunStatus.ne(TaskRunStatus::Running))
                    .order_by_desc(task_run::Column::Attempt)
                    .limit(prior_streak_target as u64)
                    .all(txn)
                    .await?;

                let permanent_fail = concrete_scenarios_executed == 0
                    && recent_runs.len() == prior_streak_target
                    && recent_runs
                        .iter()
                        .all(|r| r.concrete_scenarios_executed == 0);

                let run_count = task_run::Entity::find()
                    .filter(task_run::Column::TaskId.eq(task_id))
                    .count(txn)
                    .await? as i32;

                // Permanent fail lands on Invalid: by this point the task
                // has produced `useless_streak_limit` consecutive runs that
                // finished zero concrete scenarios — strong evidence the
                // config can't be executed. A single useful run earlier
                // would have reset the streak.
                let new_status = if permanent_fail {
                    TaskStatus::Invalid
                } else {
                    TaskStatus::Queued
                };

                let mut active_task: task::ActiveModel = task_model.clone().into();
                active_task.task_status = Set(new_status);
                active_task.attempt_count = Set(run_count);
                let updated_task = active_task.update(txn).await?;

                // Update the current running task_run
                if let Some(run) = task_run::Entity::find()
                    .filter(task_run::Column::TaskId.eq(task_id))
                    .filter(task_run::Column::TaskRunStatus.eq(TaskRunStatus::Running))
                    .order_by_desc(task_run::Column::Attempt)
                    .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
                    .one(txn)
                    .await?
                {
                    let mut active_run: task_run::ActiveModel = run.into();
                    active_run.task_run_status = Set(TaskRunStatus::Failed);
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
