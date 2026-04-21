use crate::entity::scenario;
use crate::entity::sea_orm_active_enums::ScenarioFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateScenarioRequest {
    pub format: ScenarioFormat,
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScenarioResponse {
    pub id: i32,
    pub format: ScenarioFormat,
    pub title: Option<String>,
}

impl From<scenario::Model> for ScenarioResponse {
    fn from(m: scenario::Model) -> Self {
        Self {
            id: m.id,
            format: m.scenario_format,
            title: m.title,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScenarioExecutionDto {
    pub id: i32,
    pub format: ScenarioFormat,
    pub title: Option<String>,
}

impl From<scenario::Model> for ScenarioExecutionDto {
    fn from(m: scenario::Model) -> Self {
        Self {
            id: m.id,
            format: m.scenario_format,
            title: m.title,
        }
    }
}
