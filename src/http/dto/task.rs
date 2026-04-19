use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::entity::sea_orm_active_enums::TaskStatus;
use crate::entity::task;
use crate::http::dto::{
    av::AvExecutionDto, map::MapExecutionDto, sampler::SamplerExecutionDto,
    scenario::ScenarioExecutionDto, simulator::SimulatorExecutionDto,
};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub plan_id: i32,
    pub av_id: i32,
    pub sampler_id: i32,
    pub simulator_id: i32,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: i32,
    pub plan_id: i32,
    pub av_id: i32,
    pub simulator_id: i32,
    pub sampler_id: i32,
    pub task_status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub retry_count: i32,
}

impl From<task::Model> for TaskResponse {
    fn from(m: task::Model) -> Self {
        Self {
            id: m.id,
            plan_id: m.plan_id,
            av_id: m.av_id,
            simulator_id: m.simulator_id,
            sampler_id: m.sampler_id,
            task_status: m.task_status,
            created_at: m.created_at.with_timezone(&Utc),
            retry_count: m.retry_count,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClaimTaskRequest {
    pub executor_id: i32,
    pub map_id: Option<i32>,
    pub scenario_id: Option<i32>,
    pub av_id: Option<i32>,
    pub simulator_id: Option<i32>,
    pub sampler_id: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ClaimTaskResponse {
    pub task: TaskExecutionDto,
    pub av: AvExecutionDto,
    pub map: MapExecutionDto,
    pub scenario: ScenarioExecutionDto,
    pub simulator: SimulatorExecutionDto,
    pub sampler: SamplerExecutionDto,
}

#[derive(Debug, Serialize)]
pub struct TaskExecutionDto {
    pub id: i32,
}

impl From<task::Model> for TaskExecutionDto {
    fn from(m: task::Model) -> Self {
        Self { id: m.id }
    }
}

#[derive(Debug, Deserialize)]
pub struct TaskRunUpdateRequest {
    pub task_id: i32,
    pub reason: Option<String>,
}
