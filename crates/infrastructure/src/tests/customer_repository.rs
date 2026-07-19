#[cfg(test)]
mod tests {
    use crate::customer_repository::InMemoryCustomerRepository;
    use application::{customer::ports::CustomerRepository, error::RepositoryError};
    use chrono::{Duration, Utc};
    use domain::{
        CustomerId,
        customer::{ActivationPermit, Customer},
    };

    /// Creating the same customer twice must be rejected as `AlreadyExists`.
    /// Both creates arrive at version 1; the repository must distinguish
    /// "duplicate create" from "stale update".
    ///
    /// Повторное создание того же клиента должно возвращать `AlreadyExists`.
    /// Оба создания приходят с версией 1; репозиторий должен отличать
    /// «повторное создание» от «устаревшего обновления».
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
        assert_eq!(result, Err(RepositoryError::AlreadyExists));
    }

    /// Saving the same freshly-created aggregate a second time (same object,
    /// no intervening command) must also return `AlreadyExists`.  The
    /// aggregate is still at version 1, so the repository treats it as
    /// another create attempt regardless of whether it is the same object or
    /// a separate one constructed with the same id.
    ///
    /// Повторный `save` того же только что созданного агрегата (тот же объект,
    /// без промежуточных команд) должен также возвращать `AlreadyExists`.
    /// Агрегат всё ещё имеет версию 1, поэтому репозиторий трактует его как
    /// ещё одну попытку создания.
    #[tokio::test]
    async fn save_same_freshly_created_customer_twice_returns_already_exists() {
        let repository = InMemoryCustomerRepository::new();
        let customer = Customer::create(CustomerId::new(), Utc::now());

        repository
            .save(&customer)
            .await
            .expect("first save should succeed");

        let result = repository.save(&customer).await;
        assert_eq!(result, Err(RepositoryError::AlreadyExists));
    }

    /// A stale update — saving an already-persisted version again — must be
    /// rejected as `VersionConflict`, not as `AlreadyExists`.
    ///
    /// Scenario: save v1, activate → v2, save v2 (stored is now v2), then
    /// try to save v2 again. The store expects v3 but receives v2, which is
    /// a genuine optimistic-lock conflict, not a duplicate create.
    ///
    /// Устаревшее обновление — повторное сохранение уже сохранённой версии —
    /// должно возвращать `VersionConflict`, а не `AlreadyExists`.
    ///
    /// Сценарий: сохранить v1, активировать → v2, сохранить v2 (в хранилище
    /// теперь v2), затем попытаться сохранить v2 ещё раз. Хранилище ждёт v3,
    /// но получает v2 — это настоящий конфликт оптимистичной блокировки, а не
    /// повторное создание.
    #[tokio::test]
    async fn rejects_stale_customer_update() {
        let repository = InMemoryCustomerRepository::new();
        let now = Utc::now();
        let id = CustomerId::new();

        let mut customer = Customer::create(id, now);
        repository
            .save(&customer)
            .await
            .expect("first save should succeed");

        let permit = ActivationPermit::new(
            customer.id(),
            customer.version(),
            now,
            now + Duration::minutes(5),
        );
        customer
            .activate(permit, now)
            .expect("activation should succeed");
        repository
            .save(&customer)
            .await
            .expect("second save (post-activate) should succeed");

        // customer is still at v2; stored now expects v3
        let result = repository.save(&customer).await;
        assert_eq!(
            result,
            Err(RepositoryError::VersionConflict {
                expected: 3,
                actual: 2
            })
        );
    }
}
