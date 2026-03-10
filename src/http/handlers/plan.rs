use axum::{Json, extract::State, http::StatusCode};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::plan::{CreatePlanRequest, PlanResponse};

pub async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<PlanResponse>>, StatusCode> {
    let plans = db::plan::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(plans.into_iter().map(PlanResponse::from).collect()))
}

pub async fn create_plan(
    State(state): State<AppState>,
    Json(payload): Json<CreatePlanRequest>,
) -> Result<Json<PlanResponse>, StatusCode> {
    if !db::map::map_exists(&state.db, payload.map_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !db::scenario::scenario_exists(&state.db, payload.scenario_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let plan = db::plan::create(&state.db, payload.name, payload.map_id, payload.scenario_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PlanResponse::from(plan)))
}
