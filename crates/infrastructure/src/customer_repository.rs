use std::collections::HashMap;
use std::sync::Mutex;

use application::customer::ports::CustomerRepository;
use application::error::RepositoryError;
use domain::CustomerId;
use domain::customer::Customer;

/// An in-memory implementation of the CustomerRepository trait
pub struct InMemoryCustomerRepository {
    customers: Mutex<HashMap<CustomerId, Customer>>,
}

/// Creates a new instance of InMemoryCustomerRepository
impl InMemoryCustomerRepository {
    pub fn new() -> Self {
        Self {
            customers: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl CustomerRepository for InMemoryCustomerRepository {
    /// Saves a customer to the repository, checking for version conflicts
    async fn save(&self, customer: &Customer) -> Result<(), application::error::RepositoryError> {
        let mut customers = self
            .customers
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;

        let actual = customer.version();
        let expected = customers
            .get(&customer.id())
            .map(|stored| stored.version().next());

        if let Some(expected_version) = expected
            && expected_version != actual
        {
            return Err(RepositoryError::VersionConflict {
                expected: expected_version.value(),
                actual: actual.value(),
            });
        }
        customers.insert(customer.id(), customer.clone());

        Ok(())
    }

    async fn find_by_id(
        &self,
        customer_id: CustomerId,
    ) -> Result<Option<Customer>, RepositoryError> {
        let customers = self
            .customers
            .lock()
            .map_err(|e| RepositoryError::StorageFailure(e.to_string()))?;
        Ok(customers.get(&customer_id).cloned())
    }
}

impl Default for InMemoryCustomerRepository {
    fn default() -> Self {
        Self::new()
    }
}
