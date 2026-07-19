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

    #[tokio::test]
    async fn rejects_duplicate_vehicle_ownership_start() {
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
            .await
            .expect("first save should succeed");

        // Act
        let result = repository.save(&duplicate_ownership).await;

        // Assert
        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 2,
                actual: 1,
            })
        );
    }

    #[tokio::test]
    async fn has_open_ownership_true_when_pending_ownership_exists() {
        // arrange: создать ownership через VehicleOwnership::start
        // save
        // act: has_open_ownership(vehicle_id)
        // assert: true
        //
        let repository = InMemoryVehicleOwnershipRepository::new();

        let ownership_id = VehicleOwnershipId::new();
        let vehicle_id = VehicleId::new();
        let owner_customer_id = CustomerId::new();
        let now = Utc::now();
        let ownership = VehicleOwnership::start(
            ownership_id,
            vehicle_id,
            owner_customer_id,
            OwnershipType::Private,
            OwnershipEligibilitySnapshot::new(vehicle_id, false),
            now,
        )
        .expect("pending ownership should be valid");

        repository
            .save(&ownership)
            .await
            .expect("ownership save should succeed");

        let has_open_ownership = repository
            .has_open_ownership(vehicle_id)
            .await
            .expect("repository check should succeed");

        assert!(has_open_ownership)
    }

    #[tokio::test]
    async fn has_open_ownership_false_when_ownership_ended() {
        // arrange: start → verify → end → save
        // act
        // assert: false
        let repository = InMemoryVehicleOwnershipRepository::new();
        let ownership_id = VehicleOwnershipId::new();
        let vehicle_id = VehicleId::new();
        let owner_customer_id = CustomerId::new();
        let started_at = Utc::now();
        let varifyed_at = started_at + chrono::Duration::minutes(1);
        let ended_at = varifyed_at + chrono::Duration::minutes(1);

        let mut ownership = VehicleOwnership::start(
            ownership_id,
            vehicle_id,
            owner_customer_id,
            OwnershipType::Private,
            OwnershipEligibilitySnapshot::new(vehicle_id, false),
            started_at,
        )
        .expect("ownership should be valid");

        ownership
            .verify(varifyed_at)
            .expect("ownership verification should succeed");

        ownership.end(ended_at).expect("ownership end should");

        repository
            .save(&ownership)
            .await
            .expect("ownership save should succeed");

        // Act
        let has_open_ownership = repository
            .has_open_ownership(vehicle_id)
            .await
            .expect("repository check should succeed");

        // Assert
        assert!(!has_open_ownership);
    }

    #[tokio::test]
    async fn has_open_ownership_false_for_different_vehicle() {
        // arrange: ownership для vehicle_a
        // act: проверить vehicle_b
        // assert: false

        let repository = InMemoryVehicleOwnershipRepository::new();
        let ownership_id = VehicleOwnershipId::new();
        let vehicle_a_id = VehicleId::new();
        let vehicle_b_id = VehicleId::new();
        let owner_customer_id = CustomerId::new();
        let now = Utc::now();

        let ownership = VehicleOwnership::start(
            ownership_id,
            vehicle_a_id,
            owner_customer_id,
            OwnershipType::Private,
            OwnershipEligibilitySnapshot::new(vehicle_a_id, false),
            now,
        )
        .expect("ownership should be valid");

        repository
            .save(&ownership)
            .await
            .expect("ownership save should succeed");

        let has_open_ownership = repository
            .has_open_ownership(vehicle_b_id)
            .await
            .expect("repository check should succeed");

        assert!(!has_open_ownership)
    }
}
