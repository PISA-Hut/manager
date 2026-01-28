use axum::http::StatusCode;
use sea_orm::DbErr;

use crate::app_state::AppState;
use crate::db;
use crate::entity::sea_orm_active_enums::TaskStatus as DbTaskStatus;
use crate::entity::{av, plan, sampler, simulator, task};
use crate::http::dto::av::AvResponse;
use crate::http::dto::plan::PlanResponse;
use crate::http::dto::sampler::SamplerResponse;
use crate::http::dto::simulator::SimulatorResponse;
use crate::http::dto::task::ClaimTaskResponse;
use crate::http::dto::task::TaskResponse;
use crate::http::dto::task::TaskStatusDto;

impl From<DbTaskStatus> for TaskStatusDto {
    fn from(value: DbTaskStatus) -> Self {
        match value {
            DbTaskStatus::Pending => TaskStatusDto::Pending,
            DbTaskStatus::InProgress => TaskStatusDto::InProgress,
            DbTaskStatus::Completed => TaskStatusDto::Completed,
            DbTaskStatus::Failed => TaskStatusDto::Failed,
        }
    }
}

#[derive(Debug)]
pub enum TaskServiceError {
    Database,
    NotFound(&'static str),
    InvalidState(&'static str),
    DataInconsistency(&'static str),
}

impl From<TaskServiceError> for (StatusCode, &'static str) {
    fn from(err: TaskServiceError) -> Self {
        match err {
            TaskServiceError::Database => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
            TaskServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            TaskServiceError::InvalidState(msg) => (StatusCode::BAD_REQUEST, msg),
            TaskServiceError::DataInconsistency(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        }
    }
}
impl From<DbErr> for TaskServiceError {
    fn from(_: DbErr) -> Self {
        TaskServiceError::Database
    }
}

pub struct ResolvedTask {
    pub task: task::Model,
    pub plan: plan::Model,
    pub av: av::Model,
    pub simulator: simulator::Model,
    pub sampler: sampler::Model,
}

pub async fn claim_task_for_worker(
    state: &AppState,
    worker_id: i32,
) -> Result<Option<ClaimTaskResponse>, TaskServiceError> {
    if db::worker::worker_exists(&state.db, worker_id).await? == false {
        return Err(TaskServiceError::NotFound("worker not found"));
    }

    let resolved = claim_and_resolve_task(&state, worker_id).await?;

    let resolved = match resolved {
        Some(r) => r,
        None => return Ok(None),
    };

    Ok(Some(ClaimTaskResponse {
        task: TaskResponse::from(resolved.task),
        plan: PlanResponse::from(resolved.plan),
        av: AvResponse::from(resolved.av),
        simulator: SimulatorResponse::from(resolved.simulator),
        sampler: SamplerResponse::from(resolved.sampler),
    }))
}

async fn claim_and_resolve_task(
    state: &AppState,
    worker_id: i32,
) -> Result<Option<ResolvedTask>, TaskServiceError> {
    let task = db::task::claim_task(&state.db, worker_id).await?;
    let task = match task {
        Some(t) => t,
        None => return Ok(None),
    };

    let plan = db::plan::get_by_id(&state.db, task.plan_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("plan not found"))?;
    let av = db::av::get_by_id(&state.db, task.av_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("av not found"))?;
    let simulator = db::simulator::get_by_id(&state.db, task.simulator_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("simulator not found"))?;
    let sampler = db::sampler::get_by_id(&state.db, task.sampler_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("sampler not found"))?;

    Ok(Some(ResolvedTask {
        task,
        plan,
        av,
        simulator,
        sampler,
    }))
}
