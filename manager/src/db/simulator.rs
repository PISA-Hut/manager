use crate::entity::simulator;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<simulator::Model>, DbErr> {
    simulator::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    image_path: String,
    config_path: String,
    nv_runtime: bool,
    extra_ports: Option<serde_json::Value>,
) -> Result<simulator::Model, DbErr> {
    let active = simulator::ActiveModel {
        name: Set(name),
        image_path: Set(image_path),
        config_path: Set(config_path),
        nv_runtime: Set(nv_runtime),
        extra_ports: Set(extra_ports),
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

pub async fn get_by_id(
    db: &DatabaseConnection,
    simulator_id: i32,
) -> Result<Option<simulator::Model>, DbErr> {
    simulator::Entity::find_by_id(simulator_id).one(db).await
}
