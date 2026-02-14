use axum::{Json, extract::State, http::StatusCode};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::simulator::{CreateSimulatorRequest, SimulatorResponse};

pub async fn list_simulators(
    State(state): State<AppState>,
) -> Result<Json<Vec<SimulatorResponse>>, StatusCode> {
    let simulators = db::simulator::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        simulators
            .into_iter()
            .map(SimulatorResponse::from)
            .collect(),
    ))
}

pub async fn create_simulator(
    State(state): State<AppState>,
    Json(payload): Json<CreateSimulatorRequest>,
) -> Result<Json<SimulatorResponse>, StatusCode> {
    let simulator_model = db::simulator::create(
        &state.db,
        payload.name,
        payload.image_path,
        payload.config_path,
        payload.nv_runtime,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SimulatorResponse::from(simulator_model)))
}
