use crate::entity::sampler;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<sampler::Model>, DbErr> {
    sampler::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    module_path: String,
) -> Result<sampler::Model, DbErr> {
    let active = sampler::ActiveModel {
        name: Set(name),
        module_path: Set(module_path),
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

pub async fn set_config(
    db: &DatabaseConnection,
    sampler_id: i32,
    content: Vec<u8>,
    content_sha256: String,
) -> Result<sampler::Model, DbErr> {
    let existing = sampler::Entity::find_by_id(sampler_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "sampler {} not found",
            sampler_id
        )))?;
    let mut am: sampler::ActiveModel = existing.into();
    am.config = Set(Some(content));
    am.config_sha256 = Set(Some(content_sha256));
    am.update(db).await
}

pub async fn clear_config(
    db: &DatabaseConnection,
    sampler_id: i32,
) -> Result<sampler::Model, DbErr> {
    let existing = sampler::Entity::find_by_id(sampler_id)
        .one(db)
        .await?
        .ok_or(DbErr::RecordNotFound(format!(
            "sampler {} not found",
            sampler_id
        )))?;
    let mut am: sampler::ActiveModel = existing.into();
    am.config = Set(None);
    am.config_sha256 = Set(None);
    am.update(db).await
}
