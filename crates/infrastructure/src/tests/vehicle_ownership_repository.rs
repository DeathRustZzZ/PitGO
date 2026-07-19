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

    /// A repeat `save` of the same `VehicleOwnershipId` (a duplicate create)
    /// must be rejected as `AlreadyExists`, not as a version conflict.
    ///
    /// Повторный `save` того же `VehicleOwnershipId` (дубликат `start`) должен
    /// возвращать `AlreadyExists`, а не конфликт версий.
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
        assert_eq!(result, Err(RepositoryError::AlreadyExists));
    }

    /// Saving the same freshly-created aggregate a second time (same object,
    /// no intervening command) must also return `AlreadyExists`.
    ///
    /// Повторный `save` того же только что созданного агрегата (тот же объект,
    /// без промежуточных команд) должен также возвращать `AlreadyExists`.
    #[tokio::test]
    async fn save_same_freshly_created_ownership_twice_returns_already_exists() {
        let repository = InMemoryVehicleOwnershipRepository::new();
        let now = Utc::now();
        let vehicle_id = VehicleId::new();

        let ownership = VehicleOwnership::start(
            VehicleOwnershipId::new(),
            vehicle_id,
            CustomerId::new(),
            OwnershipType::Private,
            OwnershipEligibilitySnapshot::new(vehicle_id, false),
            now,
        )
        .expect("ownership should be valid");

        repository
            .save(&ownership)
            .await
            .expect("first save should succeed");

        let result = repository.save(&ownership).await;
        assert_eq!(result, Err(RepositoryError::AlreadyExists));
    }

    /// A stale update — saving a version that skipped an intermediate
    /// persistence step — must be rejected as `VersionConflict`.
    ///
    /// Scenario: start → v1, save (stored v1), verify → v2, end → v3, then
    /// try to save v3 without having saved v2. The store holds v1 and expects
    /// v2, but receives v3, which is a genuine stale write.
    ///
    /// Устаревшее обновление — сохранение версии, пропустившей промежуточный
    /// шаг персистентности, — должно возвращать `VersionConflict`.
    ///
    /// Сценарий: start → v1, save (в хранилище v1), verify → v2, end → v3,
    /// затем попытаться сохранить v3, не сохранив v2. Хранилище держит v1 и
    /// ждёт v2, но получает v3 — это настоящая устаревшая запись.
    #[tokio::test]
    async fn rejects_stale_ownership_update() {
        let repository = InMemoryVehicleOwnershipRepository::new();
        let vehicle_id = VehicleId::new();
        let started_at = Utc::now();
        let verified_at = started_at + chrono::Duration::minutes(1);
        let ended_at = verified_at + chrono::Duration::minutes(1);

        let mut ownership = VehicleOwnership::start(
            VehicleOwnershipId::new(),
            vehicle_id,
            CustomerId::new(),
            OwnershipType::Private,
            OwnershipEligibilitySnapshot::new(vehicle_id, false),
            started_at,
        )
        .expect("ownership should be valid");

        repository
            .save(&ownership)
            .await
            .expect("first save (v1) should succeed");

        // Apply two commands without saving — ownership advances to v3
        ownership
            .verify(verified_at)
            .expect("verify should succeed");
        ownership.end(ended_at).expect("end should succeed");

        // stored=v1, expected=v2, actual=v3
        let result = repository.save(&ownership).await;
        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 2,
                actual: 3
            })
        );
    }

    /// An ownership in `PendingVerification` — right after `start`, before
    /// `verify` — already counts as "open" and must block a second ownership on
    /// the same vehicle. This is the invariant hole that was closed.
    ///
    /// Владение в `PendingVerification` (сразу после `start`, до `verify`) уже
    /// считается «открытым» и должно блокировать создание второго владения на
    /// ту же машину — это и есть закрытая дыра инварианта.
    #[tokio::test]
    async fn has_open_ownership_true_when_pending_ownership_exists() {
        // Arrange
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

        // Act
        let has_open_ownership = repository
            .has_open_ownership(vehicle_id)
            .await
            .expect("repository check should succeed");

        // Assert
        assert!(has_open_ownership);
    }

    /// A terminated ownership (`Ended`, after `verify` → `end`) does not occupy
    /// the vehicle — the filter must not confuse "once was" with "open now".
    ///
    /// Завершённое владение (`Ended`, после `verify` → `end`) не занимает
    /// машину — фильтр не должен путать «было» с «открыто сейчас».
    #[tokio::test]
    async fn has_open_ownership_false_when_ownership_ended() {
        // Arrange
        let repository = InMemoryVehicleOwnershipRepository::new();
        let ownership_id = VehicleOwnershipId::new();
        let vehicle_id = VehicleId::new();
        let owner_customer_id = CustomerId::new();
        let started_at = Utc::now();
        let verified_at = started_at + chrono::Duration::minutes(1);
        let ended_at = verified_at + chrono::Duration::minutes(1);

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
            .verify(verified_at)
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

    /// An open ownership on one vehicle must not leak into the check for a
    /// different vehicle — the filter has to consider `vehicle_id`, not status
    /// alone.
    ///
    /// Открытое владение на одну машину не должно «протекать» на проверку
    /// другой машины — фильтр обязан учитывать `vehicle_id`, а не только статус.
    #[tokio::test]
    async fn has_open_ownership_false_for_different_vehicle() {
        // Arrange
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

        // Act
        let has_open_ownership = repository
            .has_open_ownership(vehicle_b_id)
            .await
            .expect("repository check should succeed");

        // Assert
        assert!(!has_open_ownership);
    }

    /// End-to-end scenario through `StartVehicleOwnershipHandler` plus the real
    /// `InMemoryVehicleOwnershipRepository` (not a mock): a second `start` on
    /// the same vehicle, while the first ownership is still
    /// `PendingVerification`, must be rejected with a domain error.
    ///
    /// Regression test for the original bug in task 001. A mock could not catch
    /// it — the defect was in how the real repository classified a pending
    /// record, so only the genuine adapter reproduces it.
    ///
    /// Сквозной сценарий через `StartVehicleOwnershipHandler` и настоящий
    /// `InMemoryVehicleOwnershipRepository` (не мок): второй `start` на ту же
    /// машину, пока первое владение ещё `PendingVerification`, должен быть
    /// отклонён доменной ошибкой.
    ///
    /// Тест на регресс исходного бага задачи 001. Мок не смог бы его поймать:
    /// дефект был в том, как настоящий репозиторий классифицировал ожидающую
    /// запись, поэтому воспроизводит его только подлинный адаптер.
    #[tokio::test]
    async fn second_start_on_same_vehicle_is_rejected_by_real_repository() {
        // Arrange
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
            .expect("first ownership should start successfully");

        let second_command = StartVehicleOwnershipCommand {
            ownership_id: VehicleOwnershipId::new(),
            vehicle_id,
            owner_customer_id: CustomerId::new(),
            ownership_type: OwnershipType::Private,
        };

        // Act
        let error = handler
            .handle(second_command)
            .await
            .expect_err("second active ownership for the same vehicle must be rejected");

        // Assert
        assert!(matches!(
            error,
            ApplicationError::Ownership(OwnershipError::ActiveOwnershipAlreadyExists)
        ));
    }
}
