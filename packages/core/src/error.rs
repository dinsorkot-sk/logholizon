use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(anyhow::Error),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest(message)
            | Self::NotFound(message)
            | Self::Conflict(message)
            | Self::Unauthorized(message)
            | Self::Forbidden(message) => formatter.write_str(message),
            Self::Internal(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            Self::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            Self::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            Self::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                err.to_string(),
            ),
        };
        (status, Json(ApiError::new(code, message))).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        if let Some(app) = err.downcast_ref::<AppError>() {
            return match app {
                AppError::BadRequest(msg) => AppError::BadRequest(msg.clone()),
                AppError::NotFound(msg) => AppError::NotFound(msg.clone()),
                AppError::Conflict(msg) => AppError::Conflict(msg.clone()),
                AppError::Unauthorized(msg) => AppError::Unauthorized(msg.clone()),
                AppError::Forbidden(msg) => AppError::Forbidden(msg.clone()),
                AppError::Internal(_) => AppError::Internal(anyhow::anyhow!("internal error")),
            };
        }
        Self::Internal(err)
    }
}
