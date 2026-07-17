#[cfg(test)]
mod tests {
    use crate::customer_repository::InMemoryCustomerRepository;
    use application::{customer::ports::CustomerRepository, error::RepositoryError};
    use chrono::Utc;
    use domain::{CustomerId, customer::Customer};

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
