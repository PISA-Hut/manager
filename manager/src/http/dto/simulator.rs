use crate::entity::simulator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateSimulatorRequest {
    pub name: String,
    pub image_path: String,
    pub config_path: String,
    pub nv_runtime: bool,
}

#[derive(Debug, Serialize)]
pub struct SimulatorResponse {
    pub id: i32,
    pub name: String,
    pub image_path: String,
    pub config_path: String,
    pub nv_runtime: bool,
}

impl From<simulator::Model> for SimulatorResponse {
    fn from(m: simulator::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            image_path: m.image_path,
            config_path: m.config_path,
            nv_runtime: m.nv_runtime,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SimulatorExecutionDto {
    pub name: String,
    pub image_path: String,
    pub config_path: String,
    pub nv_runtime: bool,
}

impl From<simulator::Model> for SimulatorExecutionDto {
    fn from(m: simulator::Model) -> Self {
        Self {
            name: m.name,
            image_path: m.image_path,
            config_path: m.config_path,
            nv_runtime: m.nv_runtime,
        }
    }
}
