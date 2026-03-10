use crate::entity::executor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateExecutorRequest {
    pub job_id: i32,
    #[serde(default)]
    pub array_id: i32,
    pub node_list: String,
    pub hostname: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutorResponse {
    pub id: i32,
    pub job_id: i32,
    pub array_id: i32,
    pub node_list: String,
    pub hostname: String,
}

impl From<executor::Model> for ExecutorResponse {
    fn from(m: executor::Model) -> Self {
        Self {
            id: m.id,
            job_id: m.slurm_job_id,
            array_id: m.slurm_array_id,
            node_list: m.slurm_node_list,
            hostname: m.hostname,
        }
    }
}
