//! HTTP handlers for `scenario_file` rows. The body is a thin
//! per-entity adapter on top of the generic `db::FileStore` helpers
//! shared with `map_file` — this module exists primarily so the
//! router (and Swagger) sees `/scenario/{id}/file/...` as a distinct
//! endpoint group.

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};

use crate::app_state::AppState;
use crate::entity::scenario_file;
use crate::http::AppError;
use crate::http::dto::scenario_file::ScenarioFileMetaResponse;
use crate::http::handlers::file_ops;

pub async fn list_files(
    State(state): State<AppState>,
    Path(scenario_id): Path<i32>,
) -> Result<Json<Vec<ScenarioFileMetaResponse>>, AppError> {
    let metas =
        file_ops::list::<scenario_file::Model, _>(&state.db, scenario_id, Into::into).await?;
    Ok(Json(metas))
}

pub async fn get_file(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path((scenario_id, relative_path)): Path<(i32, String)>,
) -> Result<axum::response::Response, AppError> {
    file_ops::get::<scenario_file::Model>(&headers, &state.db, scenario_id, &relative_path).await
}

pub async fn put_file(
    State(state): State<AppState>,
    Path((scenario_id, relative_path)): Path<(i32, String)>,
    body: Bytes,
) -> Result<Json<ScenarioFileMetaResponse>, AppError> {
    let model =
        file_ops::put::<scenario_file::Model>(&state.db, scenario_id, relative_path, body).await?;
    Ok(Json(model.into()))
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path((scenario_id, relative_path)): Path<(i32, String)>,
) -> Result<StatusCode, AppError> {
    file_ops::delete::<scenario_file::Model>(&state.db, scenario_id, &relative_path).await
}
