use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Conflict(String),
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self::Internal(value)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Internal(value.into())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEnvelope {
    error: ErrorObject,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorObject {
    code: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(message) => {
                (axum::http::StatusCode::NOT_FOUND, "NOT_FOUND", message)
            }
            AppError::BadRequest(message) => (
                axum::http::StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                message,
            ),
            AppError::Unauthorized(message) => (
                axum::http::StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                message,
            ),
            AppError::Conflict(message) => (axum::http::StatusCode::CONFLICT, "CONFLICT", message),
            AppError::Internal(error) => {
                tracing::error!(error = ?error, "internal server error");
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR",
                    "Unexpected server error".to_string(),
                )
            }
        };

        (
            status,
            Json(ErrorEnvelope {
                error: ErrorObject {
                    code: code.to_string(),
                    message,
                },
            }),
        )
            .into_response()
    }
}
