#[cfg(test)]
mod tests {
    use crate::vehicle_repository::InMemoryVehicleRepository;
    use application::{error::RepositoryError, vehicle::ports::VehicleRepository};
    use chrono::{Duration, Utc};
    use domain::{
        VehicleId,
        vehicle::{Vehicle, VehicleActivationPermit},
    };

    /// Creating the same vehicle twice must be rejected as `AlreadyExists`.
    /// Both creates arrive at version 1; the repository must distinguish
    /// "duplicate create" from "stale update".
    ///
    /// Повторное создание того же автомобиля должно возвращать `AlreadyExists`.
    /// Оба создания приходят с версией 1; репозиторий должен отличать
    /// «повторное создание» от «устаревшего обновления».
    #[tokio::test]
    async fn rejects_duplicate_vehicle_create() {
        let repository = InMemoryVehicleRepository::new();
        let vehicle_id = VehicleId::new();

        let first_vehicle = Vehicle::create(vehicle_id, Utc::now());
        let duplicate_vehicle = Vehicle::create(vehicle_id, Utc::now());

        repository
            .save(&first_vehicle)
            .await
            .expect("First save should succeed");

        let result = repository.save(&duplicate_vehicle).await;
        assert_eq!(result, Err(RepositoryError::AlreadyExists));
    }

    /// Saving the same freshly-created aggregate a second time (same object,
    /// no intervening command) must also return `AlreadyExists`.  The
    /// aggregate is still at version 1, so the repository treats it as
    /// another create attempt.
    ///
    /// Повторный `save` того же только что созданного агрегата (тот же объект,
    /// без промежуточных команд) должен также возвращать `AlreadyExists`.
    #[tokio::test]
    async fn save_same_freshly_created_vehicle_twice_returns_already_exists() {
        let repository = InMemoryVehicleRepository::new();
        let vehicle = Vehicle::create(VehicleId::new(), Utc::now());

        repository
            .save(&vehicle)
            .await
            .expect("first save should succeed");

        let result = repository.save(&vehicle).await;
        assert_eq!(result, Err(RepositoryError::AlreadyExists));
    }

    /// A stale update — saving an already-persisted version again — must be
    /// rejected as `VersionConflict`, not as `AlreadyExists`.
    ///
    /// Scenario: save v1, activate → v2, save v2 (stored is now v2), then
    /// try to save v2 again. The store expects v3 but receives v2.
    ///
    /// Устаревшее обновление — повторное сохранение уже сохранённой версии —
    /// должно возвращать `VersionConflict`, а не `AlreadyExists`.
    #[tokio::test]
    async fn rejects_stale_vehicle_update() {
        let repository = InMemoryVehicleRepository::new();
        let now = Utc::now();
        let id = VehicleId::new();

        let mut vehicle = Vehicle::create(id, now);
        repository
            .save(&vehicle)
            .await
            .expect("first save should succeed");

        let permit = VehicleActivationPermit::new(id, now, now + Duration::minutes(5));
        vehicle
            .activate(permit, now)
            .expect("activation should succeed");
        repository
            .save(&vehicle)
            .await
            .expect("second save (post-activate) should succeed");

        // vehicle is still at v2; stored now expects v3
        let result = repository.save(&vehicle).await;
        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 3,
                actual: 2
            })
        );
    }
}
