use crate::entity::av;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<av::Model>, DbErr> {
    av::Entity::find().all(db).await
}

pub async fn create(db: &DatabaseConnection, name: String) -> Result<av::Model, DbErr> {
    let active = av::ActiveModel {
        name: Set(name),
        ..Default::default()
    };

    active.insert(db).await
}
