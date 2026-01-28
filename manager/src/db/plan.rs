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

pub async fn plan_exists(db: &DatabaseConnection, plan_id: i32) -> Result<bool, DbErr> {
    let count = plan::Entity::find_by_id(plan_id).one(db).await?.is_some() as i64;

    Ok(count > 0)
}

pub async fn get_by_id(
    db: &DatabaseConnection,
    plan_id: i32,
) -> Result<Option<plan::Model>, DbErr> {
    plan::Entity::find_by_id(plan_id).one(db).await
}
