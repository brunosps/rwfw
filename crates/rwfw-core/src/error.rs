use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Validation error")]
    Validation(HashMap<String, Vec<String>>),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, serde_json::json!({ "error": msg })),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                serde_json::json!({ "error": msg }),
            ),
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, serde_json::json!({ "error": msg }))
            }
            AppError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                serde_json::json!({ "errors": errors }),
            ),
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, serde_json::json!({ "error": msg }))
            }
            AppError::Internal(err) => {
                tracing::error!(error = %err, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    serde_json::json!({ "error": "Internal server error" }),
                )
            }
        };

        (status, axum::Json(body)).into_response()
    }
}
