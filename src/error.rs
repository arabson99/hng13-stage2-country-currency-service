use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Country not found: {0}")]
    NotFound(String),

    #[allow(dead_code)]
    #[error("Validation failed")]
    ValidationError(HashMap<String, String>),

    #[error("External data source unavailable: {api_name}")]
    ApiError {
        #[source]
        source: reqwest::Error,
        api_name: String,
    },

    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Image generation failed: {0}")]
    ImageError(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::ApiError { .. } => StatusCode::SERVICE_UNAVAILABLE,
            AppError::DatabaseError(_) | AppError::Internal(_) | AppError::ImageError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        let (status, body) = match self {
            AppError::NotFound(message) => (
                self.status_code(),
                json!({ "error": message }),
            ),
            AppError::ValidationError(details) => (
                self.status_code(),
                json!({ "error": "Validation failed", "details": details }),
            ),
            AppError::ApiError { source, api_name } => (
                self.status_code(),
                json!({ "error": "External data source unavailable", "details": format!("Could not fetch data from {}: {}", api_name, source) }),
            ),
            AppError::DatabaseError(e) => {
                log::error!("Database error: {:?}", e);
                (
                    self.status_code(),
                    json!({ "error": "Internal server error" }),
                )
            }
            AppError::ImageError(e) => {
                log::error!("Image error: {:?}", e);
                (
                    self.status_code(),
                    json!({ "error": "Image generation failed" }),
                )
            }
            AppError::Internal(e) => {
                log::error!("Internal error: {:?}", e);
                (
                    self.status_code(),
                    json!({ "error": "Internal server error" }),
                )
            }
        };
        HttpResponse::build(status).json(body)
    }
}