use crate::entity::scenario;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub scenario_path: String,
    pub param_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScenarioResponse {
    pub id: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub scenario_path: String,
    pub param_path: Option<String>,
}

impl From<scenario::Model> for ScenarioResponse {
    fn from(m: scenario::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            description: m.description,
            scenario_path: m.scenario_path,
            param_path: m.param_path,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScenarioExecutionDto {
    pub title: Option<String>,
    pub description: Option<String>,
    pub scenario_path: String,
    pub param_path: Option<String>,
}

impl From<scenario::Model> for ScenarioExecutionDto {
    fn from(m: scenario::Model) -> Self {
        Self {
            title: m.title,
            description: m.description,
            scenario_path: m.scenario_path,
            param_path: m.param_path,
        }
    }
}
