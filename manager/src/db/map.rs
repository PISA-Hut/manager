use crate::entity::map;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<map::Model>, DbErr> {
    map::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    xodr: bool,
    osm: bool,
    path: String,
) -> Result<map::Model, DbErr> {
    let active = map::ActiveModel {
        name: Set(name),
        xodr: Set(xodr),
        osm: Set(osm),
        path: Set(path),
        ..Default::default()
    };

    active.insert(db).await
}
