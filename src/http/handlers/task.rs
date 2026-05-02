use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::task::{
    ClaimTaskRequest, ClaimTaskResponse, CreateTaskRequest, TaskResponse, TaskRunUpdateRequest,
};
use crate::service;

pub async fn list_tasks(
    State(state): State<AppState>,
) -> Result<Json<Vec<TaskResponse>>, AppError> {
    let tasks = db::task::find_all(&state.db).await?;
    Ok(Json(tasks.into_iter().map(TaskResponse::from).collect()))
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    if !db::plan::plan_exists(&state.db, payload.plan_id).await? {
        return Err(AppError::bad_request("Plan does not exist"));
    }
    if !db::av::av_exists(&state.db, payload.av_id).await? {
        return Err(AppError::bad_request("AV does not exist"));
    }
    if !db::sampler::sampler_exists(&state.db, payload.sampler_id).await? {
        return Err(AppError::bad_request("Sampler does not exist"));
    }
    if !db::simulator::simulator_exists(&state.db, payload.simulator_id).await? {
        return Err(AppError::bad_request("Simulator does not exist"));
    }

    let task = db::task::create(
        &state.db,
        payload.plan_id,
        payload.av_id,
        payload.sampler_id,
        payload.simulator_id,
    )
    .await?;
    Ok(Json(TaskResponse::from(task)))
}

pub async fn claim_task(
    State(state): State<AppState>,
    Json(req): Json<ClaimTaskRequest>,
) -> Result<Json<Option<ClaimTaskResponse>>, AppError> {
    let resp = service::task::claim_task_for_executor(
        &state,
        req.executor_id,
        req.task_id,
        req.map_id,
        req.scenario_id,
        req.av_id,
        req.simulator_id,
        req.sampler_id,
    )
    .await?;
    Ok(Json(resp))
}

/// `concrete_scenarios_executed` feeds the "ten useless runs in a row"
/// permanent-fail heuristic. Negative counts make no sense and would
/// silently bypass the `== 0` check, so reject them at the boundary
/// rather than letting them reach the DB.
fn validate_concrete_count(n: i32) -> Result<(), AppError> {
    if n < 0 {
        return Err(AppError::bad_request(
            "concrete_scenarios_executed must be >= 0",
        ));
    }
    Ok(())
}

pub async fn task_failed(
    State(state): State<AppState>,
    Json(payload): Json<TaskRunUpdateRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    validate_concrete_count(payload.concrete_scenarios_executed)?;
    let updated = service::task::fail_task(
        &state,
        payload.task_id,
        payload.reason,
        payload.log,
        payload.concrete_scenarios_executed,
    )
    .await?;
    Ok(Json(TaskResponse::from(updated)))
}

pub async fn task_completed(
    State(state): State<AppState>,
    Json(payload): Json<TaskRunUpdateRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    validate_concrete_count(payload.concrete_scenarios_executed)?;
    let updated = service::task::complete_task(
        &state,
        payload.task_id,
        payload.log,
        payload.concrete_scenarios_executed,
    )
    .await?;
    Ok(Json(TaskResponse::from(updated)))
}

pub async fn task_aborted(
    State(state): State<AppState>,
    Json(payload): Json<TaskRunUpdateRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    validate_concrete_count(payload.concrete_scenarios_executed)?;
    let updated = service::task::abort_task(
        &state,
        payload.task_id,
        payload.reason,
        payload.log,
        payload.concrete_scenarios_executed,
    )
    .await?;
    Ok(Json(TaskResponse::from(updated)))
}
