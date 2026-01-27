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
) -> Result<task::Model, DbErr> {
    let active = task::ActiveModel {
        plan_id: Set(plan_id),
        av_id: Set(av_id),
        status: Set(Some("created".to_string())),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn claim_one_unassigned(
    db: &DatabaseConnection,
    worker_id: i32,
) -> Result<Option<task::Model>, DbErr> {
    let result = db
        .transaction(|txn| {
            Box::pin(async move {
                let task = task::Entity::find()
                    .filter(task::Column::WorkerId.is_null())
                    .order_by_asc(task::Column::Id)
                    .one(txn)
                    .await?;

                let Some(task) = task else {
                    return Ok(None);
                };

                let mut active: task::ActiveModel = task.into();
                active.worker_id = Set(Some(worker_id));
                active.status = Set(Some("running".to_string()));
                active.executed_at = Set(Some(Utc::now()));

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
) -> Result<Option<task::Model>, DbErr> {
    let task = task::Entity::find_by_id(task_id).one(db).await?;

    let Some(task) = task else {
        return Ok(None);
    };

    let mut active: task::ActiveModel = task.into();
    active.status = Set(Some("completed".to_string()));
    active.finished_at = Set(Some(Utc::now()));

    let updated = active.update(db).await?;
    Ok(Some(updated))
}
