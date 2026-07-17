//! HTTP routes for customer operations.
//!
//! This module translates customer API requests into application-layer commands
//! and returns transport-level responses.

use application::customer::commands::CreateCustomerCommand;
use application::customer::handlers::{CreateCustomerHandler, GetCustomerHandler};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::error::ApiError;

/// Request body for creating a new customer.
#[derive(Deserialize)]
pub struct CreateCustomerRequest {
    /// Client-provided customer identifier.
    pub customer_id: Uuid,
}

/// Response body for a customer.
#[derive(Serialize)]
pub struct CustomerResponse {
    pub customer_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

    handler.handle(cmd).await?;
    Ok((
        StatusCode::CREATED,
        Json("Customer created successfully".to_string()),
    ))
}

/// Handles `GET /customers/{id}`.
pub async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerResponse>, ApiError> {
    let handler = GetCustomerHandler::new(state.customer_repository);

    match handler.handle(id.into()).await? {
        Some(customer) => Ok(Json(CustomerResponse {
            customer_id: customer.id().into(),
            status: customer.status().kind().to_string(),
            created_at: customer.created_at(),
            updated_at: customer.updated_at(),
        })),
        None => Err(ApiError::not_found("Customer not found.")),
    }
}
