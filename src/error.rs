use crate::models::ProblemDetails;
use actix_web::{
    error::{JsonPayloadError, ResponseError},
    http::StatusCode,
    HttpResponse,
};
use std::fmt;

impl ProblemDetails {
    pub fn new(title: &str, status: StatusCode, detail: &str, instance: Option<String>) -> Self {
        ProblemDetails {
            title: title.to_string(),
            status: status.as_u16(),
            detail: detail.to_string(),
            instance,
        }
    }

    pub fn puzzle_not_generated(instance: Option<String>) -> Self {
        ProblemDetails::new(
            "Puzzle not ready",
            StatusCode::SERVICE_UNAVAILABLE,
            "Please wait while puzzle is being generated.",
            instance,
        )
    }

    pub fn invalid_request(detail: &str, instance: Option<String>) -> Self {
        ProblemDetails::new("Invalid request", StatusCode::BAD_REQUEST, detail, instance)
    }

    pub fn validation_error(detail: &str, instance: Option<String>) -> Self {
        ProblemDetails::new(
            "Validation failed",
            StatusCode::BAD_REQUEST,
            detail,
            instance,
        )
    }

    pub fn internal_error(instance: Option<String>) -> Self {
        ProblemDetails::new(
            "Internal server error",
            StatusCode::INTERNAL_SERVER_ERROR,
            "An unexpected error occurred.",
            instance,
        )
    }
}

impl fmt::Display for ProblemDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.title, self.detail)
    }
}

impl ResponseError for ProblemDetails {
    fn error_response(&self) -> HttpResponse {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        HttpResponse::build(status).json(self)
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Debug)]
pub enum ApiError {
    ValidationError(String),
    DatabaseError(String),
    NotFound,
    JwtError(String),
    ExternalServiceError(String),
    Unauthorized,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            ApiError::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            ApiError::NotFound => write!(f, "Resource not found"),
            ApiError::JwtError(msg) => write!(f, "JWT error: {msg}"),
            ApiError::ExternalServiceError(msg) => write!(f, "External service error: {msg}"),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::DatabaseError(msg) => {
                if msg == "Puzzle not generated yet" {
                    StatusCode::SERVICE_UNAVAILABLE
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::JwtError(_) | ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::ExternalServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::ValidationError(msg) => {
                let details = ProblemDetails::validation_error(msg, None);
                HttpResponse::BadRequest().json(details)
            }
            ApiError::DatabaseError(msg) => {
                tracing::error!("Database error: {msg}");
                let (status, details) = if msg == "Puzzle not generated yet" {
                    (
                        StatusCode::SERVICE_UNAVAILABLE,
                        ProblemDetails::puzzle_not_generated(None),
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ProblemDetails::internal_error(None),
                    )
                };
                HttpResponse::build(status).json(details)
            }
            ApiError::NotFound => {
                let details = ProblemDetails::invalid_request("Resource not found", None);
                HttpResponse::NotFound().json(details)
            }
            ApiError::JwtError(msg) => {
                tracing::warn!("JWT error: {msg}");
                let details = ProblemDetails::new(
                    "Unauthorized",
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token",
                    None,
                );
                HttpResponse::Unauthorized().json(details)
            }
            ApiError::ExternalServiceError(msg) => {
                tracing::error!("External service error: {msg}");
                let details = ProblemDetails::internal_error(None);
                HttpResponse::InternalServerError().json(details)
            }
            ApiError::Unauthorized => {
                let details = ProblemDetails::new(
                    "Unauthorized",
                    StatusCode::UNAUTHORIZED,
                    "Authentication required",
                    None,
                );
                HttpResponse::Unauthorized().json(details)
            }
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound,
            e => ApiError::DatabaseError(e.to_string()),
        }
    }
}

impl From<JsonPayloadError> for ApiError {
    fn from(err: JsonPayloadError) -> Self {
        ApiError::ValidationError(err.to_string())
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError::ValidationError(err)
    }
}
