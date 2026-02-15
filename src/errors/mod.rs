/// BAD REQUEST IS COMMENTED OUT FOR NOW, UNCOMMENT IF NEEDED IN THE FUTURE
use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Not Found")]
    NotFound(String),

    // #[error("Bad Request: {0}")]
    // BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden(String),

    #[error("Internal Server Error")]
    InternalServerError(String),

    #[error("Validation Error: {0}")]
    ValidationError(String),

    #[error("Database Error: {0}")]
    DatabaseError(String),

    #[error("JWT Error: {0}")]
    JwtError(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            // ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", msg),
            ),
            ApiError::JwtError(msg) => (StatusCode::UNAUTHORIZED, format!("JWT error: {}", msg)),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ApiError::InternalServerError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error: {}", msg),
            ),
        };

        let error_response = ErrorResponse {
            error: status.canonical_reason().unwrap_or("Error").to_string(),
            message,
            status: status.as_u16(),
        };

        HttpResponse::build(status).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            // ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::JwtError(_) => StatusCode::UNAUTHORIZED,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
