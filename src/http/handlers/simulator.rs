use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::simulator::{CreateSimulatorRequest, SimulatorResponse};

pub async fn list_simulators(
    State(state): State<AppState>,
) -> Result<Json<Vec<SimulatorResponse>>, AppError> {
    let simulators = db::simulator::find_all(&state.db).await?;
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
) -> Result<Json<SimulatorResponse>, AppError> {
    let simulator_model = db::simulator::create(
        &state.db,
        payload.name,
        payload.image_path,
        payload.nv_runtime,
        payload.carla_runtime,
        payload.ros_runtime,
    )
    .await?;
    Ok(Json(SimulatorResponse::from(simulator_model)))
}
