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
        task_status: Set(TaskStatus::Pending),
        retry_count: Set(0),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn claim_task_with_filters(
    db: &DatabaseConnection,
    executor_id: i32,
    map_id: Option<i32>,
    scenario_id: Option<i32>,
    av_id: Option<i32>,
    simulator_id: Option<i32>,
    sampler_id: Option<i32>,
) -> Result<Option<task::Model>, DbErr> {
    let result = db
        .transaction(|txn| {
            Box::pin(async move {
                let task = task::Entity::find()
                    .join(JoinType::InnerJoin, task::Relation::Plan.def())
                    .filter(task::Column::TaskStatus.eq(TaskStatus::Pending))
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

                let mut active: task::ActiveModel = task.clone().into();
                active.task_status = Set(TaskStatus::Running);
                let updated = active.update(txn).await?;

                let active_run = task_run::ActiveModel {
                    task_id: Set(updated.id),
                    executor_id: Set(executor_id),
                    attempt: Set(updated.retry_count + 1),
                    task_run_status: Set(TaskRunStatus::Running),
                    started_at: Set(Some(Utc::now().fixed_offset())),
                    ..Default::default()
                };
                active_run.insert(txn).await?;

                Ok(Some(updated))
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

                let mut active_task: task::ActiveModel = task_model.clone().into();
                active_task.task_status = Set(TaskStatus::Pending);
                active_task.retry_count = Set(task_model.retry_count + 1);
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
                    active_run.task_run_status = Set(TaskRunStatus::Failed);
                    active_run.finished_at = Set(Some(Utc::now().fixed_offset()));
                    active_run.error_message = Set(Some(reason));
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

pub async fn invalidate_task(
    db: &DatabaseConnection,
    task_id: i32,
    reason: String,
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

                let mut active_task: task::ActiveModel = task.into();
                active_task.task_status = Set(TaskStatus::Invalid);
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
