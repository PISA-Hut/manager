use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::sampler::{CreateSamplerRequest, SamplerResponse};

pub async fn list_samplers(
    State(state): State<AppState>,
) -> Result<Json<Vec<SamplerResponse>>, AppError> {
    let samplers = db::sampler::find_all(&state.db).await?;
    Ok(Json(
        samplers.into_iter().map(SamplerResponse::from).collect(),
    ))
}

pub async fn create_sampler(
    State(state): State<AppState>,
    Json(payload): Json<CreateSamplerRequest>,
) -> Result<Json<SamplerResponse>, AppError> {
    let sampler_model = db::sampler::create(&state.db, payload.name, payload.module_path).await?;
    Ok(Json(SamplerResponse::from(sampler_model)))
}
