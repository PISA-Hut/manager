use axum::http::StatusCode;
use sea_orm::DbErr;

use crate::app_state::AppState;
use crate::db;
use crate::entity::{av, map, sampler, scenario, simulator, task};
use crate::http::dto::av::AvExecutionDto;
use crate::http::dto::map::MapExecutionDto;
use crate::http::dto::sampler::SamplerExecutionDto;
use crate::http::dto::scenario::ScenarioExecutionDto;
use crate::http::dto::simulator::SimulatorExecutionDto;
use crate::http::dto::task::{ClaimTaskResponse, TaskExecutionDto};

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
    pub av: av::Model,
    pub map: map::Model,
    pub scenario: scenario::Model,
    pub simulator: simulator::Model,
    pub sampler: sampler::Model,
    pub task_run_id: i32,
}

pub async fn claim_task_for_executor(
    state: &AppState,
    executor_id: i32,
    task_id: Option<i32>,
    map_id: Option<i32>,
    scenario_id: Option<i32>,
    av_id: Option<i32>,
    simulator_id: Option<i32>,
    sampler_id: Option<i32>,
) -> Result<Option<ClaimTaskResponse>, TaskServiceError> {
    if !db::executor::executor_exists(&state.db, executor_id).await? {
        return Err(TaskServiceError::NotFound("worker not found"));
    }

    let resolved = claim_and_resolve_task(
        state,
        executor_id,
        task_id,
        map_id,
        scenario_id,
        av_id,
        simulator_id,
        sampler_id,
    )
    .await?;

    let resolved = match resolved {
        Some(r) => r,
        None => return Ok(None),
    };

    Ok(Some(ClaimTaskResponse {
        task: TaskExecutionDto::from(resolved.task),
        task_run_id: resolved.task_run_id,
        av: AvExecutionDto::from(resolved.av),
        simulator: SimulatorExecutionDto::from(resolved.simulator),
        scenario: ScenarioExecutionDto::from(resolved.scenario),
        sampler: SamplerExecutionDto::from(resolved.sampler),
        map: MapExecutionDto::from(resolved.map),
    }))
}

async fn claim_and_resolve_task(
    state: &AppState,
    executor_id: i32,
    task_id: Option<i32>,
    map_id: Option<i32>,
    scenario_id: Option<i32>,
    av_id: Option<i32>,
    simulator_id: Option<i32>,
    sampler_id: Option<i32>,
) -> Result<Option<ResolvedTask>, TaskServiceError> {
    let claimed = db::task::claim_task_with_filters(
        &state.db,
        executor_id,
        task_id,
        map_id,
        scenario_id,
        av_id,
        simulator_id,
        sampler_id,
    )
    .await?;
    let (task, task_run_id) = match claimed {
        Some(t) => t,
        None => return Ok(None),
    };

    let plan = db::plan::get_by_id(&state.db, task.plan_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("plan not found"))?;
    let av = db::av::get_by_id(&state.db, task.av_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("av not found"))?;
    let map = db::map::get_by_id(&state.db, plan.map_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("map not found"))?;
    let simulator = db::simulator::get_by_id(&state.db, task.simulator_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("simulator not found"))?;
    let scenario = db::scenario::get_by_id(&state.db, plan.scenario_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("scenario not found"))?;
    let sampler = db::sampler::get_by_id(&state.db, task.sampler_id)
        .await?
        .ok_or(TaskServiceError::DataInconsistency("sampler not found"))?;

    Ok(Some(ResolvedTask {
        task,
        av,
        map,
        scenario,
        simulator,
        sampler,
        task_run_id,
    }))
}

pub async fn complete_task(
    state: &AppState,
    task_id: i32,
    log: Option<String>,
    concrete_scenarios_executed: i32,
) -> Result<task::Model, TaskServiceError> {
    println!(
        "Completing task {} (concrete_scenarios_executed={})",
        task_id, concrete_scenarios_executed
    );
    let updated =
        db::task::complete_task(&state.db, task_id, log, concrete_scenarios_executed).await?;
    let updated = match updated {
        Some(t) => t,
        None => return Err(TaskServiceError::NotFound("task not found")),
    };

    Ok(updated)
}

pub async fn fail_task(
    state: &AppState,
    task_id: i32,
    reason: Option<String>,
    log: Option<String>,
    concrete_scenarios_executed: i32,
) -> Result<task::Model, TaskServiceError> {
    let reason = reason.unwrap_or_else(|| "task failed".to_string());
    println!(
        "Failing task {} with reason: {} (concrete_scenarios_executed={})",
        task_id, reason, concrete_scenarios_executed
    );
    let updated = db::task::fail_task(
        &state.db,
        task_id,
        reason,
        log,
        concrete_scenarios_executed,
        state.useless_streak_limit,
    )
    .await?;
    let updated = match updated {
        Some(t) => t,
        None => return Err(TaskServiceError::NotFound("task not found")),
    };

    Ok(updated)
}

pub async fn abort_task(
    state: &AppState,
    task_id: i32,
    reason: Option<String>,
    log: Option<String>,
    concrete_scenarios_executed: i32,
) -> Result<task::Model, TaskServiceError> {
    let reason = reason.unwrap_or_else(|| "task aborted".to_string());
    println!(
        "Aborting task {} with reason: {} (concrete_scenarios_executed={})",
        task_id, reason, concrete_scenarios_executed
    );
    let updated =
        db::task_run::abort_task(&state.db, task_id, reason, log, concrete_scenarios_executed)
            .await?;
    let updated = match updated {
        Some(t) => t,
        None => return Err(TaskServiceError::NotFound("task not found")),
    };
    Ok(updated)
}
