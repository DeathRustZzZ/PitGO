use crate::error::RepositoryError;
use domain::VehicleId;
use domain::vehicle::aggregate::Vehicle;

/// Defines the VehicleRepository trait for managing Vehicle entities in the repository
pub trait VehicleRepository: Send + Sync {
    /// Saves the given Vehicle entity to the repository
    fn save(&self, vehicle: &Vehicle) -> Result<(), RepositoryError>;

    /// Finds a Vehicle entity by its vehicle ID
    fn find_by_id(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, RepositoryError>;
}
