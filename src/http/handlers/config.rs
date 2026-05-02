//! HTTP handlers for AV / Simulator / Sampler config bytes.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::handlers::bytes::{build_blob_response, sha256_hex};

pub async fn get_av_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    let av = db::av::get_by_id(&state.db, av_id)
        .await?
        .ok_or_else(|| AppError::not_found("av not found"))?;
    let content = av
        .config
        .ok_or_else(|| AppError::not_found("config not set"))?;
    let sha = av.config_sha256.unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

pub async fn put_av_config(
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    db::av::set_config(&state.db, av_id, content, sha).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_av_config(
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    db::av::clear_config(&state.db, av_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_simulator_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    let sim = db::simulator::get_by_id(&state.db, sim_id)
        .await?
        .ok_or_else(|| AppError::not_found("simulator not found"))?;
    let content = sim
        .config
        .ok_or_else(|| AppError::not_found("config not set"))?;
    let sha = sim.config_sha256.unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

pub async fn put_simulator_config(
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    db::simulator::set_config(&state.db, sim_id, content, sha).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_simulator_config(
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    db::simulator::clear_config(&state.db, sim_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_sampler_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    let sampler = db::sampler::get_by_id(&state.db, sampler_id)
        .await?
        .ok_or_else(|| AppError::not_found("sampler not found"))?;
    let content = sampler
        .config
        .ok_or_else(|| AppError::not_found("config not set"))?;
    let sha = sampler
        .config_sha256
        .unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

pub async fn put_sampler_config(
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    db::sampler::set_config(&state.db, sampler_id, content, sha).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_sampler_config(
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    db::sampler::clear_config(&state.db, sampler_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
