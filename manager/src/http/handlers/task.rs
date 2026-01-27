use axum::{Json, extract::State, http::StatusCode};

use crate::{
    app_state::AppState,
    db,
    http::dto::task::{ClaimTaskRequest, CompleteTaskRequest, CreateTaskRequest, TaskResponse},
};

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
    // Validate foreign keys first
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

    // Create task (unassigned)
    let task = db::task::create(&state.db, payload.plan_id, payload.av_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

    Ok(Json(TaskResponse::from(task)))
}

pub async fn claim_task(
    State(state): State<AppState>,
    Json(payload): Json<ClaimTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, &'static str)> {
    // Validate worker exists
    if !db::worker::worker_exists(&state.db, payload.worker_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?
    {
        return Err((StatusCode::BAD_REQUEST, "Worker does not exist"));
    }

    let task = db::task::claim_one_unassigned(&state.db, payload.worker_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

    match task {
        Some(t) => Ok(Json(TaskResponse::from(t))),
        None => Err((StatusCode::NOT_FOUND, "No unassigned tasks available")),
    }
}

pub async fn complete_task(
    State(state): State<AppState>,
    Json(payload): Json<CompleteTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, &'static str)> {
    let task = db::task::complete_task(&state.db, payload.task_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error"))?;

    let Some(task) = task else {
        return Err((StatusCode::BAD_REQUEST, "Task does not exist"));
    };

    Ok(Json(TaskResponse::from(task)))
}
