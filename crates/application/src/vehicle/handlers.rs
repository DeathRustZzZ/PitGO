use crate::error::ApplicationError;
use crate::vehicle::commands::CreateVehicleCommand;
use crate::vehicle::ports::VehicleRepository;
use chrono::Utc;
use domain::VehicleId;
use domain::vehicle::aggregate::Vehicle;
use std::sync::Arc;

/// Handler for creating a new vehicle
pub struct CreateVehicleHandler {
    repository: Arc<dyn VehicleRepository>,
}

/// Implementation of the CreateVehicleHandler
impl CreateVehicleHandler {
    /// Create a new instance of the handler
    pub fn new(repository: Arc<dyn VehicleRepository>) -> Self {
        Self { repository }
    }

    /// Handle the command to create a new vehicle
    pub async fn handle(&self, cmd: CreateVehicleCommand) -> Result<(), ApplicationError> {
        let now = Utc::now();
        let vehicle = Vehicle::create(cmd.vehicle_id, now);
        self.repository.save(&vehicle).await?;
        Ok(())
    }
}

/// Handler for searching for a vehicle by ID
pub struct GetVehicleHandler {
    repository: Arc<dyn VehicleRepository>,
}

impl GetVehicleHandler {
    pub fn new(repository: Arc<dyn VehicleRepository>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, ApplicationError> {
        Ok(self.repository.find_by_id(vehicle_id).await?)
    }
}
