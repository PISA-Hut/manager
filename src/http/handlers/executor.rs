use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::executor::{CreateExecutorRequest, ExecutorResponse};

pub async fn list_executors(
    State(state): State<AppState>,
) -> Result<Json<Vec<ExecutorResponse>>, AppError> {
    let executors = db::executor::find_all(&state.db).await?;
    Ok(Json(
        executors.into_iter().map(ExecutorResponse::from).collect(),
    ))
}

pub async fn create_executor(
    State(state): State<AppState>,
    Json(payload): Json<CreateExecutorRequest>,
) -> Result<Json<ExecutorResponse>, AppError> {
    let executor = db::executor::create(
        &state.db,
        payload.job_id,
        payload.array_id,
        payload.node_list,
        payload.hostname,
    )
    .await?;
    Ok(Json(ExecutorResponse::from(executor)))
}
