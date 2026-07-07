use crate::AppState;
use crate::error::ApiError;
use application::ownership::commands::StartVehicleOwnershipCommand;
use application::ownership::handlers::StartVehicleOwnershipHandler;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use domain::vehicle_ownership::OwnershipType;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateVehicleOwnershipRequest {
    pub ownership_id: Uuid,
    pub vehicle_id: Uuid,
    pub owner_customer_id: Uuid,
    pub ownership_type: OwnershipTypeDto,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipTypeDto {
    Private,
    Company,
    Leasing,
    Fleet,
    Unknown,
}

impl OwnershipTypeDto {
    fn into_domain(self) -> OwnershipType {
        match self {
            Self::Private => OwnershipType::Private,
            Self::Company => OwnershipType::Company,
            Self::Leasing => OwnershipType::Leasing,
            Self::Fleet => OwnershipType::Fleet,
            Self::Unknown => OwnershipType::Unknown,
        }
    }
}

pub async fn create_vehicle_ownership(
    State(state): State<AppState>,
    Json(body): Json<CreateVehicleOwnershipRequest>,
) -> Result<(StatusCode, Json<String>), ApiError> {
    let cmd = StartVehicleOwnershipCommand {
        ownership_id: body.ownership_id.into(),
        vehicle_id: body.vehicle_id.into(),
        owner_customer_id: body.owner_customer_id.into(),
        ownership_type: body.ownership_type.into_domain(),
    };

    let repository = state.vehicle_ownership_repository;

    let handler = StartVehicleOwnershipHandler::new(repository);

    handler.handle(cmd)?;
    Ok((
        StatusCode::CREATED,
        Json("Vehicle ownership started successfully".to_string()),
    ))
}
