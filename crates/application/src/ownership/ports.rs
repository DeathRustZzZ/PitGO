use crate::error::RepositoryError;
use domain::vehicle_ownership::aggregate::VehicleOwnership;
use domain::{VehicleId, VehicleOwnershipId};

pub trait VehicleOwnershipRepository {
    fn has_active_ownership(&self, vehicle_id: VehicleId) -> Result<bool, RepositoryError>;

    fn save(&self, ownership: &VehicleOwnership) -> Result<(), RepositoryError>;

    fn find_by_id(
        &self,
        ownership_id: VehicleOwnershipId,
    ) -> Result<Option<VehicleOwnership>, RepositoryError>;
}
