use crate::entity::worker;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<worker::Model>, DbErr> {
    worker::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    job_id: i32,
    node_list: String,
    hostname: String,
) -> Result<worker::Model, DbErr> {
    let active = worker::ActiveModel {
        job_id: Set(job_id),
        node_list: Set(node_list),
        hostname: Set(hostname),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn worker_exists(db: &DatabaseConnection, worker_id: i32) -> Result<bool, DbErr> {
    let count = worker::Entity::find_by_id(worker_id)
        .one(db)
        .await?
        .is_some() as i64;

    Ok(count > 0)
}
