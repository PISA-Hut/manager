use crate::entity::map;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateMapRequest {
    pub name: String,
    pub xodr_path: Option<String>,
    pub osm_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MapResponse {
    pub id: i32,
    pub name: String,
    pub xodr_path: Option<String>,
    pub osm_path: Option<String>,
}

impl From<map::Model> for MapResponse {
    fn from(m: map::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            xodr_path: m.xodr_path,
            osm_path: m.osm_path,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MapExecutionDto {
    pub name: String,
    pub xodr_path: Option<String>,
    pub osm_path: Option<String>,
}

impl From<map::Model> for MapExecutionDto {
    fn from(m: map::Model) -> Self {
        Self {
            name: m.name,
            xodr_path: m.xodr_path,
            osm_path: m.osm_path,
        }
    }
}
