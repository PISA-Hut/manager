//! HTTP handlers for AV / Simulator / Sampler config bytes.
//!
//! All three entities expose the same `(GET | PUT | DELETE)
//! /{kind}/{id}/config` shape. We dispatch through the
//! `db::ConfigBearing` trait so the routing is per-entity (so Swagger
//! still shows three discrete endpoint groups) but the handler bodies
//! all share three generic helpers.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::app_state::AppState;
use crate::db::ConfigBearing;
use crate::entity::{av, sampler, simulator};
use crate::http::AppError;
use crate::http::handlers::bytes::{build_blob_response, sha256_hex};

async fn get_config<E: ConfigBearing>(
    headers: HeaderMap,
    state: AppState,
    id: i32,
) -> Result<axum::response::Response, AppError> {
    let model = E::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("{} not found", E::kind())))?;
    let content = model
        .config_bytes()
        .ok_or_else(|| AppError::not_found("config not set"))?
        .to_vec();
    let sha = model
        .config_sha256()
        .map(str::to_owned)
        .unwrap_or_else(|| sha256_hex(&content));
    Ok(build_blob_response(&headers, content, &sha))
}

async fn put_config<E: ConfigBearing>(
    state: AppState,
    id: i32,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    E::set_config(&state.db, id, content, sha).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_config<E: ConfigBearing>(state: AppState, id: i32) -> Result<StatusCode, AppError> {
    E::clear_config(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Per-entity adapters — one line each. Spelled out separately so the
// router (and Swagger / OpenAPI) treats `/av/.../config`,
// `/simulator/.../config`, `/sampler/.../config` as distinct
// endpoints rather than smushing them through one path.

pub async fn get_av_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    get_config::<av::Model>(headers, state, av_id).await
}

pub async fn put_av_config(
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    put_config::<av::Model>(state, av_id, body).await
}

pub async fn delete_av_config(
    State(state): State<AppState>,
    Path(av_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    delete_config::<av::Model>(state, av_id).await
}

pub async fn get_simulator_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    get_config::<simulator::Model>(headers, state, sim_id).await
}

pub async fn put_simulator_config(
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    put_config::<simulator::Model>(state, sim_id, body).await
}

pub async fn delete_simulator_config(
    State(state): State<AppState>,
    Path(sim_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    delete_config::<simulator::Model>(state, sim_id).await
}

pub async fn get_sampler_config(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
) -> Result<axum::response::Response, AppError> {
    get_config::<sampler::Model>(headers, state, sampler_id).await
}

pub async fn put_sampler_config(
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    put_config::<sampler::Model>(state, sampler_id, body).await
}

pub async fn delete_sampler_config(
    State(state): State<AppState>,
    Path(sampler_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    delete_config::<sampler::Model>(state, sampler_id).await
}
