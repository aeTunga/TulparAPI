use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("LZ4 decompression error: {0}")]
    Lz4(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Task join error: {0}")]
    TaskJoin(String),
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Internal database error".into())
            }
            AppError::Io(e) => {
                tracing::error!(error = %e, "IO error");
                (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", "Internal storage error".into())
            }
            AppError::Lz4(msg) => {
                tracing::error!(error = %msg, "LZ4 error");
                (StatusCode::INTERNAL_SERVER_ERROR, "DECOMPRESSION_ERROR", "Decompression failed".into())
            }
            AppError::Serde(e) => {
                tracing::error!(error = %e, "Serialization error");
                (StatusCode::INTERNAL_SERVER_ERROR, "SERIALIZATION_ERROR", "Data processing failed".into())
            }
            AppError::TaskJoin(msg) => {
                tracing::error!(error = %msg, "Task join error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal error".into())
            }
        };

        (status, Json(ErrorBody { code, message })).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
