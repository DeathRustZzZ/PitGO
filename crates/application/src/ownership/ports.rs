use crate::error::RepositoryError;
use domain::vehicle_ownership::aggregate::VehicleOwnership;
use domain::{VehicleId, VehicleOwnershipId};

#[async_trait::async_trait]
pub trait VehicleOwnershipRepository: Send + Sync {
    /// Checks whether the given vehicle currently has an open ownership record
    /// (`PendingVerification` or `Active`). An open record occupies the vehicle
    /// and blocks starting a new one.
    async fn has_open_ownership(&self, vehicle_id: VehicleId) -> Result<bool, RepositoryError>;

    /// Saves the given VehicleOwnership entity to the repository
    async fn save(&self, ownership: &VehicleOwnership) -> Result<(), RepositoryError>;

    /// Finds a VehicleOwnership entity by its ownership ID
    async fn find_by_id(
        &self,
        ownership_id: VehicleOwnershipId,
    ) -> Result<Option<VehicleOwnership>, RepositoryError>;
}
