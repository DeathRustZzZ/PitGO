use crate::AppState;
use crate::error::ApiError;
use application::vehicle::commands::CreateVehicleCommand;
use application::vehicle::handlers::CreateVehicleHandler;
use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateVehicleRequest {
    pub vehicle_id: Uuid,
}

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
