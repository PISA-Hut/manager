use crate::entity::sea_orm_active_enums::TaskStatus;
use crate::entity::task;
use chrono::Utc;
use sea_orm::*;

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
        status: Set(TaskStatus::Pending),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn claim_task_with_filters(
    db: &DatabaseConnection,
    worker_id: i32,
    plan_id: Option<i32>,
    av_id: Option<i32>,
    simulator_id: Option<i32>,
    sampler_id: Option<i32>,
) -> Result<Option<task::Model>, DbErr> {
    let result = db
        .transaction(|txn| {
            Box::pin(async move {
                let task = task::Entity::find()
                    .filter(task::Column::WorkerId.is_null())
                    .filter(task::Column::Status.eq(TaskStatus::Pending))
                    .filter(
                        task::Column::PlanId
                            .is_null()
                            .or(task::Column::PlanId.eq(plan_id.unwrap_or(0))),
                    )
                    .filter(
                        task::Column::AvId
                            .is_null()
                            .or(task::Column::AvId.eq(av_id.unwrap_or(0))),
                    )
                    .filter(
                        task::Column::SimulatorId
                            .is_null()
                            .or(task::Column::SimulatorId.eq(simulator_id.unwrap_or(0))),
                    )
                    .filter(
                        task::Column::SamplerId
                            .is_null()
                            .or(task::Column::SamplerId.eq(sampler_id.unwrap_or(0))),
                    )
                    .order_by_asc(task::Column::Id)
                    .one(txn)
                    .await?;

                let Some(task) = task else {
                    return Ok(None);
                };

                let mut active: task::ActiveModel = task.into();
                active.worker_id = Set(Some(worker_id));
                active.status = Set(TaskStatus::InProgress);
                active.executed_at = Set(Some(Utc::now().fixed_offset()));

                let updated = active.update(txn).await?;
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
    status: TaskStatus,
) -> Result<Option<task::Model>, DbErr> {
    let task = task::Entity::find_by_id(task_id).one(db).await?;

    let Some(task) = task else {
        return Ok(None);
    };

    let mut active: task::ActiveModel = task.into();
    active.status = Set(status);
    active.finished_at = Set(Some(Utc::now().fixed_offset()));

    let updated = active.update(db).await?;
    Ok(Some(updated))
}
