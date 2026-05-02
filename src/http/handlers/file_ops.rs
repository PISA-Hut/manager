//! Generic CRUD helpers for `(parent_id, relative_path) → bytes`
//! tables — backed by the `db::FileStore` trait, used by both
//! `scenario_file` and `map_file` handlers.

use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
};
use sea_orm::DatabaseConnection;

use crate::db::FileStore;
use crate::http::AppError;
use crate::http::handlers::bytes::{build_blob_response, sha256_hex};

/// `..` and absolute paths are rejected at the handler boundary so a
/// crafted `relative_path` can't escape the parent's namespace.
fn reject_traversal(p: &str) -> Result<(), AppError> {
    if p.contains("..") || p.starts_with('/') {
        return Err(AppError::bad_request("invalid path"));
    }
    Ok(())
}

pub async fn list<E, R>(
    db: &DatabaseConnection,
    parent_id: i32,
    into_meta: impl Fn(E) -> R,
) -> Result<Vec<R>, AppError>
where
    E: FileStore,
{
    let files = E::find_by_parent(db, parent_id).await?;
    Ok(files.into_iter().map(into_meta).collect())
}

pub async fn get<E: FileStore>(
    headers: &HeaderMap,
    db: &DatabaseConnection,
    parent_id: i32,
    relative_path: &str,
) -> Result<axum::response::Response, AppError> {
    reject_traversal(relative_path)?;
    let file = E::get(db, parent_id, relative_path)
        .await?
        .ok_or_else(|| AppError::not_found("file not found"))?;
    Ok(build_blob_response(
        headers,
        file.content().to_vec(),
        file.content_sha256(),
    ))
}

pub async fn put<E: FileStore>(
    db: &DatabaseConnection,
    parent_id: i32,
    relative_path: String,
    body: Bytes,
) -> Result<E, AppError> {
    reject_traversal(&relative_path)?;
    let content = body.to_vec();
    let sha = sha256_hex(&content);
    Ok(E::put(db, parent_id, relative_path, content, sha).await?)
}

pub async fn delete<E: FileStore>(
    db: &DatabaseConnection,
    parent_id: i32,
    relative_path: &str,
) -> Result<StatusCode, AppError> {
    reject_traversal(relative_path)?;
    let deleted = E::delete(db, parent_id, relative_path).await?;
    if deleted == 0 {
        return Err(AppError::not_found("file not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
