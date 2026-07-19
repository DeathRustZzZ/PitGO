#[cfg(test)]
mod tests {
    use crate::customer_repository::InMemoryCustomerRepository;
    use application::{customer::ports::CustomerRepository, error::RepositoryError};
    use chrono::Utc;
    use domain::{CustomerId, customer::Customer};

    /// Creating the same customer twice must be rejected by the
    /// optimistic-locking check rather than silently overwriting the first
    /// record. The expected/actual pair asserts the exact arithmetic: the
    /// second aggregate arrives at version 1 while the store already holds
    /// version 1 and therefore requires 2.
    ///
    /// Повторное создание того же клиента должно отклоняться проверкой
    /// оптимистичной блокировки, а не молча перезаписывать первую запись. Пара
    /// expected/actual фиксирует точную арифметику: второй агрегат приходит с
    /// версией 1, тогда как в хранилище уже лежит версия 1 и, значит, требуется 2.
    #[tokio::test]
    async fn rejects_duplicate_customer_create() {
        let repository = InMemoryCustomerRepository::new();
        let customer_id = CustomerId::new();

        let first_customer = Customer::create(customer_id, Utc::now());
        let duplicate_customer = Customer::create(customer_id, Utc::now());

        repository
            .save(&first_customer)
            .await
            .expect("First save should succeed");
        let result = repository.save(&duplicate_customer).await;
        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 2,
                actual: 1
            })
        )
    }
}
