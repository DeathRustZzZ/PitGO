#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::vehicle_ownership_repository::InMemoryVehicleOwnershipRepository;
    use application::error::{ApplicationError, RepositoryError};
    use application::ownership::commands::StartVehicleOwnershipCommand;
    use application::ownership::handlers::StartVehicleOwnershipHandler;
    use application::ownership::ports::VehicleOwnershipRepository;
    use chrono::Utc;
    use domain::vehicle_ownership::{
        OwnershipEligibilitySnapshot, OwnershipError, OwnershipType, VehicleOwnership,
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

        assert!(has_open_ownership);
    }

    #[tokio::test]
    async fn has_open_ownership_false_when_ownership_ended() {
        let repository = InMemoryVehicleOwnershipRepository::new();
        let ownership_id = VehicleOwnershipId::new();
        let vehicle_id = VehicleId::new();
        let owner_customer_id = CustomerId::new();
        let started_at = Utc::now();
        let varified_at = started_at + chrono::Duration::minutes(1);
        let ended_at = varified_at + chrono::Duration::minutes(1);

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
            .verify(varified_at)
            .expect("ownership verification should succeed");

        ownership
            .end(ended_at)
            .expect("ownership end should succeed");

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

        assert!(!has_open_ownership);
    }

    // Test to ensure that the second start of ownership on the same vehicle is rejected by the real repository
    #[tokio::test()]
    async fn second_start_on_same_vehicle_is_rejected_by_real_repository() {
        let repository = Arc::new(InMemoryVehicleOwnershipRepository::new());

        let handler = StartVehicleOwnershipHandler::new(
            Arc::clone(&repository) as Arc<dyn VehicleOwnershipRepository>
        );

        let vehicle_id = VehicleId::new();

        let first_command = StartVehicleOwnershipCommand {
            ownership_id: VehicleOwnershipId::new(),
            vehicle_id,
            owner_customer_id: CustomerId::new(),
            ownership_type: OwnershipType::Private,
        };

        handler
            .handle(first_command)
            .await
            .expect("firs ownership should start successfully");

        let seccond_command = StartVehicleOwnershipCommand {
            ownership_id: VehicleOwnershipId::new(),
            vehicle_id,
            owner_customer_id: CustomerId::new(),
            ownership_type: OwnershipType::Private,
        };

        let error = handler
            .handle(seccond_command)
            .await
            .expect_err("second active ownership for the same vehicle must be rejected");

        assert!(matches!(
            error,
            ApplicationError::Ownership(OwnershipError::ActiveOwnershipAlreadyExists)
        ));
    }
}
