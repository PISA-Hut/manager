use crate::entity::scenario;
use crate::entity::sea_orm_active_enums::ScenarioFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub format: ScenarioFormat,
    pub title: Option<String>,
    pub scenario_path: String,
    pub goal_config: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ScenarioResponse {
    pub id: i32,
    pub format: ScenarioFormat,
    pub title: Option<String>,
    pub scenario_path: String,
    pub goal_config: serde_json::Value,
}

impl From<scenario::Model> for ScenarioResponse {
    fn from(m: scenario::Model) -> Self {
        Self {
            id: m.id,
            format: m.scenario_format,
            title: m.title,
            scenario_path: m.scenario_path,
            goal_config: m.goal_config,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScenarioExecutionDto {
    pub format: ScenarioFormat,
    pub title: Option<String>,
    pub scenario_path: String,
    pub goal_config: serde_json::Value,
}

impl From<scenario::Model> for ScenarioExecutionDto {
    fn from(m: scenario::Model) -> Self {
        Self {
            format: m.scenario_format,
            title: m.title,
            scenario_path: m.scenario_path,
            goal_config: m.goal_config,
        }
    }
}
