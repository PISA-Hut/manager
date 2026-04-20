use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse};
use std::path::Path;

use crate::app_state::AppState;

#[derive(serde::Deserialize)]
pub struct FileQuery {
    pub path: String,
}

pub async fn get_scenario_file(
    State(state): State<AppState>,
    Query(query): Query<FileQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Prevent path traversal
    if query.path.contains("..") {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_string()));
    }

    let file_path = Path::new(&state.scenario_storage_dir).join(&query.path);

    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {e}")))?;

    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/xml; charset=utf-8")],
        content,
    ))
}
