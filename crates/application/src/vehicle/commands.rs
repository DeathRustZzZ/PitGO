use domain::VehicleId;

/// Command to create a new vehicle
#[derive(Debug, Clone, Copy)]
pub struct CreateVehicleCommand {
    pub vehicle_id: VehicleId,
}
