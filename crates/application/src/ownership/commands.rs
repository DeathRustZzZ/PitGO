use domain::vehicle_ownership::state::OwnershipType;
use domain::{CustomerId, VehicleId, VehicleOwnershipId};

/// Command to start vehicle ownership
#[derive(Debug, Clone)]
pub struct StartVehicleOwnershipCommand {
    pub ownership_id: VehicleOwnershipId,
    pub vehicle_id: VehicleId,
    pub owner_customer_id: CustomerId,
    pub ownership_type: OwnershipType,
}
