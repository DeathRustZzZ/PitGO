use crate::error::ApplicationError;
use crate::ownership::commands::StartVehicleOwnershipCommand;
use crate::ownership::ports::VehicleOwnershipRepository;
use chrono::Utc;
use domain::vehicle_ownership::aggregate::VehicleOwnership;
use domain::vehicle_ownership::snapshot::OwnershipEligibilitySnapshot;
use std::sync::Arc;

/// Handler for starting vehicle ownership
pub struct StartVehicleOwnershipHandler {
    repository: Arc<dyn VehicleOwnershipRepository>,
}

/// Implementation of the StartVehicleOwnershipHandler
impl StartVehicleOwnershipHandler {
    pub fn new(repository: Arc<dyn VehicleOwnershipRepository>) -> Self {
        Self { repository }
    }

    pub fn handle(&self, cmd: StartVehicleOwnershipCommand) -> Result<(), ApplicationError> {
        let now = Utc::now();
        let has_active_ownership = self.repository.has_active_ownership(cmd.vehicle_id)?;
        let snapshot = OwnershipEligibilitySnapshot::new(cmd.vehicle_id, has_active_ownership);
        let ownership = VehicleOwnership::start(
            cmd.ownership_id,
            cmd.vehicle_id,
            cmd.owner_customer_id,
            cmd.ownership_type,
            snapshot,
            now,
        )?;

        self.repository.save(&ownership)?;

        Ok(())
    }
}
