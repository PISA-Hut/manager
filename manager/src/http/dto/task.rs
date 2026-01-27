use crate::entity::task;
use chrono::{DateTime, Utc};
use sea_orm::ActiveEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub plan_id: i32,
    pub av_id: i32,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: i32,
    pub plan_id: i32,
    pub av_id: i32,
    pub simulator_id: i32,
    pub sampler_id: i32,
    pub worker_id: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl From<task::Model> for TaskResponse {
    fn from(m: task::Model) -> Self {
        Self {
            id: m.id,
            plan_id: m.plan_id,
            av_id: m.av_id,
            simulator_id: m.simulator_id,
            sampler_id: m.sampler_id,
            worker_id: m.worker_id,
            status: ActiveEnum::to_value(&m.status),
            created_at: m.created_at.with_timezone(&Utc),
            executed_at: m.executed_at.map(|dt| dt.with_timezone(&Utc)),
            finished_at: m.finished_at.map(|dt| dt.with_timezone(&Utc)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClaimTaskRequest {
    pub worker_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CompleteTaskRequest {
    pub task_id: i32,
}
