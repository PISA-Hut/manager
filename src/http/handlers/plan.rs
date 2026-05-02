use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::plan::{CreatePlanRequest, PlanResponse};

pub async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<PlanResponse>>, AppError> {
    let plans = db::plan::find_all(&state.db).await?;
    Ok(Json(plans.into_iter().map(PlanResponse::from).collect()))
}

pub async fn create_plan(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlanRequest>,
) -> Result<Json<PlanResponse>, AppError> {
    if !db::map::map_exists(&state.db, payload.map_id).await? {
        return Err(AppError::bad_request(format!(
            "map {} does not exist",
            payload.map_id
        )));
    }
    if !db::scenario::scenario_exists(&state.db, payload.scenario_id).await? {
        return Err(AppError::bad_request(format!(
            "scenario {} does not exist",
            payload.scenario_id
        )));
    }
    let plan =
        db::plan::create(&state.db, payload.name, payload.map_id, payload.scenario_id).await?;
    Ok(Json(PlanResponse::from(plan)))
}
