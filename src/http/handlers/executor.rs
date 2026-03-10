use axum::{Json, extract::State, http::StatusCode};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::executor::{CreateExecutorRequest, ExecutorResponse};

pub async fn list_executors(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExecutorResponse>>, StatusCode> {
    let executors = db::executor::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        executors.into_iter().map(ExecutorResponse::from).collect(),
    ))
}

pub async fn create_executor(
    State(state): State<AppState>,
    Json(payload): Json<CreateExecutorRequest>,
) -> Result<Json<ExecutorResponse>, StatusCode> {
    let executor = db::executor::create(
        &state.db,
        payload.job_id,
        payload.array_id,
        payload.node_list,
        payload.hostname,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ExecutorResponse::from(executor)))
}
