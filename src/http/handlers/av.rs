use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::av::{AvResponse, CreateAvRequest};

pub async fn list_avs(State(state): State<AppState>) -> Result<Json<Vec<AvResponse>>, AppError> {
    let avs = db::av::find_all(&state.db).await?;
    Ok(Json(avs.into_iter().map(AvResponse::from).collect()))
}

pub async fn create_av(
    State(state): State<AppState>,
    Json(payload): Json<CreateAvRequest>,
) -> Result<Json<AvResponse>, AppError> {
    let av_model = db::av::create(
        &state.db,
        payload.name,
        payload.image_path,
        payload.nv_runtime,
        payload.carla_runtime,
        payload.ros_runtime,
    )
    .await?;
    Ok(Json(AvResponse::from(av_model)))
}
