use crate::entity::av;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AvResponse {
    pub id: i32,
    pub name: String,
}

impl From<av::Model> for AvResponse {
    fn from(m: av::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAvRequest {
    pub name: String,
}
