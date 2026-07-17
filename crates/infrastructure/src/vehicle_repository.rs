use application::error::RepositoryError;
use application::vehicle::ports::VehicleRepository;
use domain::VehicleId;
use domain::vehicle::Vehicle;
use std::collections::HashMap;
use std::sync::Mutex;

/// An in-memory implementation of the VehicleRepository trait for testing purposes.
pub struct InMemoryVehicleRepository {
    vehicle: Mutex<HashMap<VehicleId, Vehicle>>,
}

/// Creates a new instance of InMemoryVehicleRepository with an empty vehicle storage.
impl InMemoryVehicleRepository {
    pub fn new() -> Self {
        Self {
            vehicle: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl VehicleRepository for InMemoryVehicleRepository {
    async fn save(&self, vehicle: &Vehicle) -> Result<(), application::error::RepositoryError> {
        let mut vehicles = self
            .vehicle
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let actual = vehicle.version();
        let expected = vehicles
            .get(&vehicle.id())
            .map(|stored| stored.version().next());
        if let Some(expected_version) = expected
            && expected_version != actual
        {
            return Err(RepositoryError::VersionConflict {
                expected: expected_version.value(),
                actual: actual.value(),
            });
        }
        vehicles.insert(vehicle.id(), vehicle.clone());

        Ok(())
    }

    async fn find_by_id(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, RepositoryError> {
        let vehicles = self
            .vehicle
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;
        Ok(vehicles.get(&vehicle_id).cloned())
    }
}

impl Default for InMemoryVehicleRepository {
    fn default() -> Self {
        Self::new()
    }
}
