use application::error::{ApplicationError, RepositoryError};
use axum::extract::State;
use axum::{Json, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

use application::customer::commands::CreateCustomerCommand;
use application::customer::handlers::CreateCustomerHandler;

use crate::AppState;

/// Request body for creating a new customer
#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    pub customer_id: Uuid,
}

/// Handler for the POST /customers endpoint
pub async fn create_customer(
    State(state): State<AppState>,
    Json(body): Json<CreateCustomerRequest>,
) -> StatusCode {
    let cmd = CreateCustomerCommand {
        customer_id: body.customer_id.into(),
    };

    let repository = state.customer_repository;

    let handler = CreateCustomerHandler::new(repository);

    match handler.handle(cmd) {
        Ok(()) => StatusCode::CREATED,
        Err(e) => match e {
            ApplicationError::Repository(repo_err) => match repo_err {
                RepositoryError::VersionConflict { .. } => StatusCode::CONFLICT,
                RepositoryError::StorageFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ApplicationError::Ownership(_) => StatusCode::UNPROCESSABLE_ENTITY,
        },
    }
}
