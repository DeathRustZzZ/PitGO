use domain::VehicleId;

// Command to create a new vehicle
pub struct CreateVehicleCommand {
    pub vehicle_id: VehicleId,
}
