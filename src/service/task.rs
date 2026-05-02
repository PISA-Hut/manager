use crate::app_state::AppState;
use crate::db;
use crate::entity::{av, map, sampler, scenario, simulator, task};
use crate::http::AppError;
use crate::http::dto::av::AvExecutionDto;
use crate::http::dto::map::MapExecutionDto;
use crate::http::dto::sampler::SamplerExecutionDto;
use crate::http::dto::scenario::ScenarioExecutionDto;
use crate::http::dto::simulator::SimulatorExecutionDto;
use crate::http::dto::task::{ClaimTaskResponse, TaskExecutionDto};

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
) -> Result<Option<ClaimTaskResponse>, AppError> {
    if !db::executor::executor_exists(&state.db, executor_id).await? {
        return Err(AppError::not_found("worker not found"));
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
) -> Result<Option<ResolvedTask>, AppError> {
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

    // These are foreign-key joins on a row we just claimed; if any are
    // missing the database is internally inconsistent (referential
    // integrity broke) — surface as 500 with a specific message rather
    // than a generic db error.
    let plan = db::plan::get_by_id(&state.db, task.plan_id)
        .await?
        .ok_or_else(|| AppError::internal("data inconsistency: plan not found"))?;
    let av = db::av::get_by_id(&state.db, task.av_id)
        .await?
        .ok_or_else(|| AppError::internal("data inconsistency: av not found"))?;
    let map = db::map::get_by_id(&state.db, plan.map_id)
        .await?
        .ok_or_else(|| AppError::internal("data inconsistency: map not found"))?;
    let simulator = db::simulator::get_by_id(&state.db, task.simulator_id)
        .await?
        .ok_or_else(|| AppError::internal("data inconsistency: simulator not found"))?;
    let scenario = db::scenario::get_by_id(&state.db, plan.scenario_id)
        .await?
        .ok_or_else(|| AppError::internal("data inconsistency: scenario not found"))?;
    let sampler = db::sampler::get_by_id(&state.db, task.sampler_id)
        .await?
        .ok_or_else(|| AppError::internal("data inconsistency: sampler not found"))?;

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
) -> Result<task::Model, AppError> {
    tracing::info!(task_id, concrete_scenarios_executed, "completing task");
    let updated =
        db::task::complete_task(&state.db, task_id, log, concrete_scenarios_executed).await?;
    updated.ok_or_else(|| AppError::not_found("task not found"))
}

pub async fn fail_task(
    state: &AppState,
    task_id: i32,
    reason: Option<String>,
    log: Option<String>,
    concrete_scenarios_executed: i32,
) -> Result<task::Model, AppError> {
    let reason = reason.unwrap_or_else(|| "task failed".to_string());
    tracing::info!(
        task_id,
        %reason,
        concrete_scenarios_executed,
        "failing task"
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
    updated.ok_or_else(|| AppError::not_found("task not found"))
}

pub async fn abort_task(
    state: &AppState,
    task_id: i32,
    reason: Option<String>,
    log: Option<String>,
    concrete_scenarios_executed: i32,
) -> Result<task::Model, AppError> {
    let reason = reason.unwrap_or_else(|| "task aborted".to_string());
    tracing::info!(
        task_id,
        %reason,
        concrete_scenarios_executed,
        "aborting task"
    );
    let updated =
        db::task_run::abort_task(&state.db, task_id, reason, log, concrete_scenarios_executed)
            .await?;
    updated.ok_or_else(|| AppError::not_found("task not found"))
}
