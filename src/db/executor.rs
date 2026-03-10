use crate::entity::executor;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<executor::Model>, DbErr> {
    executor::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    job_id: i32,
    array_id: i32,
    node_list: String,
    hostname: String,
) -> Result<executor::Model, DbErr> {
    let active = executor::ActiveModel {
        slurm_job_id: Set(job_id),
        slurm_array_id: Set(array_id),
        slurm_node_list: Set(node_list),
        hostname: Set(hostname),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn executor_exists(db: &DatabaseConnection, executor_id: i32) -> Result<bool, DbErr> {
    let count = executor::Entity::find_by_id(executor_id)
        .one(db)
        .await?
        .is_some() as i64;
    Ok(count > 0)
}
