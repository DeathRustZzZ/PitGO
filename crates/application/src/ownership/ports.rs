use crate::error::RepositoryError;
use domain::vehicle_ownership::aggregate::VehicleOwnership;
use domain::{VehicleId, VehicleOwnershipId};

pub trait VehicleOwnershipRepository: Send + Sync {
    /// Checks if there is an active ownership for the given vehicle ID
    fn has_active_ownership(&self, vehicle_id: VehicleId) -> Result<bool, RepositoryError>;

    /// Saves the given VehicleOwnership entity to the repository
    fn save(&self, ownership: &VehicleOwnership) -> Result<(), RepositoryError>;

    /// Finds a VehicleOwnership entity by its ownership ID
    fn find_by_id(
        &self,
        ownership_id: VehicleOwnershipId,
    ) -> Result<Option<VehicleOwnership>, RepositoryError>;
}
