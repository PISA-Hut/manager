use axum::{Json, extract::State, http::StatusCode};

use crate::{
    app_state::AppState,
    db,
    http::dto::map::{CreateMapRequest, MapResponse},
};

pub async fn list_maps(
    State(state): State<AppState>,
) -> Result<Json<Vec<MapResponse>>, StatusCode> {
    let maps = db::map::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(maps.into_iter().map(MapResponse::from).collect()))
}

pub async fn create_map(
    State(state): State<AppState>,
    Json(payload): Json<CreateMapRequest>,
) -> Result<Json<MapResponse>, StatusCode> {
    let map = db::map::create(
        &state.db,
        payload.name,
        payload.xodr,
        payload.osm,
        payload.path,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(MapResponse::from(map)))
}
