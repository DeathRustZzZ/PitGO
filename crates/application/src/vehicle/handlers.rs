use crate::error::ApplicationError;
use crate::vehicle::commands::CreateVehicleCommand;
use crate::vehicle::ports::VehicleRepository;
use chrono::Utc;
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
    pub fn handle(&self, cmd: CreateVehicleCommand) -> Result<(), ApplicationError> {
        let now = Utc::now();
        let vehicle = Vehicle::create(cmd.vehicle_id, now);
        self.repository.save(&vehicle)?;
        Ok(())
    }
}
