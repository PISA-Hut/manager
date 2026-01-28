use crate::entity::simulator;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<simulator::Model>, DbErr> {
    simulator::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    module_path: String,
) -> Result<simulator::Model, DbErr> {
    let active = simulator::ActiveModel {
        name: Set(name),
        module_path: Set(module_path),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn simulator_exists(db: &DatabaseConnection, simulator_id: i32) -> Result<bool, DbErr> {
    let count = simulator::Entity::find_by_id(simulator_id)
        .one(db)
        .await?
        .is_some() as i64;

    Ok(count > 0)
}
