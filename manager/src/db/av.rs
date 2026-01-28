use crate::entity::av;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<av::Model>, DbErr> {
    av::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    config_path: String,
) -> Result<av::Model, DbErr> {
    let active = av::ActiveModel {
        name: Set(name),
        config_path: Set(config_path),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn av_exists(db: &DatabaseConnection, av_id: i32) -> Result<bool, DbErr> {
    let count = av::Entity::find_by_id(av_id).one(db).await?.is_some() as i64;

    Ok(count > 0)
}

pub async fn get_by_id(db: &DatabaseConnection, av_id: i32) -> Result<Option<av::Model>, DbErr> {
    av::Entity::find_by_id(av_id).one(db).await
}
