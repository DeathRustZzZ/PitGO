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

impl VehicleRepository for InMemoryVehicleRepository {
    fn save(&self, vehicle: &Vehicle) -> Result<(), application::error::RepositoryError> {
        let mut vehicles = self
            .vehicle
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;
        vehicles.insert(vehicle.id(), vehicle.clone());

        Ok(())
    }

    fn find_by_id(&self, vehicle_id: VehicleId) -> Result<Option<Vehicle>, RepositoryError> {
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
