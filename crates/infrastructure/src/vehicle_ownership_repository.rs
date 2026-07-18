use std::collections::HashMap;
use std::sync::Mutex;

use application::error::RepositoryError;
use application::ownership::ports::VehicleOwnershipRepository;
use domain::vehicle_ownership::VehicleOwnership;
use domain::{VehicleOwnershipId, vehicle_ownership};

pub struct InMemoryVehicleOwnershipRepository {
    vehicle_ownership: Mutex<HashMap<VehicleOwnershipId, VehicleOwnership>>,
}

impl InMemoryVehicleOwnershipRepository {
    pub fn new() -> Self {
        Self {
            vehicle_ownership: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl VehicleOwnershipRepository for InMemoryVehicleOwnershipRepository {
    async fn save(&self, ownership: &VehicleOwnership) -> Result<(), RepositoryError> {
        let mut ownerships = self
            .vehicle_ownership
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let actual = ownership.version();
        let expected = ownerships
            .get(&ownership.id())
            .map(|stored| stored.version().next());

        if let Some(expected_version) = expected
            && expected_version != actual
        {
            return Err(RepositoryError::VersionConflict {
                expected: expected_version.value(),
                actual: actual.value(),
            });
        }
        ownerships.insert(ownership.id(), ownership.clone());
        Ok(())
    }

    /// Finds a vehicle ownership by its ID.
    /// Returns `Ok(Some(VehicleOwnership))` if found, `Ok(None)` if not found, or an error if there was a storage failure.
    async fn find_by_id(
        &self,
        ownership_id: VehicleOwnershipId,
    ) -> Result<Option<VehicleOwnership>, RepositoryError> {
        let ownership = self
            .vehicle_ownership
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;
        Ok(ownership.get(&ownership_id).cloned())
    }

    /// Checks if there is an active ownership for the given vehicle ID.
    /// Returns `true` if an active ownership exists, otherwise returns `false`.
    async fn has_open_ownership(
        &self,
        vehicle_id: domain::VehicleId,
    ) -> Result<bool, RepositoryError> {
        let ownerships = self
            .vehicle_ownership
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let has_active = ownerships.values().any(|ownership| {
            ownership.vehicle_id() == vehicle_id
                && *ownership.status() == vehicle_ownership::OwnershipStatus::Active
        });

        Ok(has_active)
    }
}

impl Default for InMemoryVehicleOwnershipRepository {
    fn default() -> Self {
        Self::new()
    }
}
