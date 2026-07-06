//! HTTP representation of backend API errors.
//!
//! This module separates internal application-layer errors from the public REST
//! API contract: responses expose only an HTTP status, a machine-readable error
//! code, and a safe user-facing message.

use application::error::{ApplicationError, RepositoryError};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorBody {
    /// Stable machine-readable error code for API clients.
    error: String,
    /// Safe message that does not expose internal storage details.
    message: String,
}

/// Error type that axum can convert directly into an HTTP response.
///
/// Used as the common error type in handlers. Because `From<ApplicationError>`
/// is implemented, handlers can return `Result<_, ApiError>` and use the `?`
/// operator on application-layer calls.
pub struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

impl ApiError {
    /// Creates a `500 Internal Server Error` response.
    ///
    /// Used for infrastructure failures whose details must not leak into the
    /// public API response.
    pub fn internal_server_error(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorBody {
                error: "internal_server_error".to_string(),
                message: message.to_string(),
            },
        }
    }

    /// Creates a `409 Conflict` response.
    ///
    /// Suitable for optimistic-locking conflicts and concurrent changes to the
    /// same entity.
    pub fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorBody {
                error: "conflict".to_string(),
                message: message.to_string(),
            },
        }
    }

    /// Creates a `422 Unprocessable Entity` response.
    ///
    /// Used when the request is syntactically valid but violates application or
    /// domain business rules.
    pub fn unprocessable_entity(message: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ErrorBody {
                error: "unprocessable_entity".to_string(),
                message: message.to_string(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    /// Converts the error into a JSON response shaped as
    /// `{ "error": "...", "message": "..." }`.
    fn into_response(self) -> Response {
        let body = Json(self.body);
        (self.status, body).into_response()
    }
}

impl From<ApplicationError> for ApiError {
    /// Maps application-layer errors to public HTTP errors.
    ///
    /// Domain errors remain generalized: specific internal variants do not leak
    /// into the API contract until they have stable public error codes.
    fn from(e: ApplicationError) -> Self {
        match e {
            ApplicationError::Repository(repo_err) => match repo_err {
                RepositoryError::VersionConflict { .. } => {
                    ApiError::conflict("Conflict occurred while processing the request.")
                }
                RepositoryError::StorageFailure(_) => ApiError::internal_server_error(
                    "Internal server error occurred while accessing the repository.",
                ),
            },
            ApplicationError::Ownership(_) => ApiError::unprocessable_entity(
                "Ownership error occurred while processing the request.",
            ),
        }
    }
}
