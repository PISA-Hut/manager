use crate::entity::scenario;
use sea_orm::*;

pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<scenario::Model>, DbErr> {
    scenario::Entity::find().all(db).await
}

pub async fn create(
    db: &DatabaseConnection,
    title: Option<String>,
    description: Option<String>,
    scenario_path: String,
    param_path: Option<String>,
) -> Result<scenario::Model, DbErr> {
    let active = scenario::ActiveModel {
        title: Set(title),
        description: Set(description),
        scenario_path: Set(scenario_path),
        param_path: Set(param_path),
        ..Default::default()
    };

    active.insert(db).await
}

pub async fn scenario_exists(db: &DatabaseConnection, scenario_id: i32) -> Result<bool, DbErr> {
    let count = scenario::Entity::find_by_id(scenario_id)
        .one(db)
        .await?
        .is_some() as i64;

    Ok(count > 0)
}

pub async fn get_by_id(
    db: &DatabaseConnection,
    scenario_id: i32,
) -> Result<Option<scenario::Model>, DbErr> {
    scenario::Entity::find_by_id(scenario_id).one(db).await
}
