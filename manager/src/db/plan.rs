use crate::entity::plan;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<plan::Model>, DbErr> {
    plan::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    name: String,
    map_id: i32,
    scenario_id: i32,
) -> Result<plan::Model, DbErr> {
    let active = plan::ActiveModel {
        name: Set(name),
        map_id: Set(map_id),
        scenario_id: Set(scenario_id),
        ..Default::default()
    };

    active.insert(db).await
}
