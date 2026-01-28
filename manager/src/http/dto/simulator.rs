use crate::entity::simulator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateSimulatorRequest {
    pub name: String,
    pub module_path: String,
}

#[derive(Debug, Serialize)]
pub struct SimulatorResponse {
    pub id: i32,
    pub name: String,
    pub module_path: String,
}

impl From<simulator::Model> for SimulatorResponse {
    fn from(m: simulator::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            module_path: m.module_path,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SimulatorExecutionDto {
    pub name: String,
    pub module_path: String,
}

impl From<simulator::Model> for SimulatorExecutionDto {
    fn from(m: simulator::Model) -> Self {
        Self {
            name: m.name,
            module_path: m.module_path,
        }
    }
}
