#[cfg(test)]
mod tests {
    use crate::vehicle_ownership_repository::InMemoryVehicleOwnershipRepository;
    use application::error::RepositoryError;
    use application::ownership::ports::VehicleOwnershipRepository;
    use chrono::Utc;
    use domain::vehicle_ownership::{
        OwnershipEligibilitySnapshot, OwnershipType, VehicleOwnership,
    };
    use domain::{CustomerId, VehicleId, VehicleOwnershipId};

    #[test]
    fn rejects_duplicate_vehicle_ownership_start() {
        // Arrange
        let repository = InMemoryVehicleOwnershipRepository::new();

        let ownership_id = VehicleOwnershipId::new();
        let vehicle_id = VehicleId::new();
        let owner_customer_id = CustomerId::new();
        let now = Utc::now();

        let first_ownership = VehicleOwnership::start(
            ownership_id,
            vehicle_id,
            owner_customer_id,
            OwnershipType::Private,
            OwnershipEligibilitySnapshot::new(vehicle_id, false),
            now,
        )
        .expect("first ownership should be valid");

        let duplicate_ownership = VehicleOwnership::start(
            ownership_id,
            vehicle_id,
            owner_customer_id,
            OwnershipType::Private,
            OwnershipEligibilitySnapshot::new(vehicle_id, false),
            now,
        )
        .expect("duplicate ownership aggregate should be created");

        repository
            .save(&first_ownership)
            .expect("first save should succeed");

        // Act
        let result = repository.save(&duplicate_ownership);

        // Assert
        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 2,
                actual: 1,
            })
        );
    }
}
