use crate::entity::sampler;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<sampler::Model>, DbErr> {
    sampler::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    module_path: String,
    config_path: Option<String>,
) -> Result<sampler::Model, DbErr> {
    let active = sampler::ActiveModel {
        name: Set(name),
        module_path: Set(module_path),
        config_path: Set(config_path),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn sampler_exists(db: &DatabaseConnection, sampler_id: i32) -> Result<bool, DbErr> {
    let count = sampler::Entity::find_by_id(sampler_id)
        .one(db)
        .await?
        .is_some() as i64;

    Ok(count > 0)
}

pub async fn get_by_id(
    db: &DatabaseConnection,
    sampler_id: i32,
) -> Result<Option<sampler::Model>, DbErr> {
    sampler::Entity::find_by_id(sampler_id).one(db).await
}
