use crate::entity::map;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateMapRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct MapResponse {
    pub id: i32,
    pub name: String,
}

impl From<map::Model> for MapResponse {
    fn from(m: map::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MapExecutionDto {
    pub id: i32,
    pub name: String,
}

impl From<map::Model> for MapExecutionDto {
    fn from(m: map::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
        }
    }
}
