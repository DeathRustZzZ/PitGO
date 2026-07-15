//! HTTP routes for vehicle operations.
//!
//! This module translates vehicle API requests into application-layer commands
//! and returns transport-level responses.

use crate::AppState;
use crate::error::ApiError;
use application::vehicle::commands::CreateVehicleCommand;
use application::vehicle::handlers::{CreateVehicleHandler, GetVehicleHandler};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
/// Request body for creating a new vehicle.
#[derive(Deserialize)]
pub struct CreateVehicleRequest {
    /// Client-provided vehicle identifier.
    pub vehicle_id: Uuid,
}

/// Response body for a vehicle.
#[derive(Serialize)]
pub struct VehicleResponse {
    pub vehicle_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Handles `POST /vehicles`.
///
/// Builds a `CreateVehicleCommand`, delegates business behavior to the
/// application layer, and maps any application error into `ApiError`.
pub async fn create_vehicle(
    State(state): State<AppState>,
    Json(body): Json<CreateVehicleRequest>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    let cmd = CreateVehicleCommand {
        vehicle_id: body.vehicle_id.into(),
    };

    let repository = state.vehicle_repository;

    let handler = CreateVehicleHandler::new(repository);

    handler.handle(cmd)?;
    Ok((
        StatusCode::CREATED,
        Json("Vehicle created successfully".to_string()),
    ))
}

pub async fn get_vehicle(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleResponse>, ApiError> {
    let handler = GetVehicleHandler::new(state.vehicle_repository);

    match handler.handle(id.into())? {
        Some(vehicle) => Ok(Json(VehicleResponse {
            vehicle_id: vehicle.id().into(),
            status: vehicle.status().kind().to_string(),
            created_at: vehicle.created_at(),
            updated_at: vehicle.updated_at(),
        })),
        None => Err(ApiError::not_found("Vehicle not found")),
    }
}
