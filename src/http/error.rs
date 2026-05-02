//! Single error type for every HTTP handler.
//!
//! Replaces the three error-tuple shapes that had grown organically:
//!
//! - `Result<T, StatusCode>` — bare status, no message body
//! - `Result<T, (StatusCode, &'static str)>` — task handlers
//! - `Result<T, (StatusCode, String)>` — file / upload handlers
//!
//! plus the bespoke `service::task::TaskServiceError` enum.
//!
//! Handlers now return `Result<T, AppError>`. `From<DbErr>` lets `?`
//! propagate database errors directly, with `RecordNotFound` mapping
//! to 404 (so missing rows surface as the right status without per-
//! call `.map_err(...)` boilerplate). Other variants fold via the
//! `not_found(...)`, `bad_request(...)` etc. constructors.

use std::borrow::Cow;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    BadRequest(Cow<'static, str>),
    NotFound(Cow<'static, str>),
    Conflict(Cow<'static, str>),
    Gone(Cow<'static, str>),
    Internal(Cow<'static, str>),
    Database(DbErr),
}

impl AppError {
    pub fn bad_request(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::BadRequest(msg.into())
    }
    pub fn not_found(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::NotFound(msg.into())
    }
    pub fn conflict(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Conflict(msg.into())
    }
    pub fn gone(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Gone(msg.into())
    }
    pub fn internal(msg: impl Into<Cow<'static, str>>) -> Self {
        Self::Internal(msg.into())
    }

    fn parts(&self) -> (StatusCode, String) {
        match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.to_string()),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.to_string()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.to_string()),
            AppError::Gone(m) => (StatusCode::GONE, m.to_string()),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.to_string()),
            // RecordNotFound is the one DB error that really means "404
            // resource missing"; everything else is a 500 with the
            // sea-orm display text so debugging logs catch it.
            AppError::Database(DbErr::RecordNotFound(m)) => (StatusCode::NOT_FOUND, m.clone()),
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
    }
}

impl From<DbErr> for AppError {
    fn from(e: DbErr) -> Self {
        AppError::Database(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = self.parts();
        // Log 5xx so operators see the underlying DbErr text without
        // surfacing it to the client; 4xx is just feedback.
        if status.is_server_error() {
            tracing::error!(%status, %message, "request failed");
        }
        (status, Json(json!({ "error": message }))).into_response()
    }
}
