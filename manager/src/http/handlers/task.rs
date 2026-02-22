use axum::{Json, extract::State, http::StatusCode};

use crate::app_state::AppState;
use crate::db;
use crate::http::dto::task::{
    ClaimTaskRequest, ClaimTaskResponse, CreateTaskRequest, TaskFailedRequest, TaskResponse,
    TaskSucceededRequest,
};
use crate::service;

pub async fn list_tasks(
    State(state): State<AppState>,
) -> Result<Json<Vec<TaskResponse>>, (StatusCode, &'static str)> {
    let tasks = db::task::find_all(&state.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

    Ok(Json(tasks.into_iter().map(TaskResponse::from).collect()))
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, &'static str)> {
    if !db::plan::plan_exists(&state.db, payload.plan_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
    {
        return Err((StatusCode::BAD_REQUEST, "Plan does not exist"));
    }

    if !db::av::av_exists(&state.db, payload.av_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
    {
        return Err((StatusCode::BAD_REQUEST, "AV does not exist"));
    }

    if !db::sampler::sampler_exists(&state.db, payload.sampler_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
    {
        return Err((StatusCode::BAD_REQUEST, "Sampler does not exist"));
    }

    if !db::simulator::simulator_exists(&state.db, payload.simulator_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
    {
        return Err((StatusCode::BAD_REQUEST, "Simulator does not exist"));
    }

    let task = db::task::create(
        &state.db,
        payload.plan_id,
        payload.av_id,
        payload.sampler_id,
        payload.simulator_id,
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

    Ok(Json(TaskResponse::from(task)))
}

pub async fn claim_task(
    State(state): State<AppState>,
    Json(req): Json<ClaimTaskRequest>,
) -> Result<Json<Option<ClaimTaskResponse>>, StatusCode> {
    service::task::claim_task_for_worker(&state, req.worker_id)
        .await
        .map(Json)
        .map_err(|e| {
            let (status, _msg): (StatusCode, &'static str) = e.into();
            status
        })
}

pub async fn task_failed(
    State(state): State<AppState>,
    Json(payload): Json<TaskFailedRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, &'static str)> {
    service::task::fail_task(&state, payload.task_id, payload.reason)
        .await
        .map(TaskResponse::from)
        .map(Json)
        .map_err(|e| {
            let (status, msg): (StatusCode, &'static str) = e.into();
            (status, msg)
        })
}

pub async fn task_succeeded(
    State(state): State<AppState>,
    Json(payload): Json<TaskSucceededRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, &'static str)> {
    service::task::complete_task(&state, payload.task_id)
        .await
        .map(TaskResponse::from)
        .map(Json)
        .map_err(|e| {
            let (status, msg): (StatusCode, &'static str) = e.into();
            (status, msg)
        })
}
