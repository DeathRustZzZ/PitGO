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

impl CustomerRepository for InMemoryCustomerRepository {
    fn save(&self, customer: &Customer) -> Result<(), application::error::RepositoryError> {
        let mut customers = self
            .customers
            .lock()
            .map_err(|_| RepositoryError::Unknown)?;

        customers.insert(customer.id(), customer.clone());

        Ok(())
    }

    fn find_by_id(&self, customer_id: CustomerId) -> Result<Option<Customer>, RepositoryError> {
        let customers = self
            .customers
            .lock()
            .map_err(|_| RepositoryError::Unknown)?;

        Ok(customers.get(&customer_id).cloned())
    }
}

impl Default for InMemoryCustomerRepository {
    fn default() -> Self {
        Self::new()
    }
}
