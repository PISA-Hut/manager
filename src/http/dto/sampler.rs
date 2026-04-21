use crate::entity::sampler;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SamplerResponse {
    pub id: i32,
    pub name: String,
    pub module_path: String,
    pub config_sha256: Option<String>,
}

impl From<sampler::Model> for SamplerResponse {
    fn from(m: sampler::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            module_path: m.module_path,
            config_sha256: m.config_sha256,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateSamplerRequest {
    pub name: String,
    pub module_path: String,
}

#[derive(Debug, Serialize)]
pub struct SamplerExecutionDto {
    pub id: i32,
    pub name: String,
    pub module_path: String,
    pub config_sha256: Option<String>,
}

impl From<sampler::Model> for SamplerExecutionDto {
    fn from(m: sampler::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            module_path: m.module_path,
            config_sha256: m.config_sha256,
        }
    }
}
