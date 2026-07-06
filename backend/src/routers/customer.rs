//! HTTP routes for customer operations.
//!
//! This module translates customer API requests into application-layer commands
//! and returns transport-level responses.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::error::ApiError;
use application::customer::commands::CreateCustomerCommand;
use application::customer::handlers::CreateCustomerHandler;

/// Request body for creating a new customer.
#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    /// Client-provided customer identifier.
    pub customer_id: Uuid,
}

/// Handles `POST /customers`.
///
/// Builds a `CreateCustomerCommand`, delegates business behavior to the
/// application layer, and maps any application error into `ApiError`.
pub async fn create_customer(
    State(state): State<AppState>,
    Json(body): Json<CreateCustomerRequest>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    let cmd = CreateCustomerCommand {
        customer_id: body.customer_id.into(),
    };

    let repository = state.customer_repository;

    let handler = CreateCustomerHandler::new(repository);

    handler.handle(cmd)?;
    Ok((
        StatusCode::CREATED,
        Json("Customer created successfully".to_string()),
    ))
}
