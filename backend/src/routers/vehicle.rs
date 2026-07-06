//! HTTP routes for vehicle operations.
//!
//! This module translates vehicle API requests into application-layer commands
//! and returns transport-level responses.

use crate::AppState;
use crate::error::ApiError;
use application::vehicle::commands::CreateVehicleCommand;
use application::vehicle::handlers::CreateVehicleHandler;
use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use uuid::Uuid;

/// Request body for creating a new vehicle.
#[derive(Deserialize)]
pub struct CreateVehicleRequest {
    /// Client-provided vehicle identifier.
    pub vehicle_id: Uuid,
}

/// Handles `POST /vehicles`.
///
/// Builds a `CreateVehicleCommand`, delegates business behavior to the
/// application layer, and maps any application error into `ApiError`.
pub async fn create_vehicle(
    State(state): State<AppState>,
    Json(body): Json<CreateVehicleRequest>,
) -> Result<Json<String>, ApiError> {
    let cmd = CreateVehicleCommand {
        vehicle_id: body.vehicle_id.into(),
    };

    let repository = state.vehicle_repository;

    let handler = CreateVehicleHandler::new(repository);

    handler.handle(cmd)?;
    Ok(Json("Vehicle created successfully".to_string()))
}
