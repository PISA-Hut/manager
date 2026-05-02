use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::map_file::MapFileMetaResponse;
use crate::http::handlers::bytes::{build_blob_response, sha256_hex};

pub async fn list_files(
    State(state): State<AppState>,
    Path(map_id): Path<i32>,
) -> Result<Json<Vec<MapFileMetaResponse>>, AppError> {
    let files = db::map_file::find_by_map(&state.db, map_id).await?;
    Ok(Json(files.into_iter().map(Into::into).collect()))
}

pub async fn get_file(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((map_id, relative_path)): Path<(i32, String)>,
) -> Result<axum::response::Response, AppError> {
    reject_traversal(&relative_path)?;

    let file = db::map_file::get(&state.db, map_id, &relative_path)
        .await?
        .ok_or_else(|| AppError::not_found("file not found"))?;

    Ok(build_blob_response(
        &headers,
        file.content,
        &file.content_sha256,
    ))
}

pub async fn put_file(
    State(state): State<AppState>,
    Path((map_id, relative_path)): Path<(i32, String)>,
    body: Bytes,
) -> Result<Json<MapFileMetaResponse>, AppError> {
    reject_traversal(&relative_path)?;

    let content = body.to_vec();
    let sha = sha256_hex(&content);
    let model = db::map_file::put(&state.db, map_id, relative_path, content, sha).await?;
    Ok(Json(model.into()))
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path((map_id, relative_path)): Path<(i32, String)>,
) -> Result<StatusCode, AppError> {
    reject_traversal(&relative_path)?;

    let deleted = db::map_file::delete(&state.db, map_id, &relative_path).await?;
    if deleted == 0 {
        return Err(AppError::not_found("file not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn reject_traversal(p: &str) -> Result<(), AppError> {
    if p.contains("..") || p.starts_with('/') {
        return Err(AppError::bad_request("invalid path"));
    }
    Ok(())
}
