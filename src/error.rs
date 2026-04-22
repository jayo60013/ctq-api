use crate::models::ProblemDetails;
use actix_web::{
    error::{JsonPayloadError, ResponseError},
    http::StatusCode,
    HttpResponse,
};
use std::fmt;

impl ProblemDetails {
    /// Constructs a `ProblemDetails` response from an `ApiError`.
    /// This is the internal conversion from domain error to HTTP response.
    fn from_api_error(error: &ApiError) -> Self {
        match error {
            ApiError::ValidationError(msg) => ProblemDetails {
                title: "Validation failed".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
                detail: msg.clone(),
                instance: None,
            },
            ApiError::DatabaseError(_msg) => ProblemDetails {
                title: "Internal server error".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: "An unexpected error occurred.".to_string(),
                instance: None,
            },
            ApiError::PuzzleNotGenerated => ProblemDetails {
                title: "Puzzle not ready".to_string(),
                status: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                detail: "Please wait while puzzle is being generated.".to_string(),
                instance: None,
            },
            ApiError::NotFound => ProblemDetails {
                title: "Invalid request".to_string(),
                status: StatusCode::NOT_FOUND.as_u16(),
                detail: "Resource not found".to_string(),
                instance: None,
            },
            ApiError::JwtError(_) => ProblemDetails {
                title: "Unauthorized".to_string(),
                status: StatusCode::UNAUTHORIZED.as_u16(),
                detail: "Invalid or expired token".to_string(),
                instance: None,
            },
            ApiError::ExternalServiceError(_) => ProblemDetails {
                title: "Internal server error".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                detail: "An unexpected error occurred.".to_string(),
                instance: None,
            },
            ApiError::Unauthorized => ProblemDetails {
                title: "Unauthorized".to_string(),
                status: StatusCode::UNAUTHORIZED.as_u16(),
                detail: "Authentication required".to_string(),
                instance: None,
            },
        }
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

/// Single unified error type for the entire API.
///
/// This replaces string-based error handling with type-safe variants.
/// Each error type maps to a specific HTTP status code and error response.
#[derive(Debug)]
pub enum ApiError {
    /// Validation failed, e.g., invalid input
    ValidationError(String),

    /// Database operation failed (excludes puzzle not generated)
    DatabaseError(String),

    /// Puzzle not yet generated (service unavailable)
    PuzzleNotGenerated,

    /// Resource not found (404)
    NotFound,

    /// JWT validation failed
    JwtError(String),

    /// External service call failed
    ExternalServiceError(String),

    /// User is not authenticated
    Unauthorized,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            ApiError::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            ApiError::PuzzleNotGenerated => write!(f, "Puzzle not generated yet"),
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
            ApiError::DatabaseError(_) | ApiError::ExternalServiceError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::PuzzleNotGenerated => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::JwtError(_) | ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        // Log errors at appropriate levels
        match self {
            ApiError::DatabaseError(msg) => tracing::error!("Database error: {msg}"),
            ApiError::JwtError(msg) => tracing::warn!("JWT error: {msg}"),
            ApiError::ExternalServiceError(msg) => tracing::error!("External service error: {msg}"),
            _ => {}
        }

        // Convert to HTTP response via ProblemDetails
        let details = ProblemDetails::from_api_error(self);
        HttpResponse::build(self.status_code()).json(details)
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

/// Converts validator crate `ValidationErrors` to `ApiError`
impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApiError::ValidationError(format!("{err:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_puzzle_not_generated_status_code() {
        let error = ApiError::PuzzleNotGenerated;
        assert_eq!(error.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_puzzle_not_generated_display() {
        let error = ApiError::PuzzleNotGenerated;
        assert_eq!(error.to_string(), "Puzzle not generated yet");
    }

    #[test]
    fn test_puzzle_not_generated_converts_to_problem_details() {
        let error = ApiError::PuzzleNotGenerated;
        let details = ProblemDetails::from_api_error(&error);
        assert_eq!(details.title, "Puzzle not ready");
        assert_eq!(details.status, StatusCode::SERVICE_UNAVAILABLE.as_u16());
        assert!(details.detail.contains("Please wait"));
    }

    #[test]
    fn test_validation_error_status_code() {
        let error = ApiError::ValidationError("Invalid input".to_string());
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_status_code() {
        let error = ApiError::NotFound;
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_unauthorized_status_code() {
        let error = ApiError::Unauthorized;
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_jwt_error_status_code() {
        let error = ApiError::JwtError("Invalid token".to_string());
        assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_database_error_status_code() {
        let error = ApiError::DatabaseError("Connection failed".to_string());
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_external_service_error_status_code() {
        let error = ApiError::ExternalServiceError("Service unavailable".to_string());
        assert_eq!(error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
