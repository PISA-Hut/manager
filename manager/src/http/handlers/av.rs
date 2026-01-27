use axum::{Json, extract::State, http::StatusCode};

use crate::{
    app_state::AppState,
    db,
    entity::av,
    http::dto::av::{AvResponse, CreateAvRequest},
};

pub async fn list_av(State(state): State<AppState>) -> Result<Json<Vec<AvResponse>>, StatusCode> {
    let avs: Vec<av::Model> = db::av::find_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(avs.into_iter().map(AvResponse::from).collect()))
}

pub async fn create_av(
    State(state): State<AppState>,
    Json(payload): Json<CreateAvRequest>,
) -> Result<Json<AvResponse>, StatusCode> {
    let av_model: av::Model = db::av::create(&state.db, payload.name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AvResponse::from(av_model)))
}
