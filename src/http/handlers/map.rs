use axum::{Json, extract::State};

use crate::app_state::AppState;
use crate::db;
use crate::http::AppError;
use crate::http::dto::map::{CreateMapRequest, MapResponse};

pub async fn list_maps(State(state): State<AppState>) -> Result<Json<Vec<MapResponse>>, AppError> {
    let maps = db::map::find_all(&state.db).await?;
    Ok(Json(maps.into_iter().map(MapResponse::from).collect()))
}

pub async fn create_map(
    State(state): State<AppState>,
    Json(payload): Json<CreateMapRequest>,
) -> Result<Json<MapResponse>, AppError> {
    let map = db::map::create(&state.db, payload.name).await?;
    Ok(Json(MapResponse::from(map)))
}
