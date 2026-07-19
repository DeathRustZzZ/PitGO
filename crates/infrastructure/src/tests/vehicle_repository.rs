#[cfg(test)]
mod tests {
    use crate::vehicle_repository::InMemoryVehicleRepository;
    use application::{error::RepositoryError, vehicle::ports::VehicleRepository};
    use chrono::Utc;
    use domain::{VehicleId, vehicle::Vehicle};

    /// Creating the same vehicle twice must be rejected by the
    /// optimistic-locking check rather than silently overwriting the first
    /// record. The expected/actual pair asserts the exact arithmetic: the
    /// second aggregate arrives at version 1 while the store already holds
    /// version 1 and therefore requires 2.
    ///
    /// Повторное создание того же автомобиля должно отклоняться проверкой
    /// оптимистичной блокировки, а не молча перезаписывать первую запись. Пара
    /// expected/actual фиксирует точную арифметику: второй агрегат приходит с
    /// версией 1, тогда как в хранилище уже лежит версия 1 и, значит, требуется 2.
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

        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 2,
                actual: 1
            })
        )
    }
}
