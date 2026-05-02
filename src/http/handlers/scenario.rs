use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::scenario::{CreateScenarioRequest, ScenarioResponse};

pub async fn list_scenarios(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScenarioResponse>>, AppError> {
    let scenarios = db::scenario::find_all(&state.db).await?;
    Ok(Json(
        scenarios.into_iter().map(ScenarioResponse::from).collect(),
    ))
}

pub async fn create_scenario(
    State(state): State<AppState>,
    Json(payload): Json<CreateScenarioRequest>,
) -> Result<Json<ScenarioResponse>, AppError> {
    let scenario = db::scenario::create(&state.db, payload.format, payload.title).await?;
    Ok(Json(ScenarioResponse::from(scenario)))
}
