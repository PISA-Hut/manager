//! HTTP handlers for `map_file` rows. Mirror of `scenario_file.rs`;
//! the actual logic lives in the generic `file_ops` helpers backed
//! by the `db::FileStore` trait.

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::app_state::AppState;
use crate::entity::map_file;
use crate::http::AppError;
use crate::http::dto::map_file::MapFileMetaResponse;
use crate::http::handlers::file_ops;

pub async fn list_files(
    State(state): State<AppState>,
    Path(map_id): Path<i32>,
) -> Result<Json<Vec<MapFileMetaResponse>>, AppError> {
    let metas = file_ops::list::<map_file::Model, _>(&state.db, map_id, Into::into).await?;
    Ok(Json(metas))
}

pub async fn get_file(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((map_id, relative_path)): Path<(i32, String)>,
) -> Result<axum::response::Response, AppError> {
    file_ops::get::<map_file::Model>(&headers, &state.db, map_id, &relative_path).await
}

pub async fn put_file(
    State(state): State<AppState>,
    Path((map_id, relative_path)): Path<(i32, String)>,
    body: Bytes,
) -> Result<Json<MapFileMetaResponse>, AppError> {
    let model = file_ops::put::<map_file::Model>(&state.db, map_id, relative_path, body).await?;
    Ok(Json(model.into()))
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path((map_id, relative_path)): Path<(i32, String)>,
) -> Result<StatusCode, AppError> {
    file_ops::delete::<map_file::Model>(&state.db, map_id, &relative_path).await
}
