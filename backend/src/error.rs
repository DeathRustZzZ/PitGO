use application::error::{ApplicationError, RepositoryError};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

pub struct ApiError {
    status: StatusCode,
    body: ErrorBody,
}

impl ApiError {
    pub fn internal_server_error(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ErrorBody {
                error: "internal_server_error".to_string(),
                message: message.to_string(),
            },
        }
    }

    pub fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ErrorBody {
                error: "conflict".to_string(),
                message: message.to_string(),
            },
        }
    }

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
    fn into_response(self) -> Response {
        let body = Json(self.body);
        (self.status, body).into_response()
    }
}

impl From<ApplicationError> for ApiError {
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
