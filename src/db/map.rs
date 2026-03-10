use crate::entity::map;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<map::Model>, DbErr> {
    map::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    xodr_path: Option<String>,
    osm_path: Option<String>,
) -> Result<map::Model, DbErr> {
    let active = map::ActiveModel {
        name: Set(name),
        xodr_path: Set(xodr_path),
        osm_path: Set(osm_path),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn map_exists(db: &DatabaseConnection, map_id: i32) -> Result<bool, DbErr> {
    let count = map::Entity::find_by_id(map_id).one(db).await?.is_some() as i64;

    Ok(count > 0)
}

pub async fn get_by_id(db: &DatabaseConnection, map_id: i32) -> Result<Option<map::Model>, DbErr> {
    map::Entity::find_by_id(map_id).one(db).await
}
