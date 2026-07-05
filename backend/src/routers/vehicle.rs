use crate::AppState;
use application::error::{ApplicationError, RepositoryError};
use application::vehicle::commands::CreateVehicleCommand;
use application::vehicle::handlers::CreateVehicleHandler;
use axum::extract::State;
use axum::{Json, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateVehicleRequest {
    pub vehicle_id: Uuid,
}

pub async fn create_vehicle(
    State(state): State<AppState>,
    Json(body): Json<CreateVehicleRequest>,
) -> StatusCode {
    let cmd = CreateVehicleCommand {
        vehicle_id: body.vehicle_id.into(),
    };

    let repository = state.vehicle_repository;

    let handler = CreateVehicleHandler::new(repository);

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
