use crate::entity::task;
use sea_orm::prelude::DateTimeUtc;
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
    pub worker_id: Option<i32>,
    pub status: Option<String>,
    pub created_at: DateTimeUtc,
    pub executed_at: Option<DateTimeUtc>,
    pub finished_at: Option<DateTimeUtc>,
}

impl From<task::Model> for TaskResponse {
    fn from(m: task::Model) -> Self {
        Self {
            id: m.id,
            plan_id: m.plan_id,
            av_id: m.av_id,
            worker_id: m.worker_id,
            status: m.status,
            created_at: m.created_at,
            executed_at: m.executed_at,
            finished_at: m.finished_at,
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
