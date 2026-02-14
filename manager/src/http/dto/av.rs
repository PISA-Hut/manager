use crate::entity::av;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AvResponse {
    pub id: i32,
    pub name: String,
    pub image_path: String,
    pub config_path: String,
    pub nv_runtime: bool,
}

impl From<av::Model> for AvResponse {
    fn from(m: av::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            image_path: m.image_path,
            config_path: m.config_path,
            nv_runtime: m.nv_runtime,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAvRequest {
    pub name: String,
    pub image_path: String,
    pub config_path: String,
    pub nv_runtime: bool,
}

#[derive(Debug, Serialize)]
pub struct AvExecutionDto {
    pub name: String,
    pub image_path: String,
    pub config_path: String,
    pub nv_runtime: bool,
}

impl From<av::Model> for AvExecutionDto {
    fn from(m: av::Model) -> Self {
        Self {
            name: m.name,
            image_path: m.image_path,
            config_path: m.config_path,
            nv_runtime: m.nv_runtime,
        }
    }
}
