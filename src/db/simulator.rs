use crate::entity::simulator;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<simulator::Model>, DbErr> {
    simulator::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    image_path: JsonValue,
    nv_runtime: bool,
    carla_runtime: bool,
    ros_runtime: bool,
) -> Result<simulator::Model, DbErr> {
    let active = simulator::ActiveModel {
        name: Set(name),
        image_path: Set(image_path),
        nv_runtime: Set(nv_runtime),
        carla_runtime: Set(carla_runtime),
        ros_runtime: Set(ros_runtime),
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

pub async fn set_config(
    db: &DatabaseConnection,
    simulator_id: i32,
    content: Vec<u8>,
    content_sha256: String,
) -> Result<simulator::Model, DbErr> {
    let existing = simulator::Entity::find_by_id(simulator_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "simulator {} not found",
            simulator_id
        )))?;
    let mut am: simulator::ActiveModel = existing.into();
    am.config = Set(Some(content));
    am.config_sha256 = Set(Some(content_sha256));
    am.update(db).await
}

pub async fn clear_config(
    db: &DatabaseConnection,
    simulator_id: i32,
) -> Result<simulator::Model, DbErr> {
    let existing = simulator::Entity::find_by_id(simulator_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "simulator {} not found",
            simulator_id
        )))?;
    let mut am: simulator::ActiveModel = existing.into();
    am.config = Set(None);
    am.config_sha256 = Set(None);
    am.update(db).await
}
