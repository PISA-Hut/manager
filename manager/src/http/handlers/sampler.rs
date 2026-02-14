use axum::{Json, extract::State, http::StatusCode};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::sampler::{CreateSamplerRequest, SamplerResponse};

pub async fn list_samplers(
    State(state): State<AppState>,
) -> Result<Json<Vec<SamplerResponse>>, StatusCode> {
    let samplers = db::sampler::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        samplers.into_iter().map(SamplerResponse::from).collect(),
    ))
}

pub async fn create_sampler(
    State(state): State<AppState>,
    Json(payload): Json<CreateSamplerRequest>,
) -> Result<Json<SamplerResponse>, StatusCode> {
    let sampler_model = db::sampler::create(
        &state.db,
        payload.name,
        payload.module_path,
        payload.config_path,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SamplerResponse::from(sampler_model)))
}
