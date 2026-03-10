use axum::{Json, extract::State, http::StatusCode};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::scenario::{CreateScenarioRequest, ScenarioResponse};

pub async fn list_scenarios(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScenarioResponse>>, StatusCode> {
    let scenarios = db::scenario::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        scenarios.into_iter().map(ScenarioResponse::from).collect(),
    ))
}

pub async fn create_scenario(
    State(state): State<AppState>,
    Json(payload): Json<CreateScenarioRequest>,
) -> Result<Json<ScenarioResponse>, StatusCode> {
    let scenario = db::scenario::create(
        &state.db,
        payload.title,
        payload.scenario_path,
        payload.goal_config,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ScenarioResponse::from(scenario)))
}
