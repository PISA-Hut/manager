use crate::entity::map;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateMapRequest {
    pub name: String,
    pub xodr: bool,
    pub osm: bool,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct MapResponse {
    pub id: i32,
    pub name: String,
    pub xodr: bool,
    pub osm: bool,
    pub path: String,
}

impl From<map::Model> for MapResponse {
    fn from(m: map::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            xodr: m.xodr,
            osm: m.osm,
            path: m.path,
        }
    }
}
