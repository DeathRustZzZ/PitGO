#[cfg(test)]
mod tests {
    use crate::vehicle_repository::InMemoryVehicleRepository;
    use application::{error::RepositoryError, vehicle::ports::VehicleRepository};
    use chrono::Utc;
    use domain::{VehicleId, vehicle::Vehicle};

    #[test]
    fn rejects_duplicate_vehicle_create() {
        let repository = InMemoryVehicleRepository::new();
        let vehicle_id = VehicleId::new();

        let first_vehicle = Vehicle::create(vehicle_id, Utc::now());
        let duplicate_vehicle = Vehicle::create(vehicle_id, Utc::now());

        repository
            .save(&first_vehicle)
            .expect("First save should succeed");

        let result = repository.save(&duplicate_vehicle);

        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 2,
                actual: 1
            })
        )
    }
}
