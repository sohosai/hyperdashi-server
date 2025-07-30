use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    InternalServerError(String),
    DatabaseError(sqlx::Error),
    ConfigError(config::ConfigError),
    IoError(std::io::Error),
    ValidationError(String),
    StorageError(String),
    InternalServer(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            AppError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AppError::ConfigError(err) => write!(f, "Configuration error: {}", err),
            AppError::IoError(err) => write!(f, "IO error: {}", err),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            AppError::InternalServer(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::DatabaseError(ref err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error occurred".to_string(),
                )
            }
            AppError::ConfigError(ref err) => {
                tracing::error!("Config error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Configuration error occurred".to_string(),
                )
            }
            AppError::IoError(ref err) => {
                tracing::error!("IO error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "IO error occurred".to_string(),
                )
            }
            AppError::StorageError(ref msg) => {
                tracing::error!("Storage error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Storage error occurred".to_string(),
                )
            }
            AppError::InternalServer(ref msg) => {
                tracing::error!("Internal server error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<config::ConfigError> for AppError {
    fn from(err: config::ConfigError) -> Self {
        AppError::ConfigError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

pub type AppResult<T> = Result<T, AppError>;
