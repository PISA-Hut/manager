use axum::{Json, extract::State, http::StatusCode};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::av::{AvResponse, CreateAvRequest};

pub async fn list_avs(State(state): State<AppState>) -> Result<Json<Vec<AvResponse>>, StatusCode> {
    let avs = db::av::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(avs.into_iter().map(AvResponse::from).collect()))
}

pub async fn create_av(
    State(state): State<AppState>,
    Json(payload): Json<CreateAvRequest>,
) -> Result<Json<AvResponse>, StatusCode> {
    let av_model = db::av::create(
        &state.db,
        payload.name,
        payload.image_path,
        payload.config_path,
        payload.nv_runtime,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AvResponse::from(av_model)))
}
