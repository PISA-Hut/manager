use crate::entity::scenario;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub title: String,
    pub scenario_path: String,
    pub param_path: String,
}

#[derive(Debug, Serialize)]
pub struct ScenarioResponse {
    pub id: i32,
    pub title: String,
    pub scenario_path: String,
    pub param_path: String,
}

impl From<scenario::Model> for ScenarioResponse {
    fn from(m: scenario::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            scenario_path: m.scenario_path,
            param_path: m.param_path,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScenarioExecutionDto {
    pub title: String,
    pub scenario_path: String,
    pub param_path: String,
}

impl From<scenario::Model> for ScenarioExecutionDto {
    fn from(m: scenario::Model) -> Self {
        Self {
            title: m.title,
            scenario_path: m.scenario_path,
            param_path: m.param_path,
        }
    }
}
