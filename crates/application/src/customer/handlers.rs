use crate::customer::commands::CreateCustomerCommand;
use crate::customer::ports::CustomerRepository;
use crate::error::ApplicationError;
use chrono::Utc;
use domain::CustomerId;
use domain::customer::aggregate::Customer;
use std::sync::Arc;

/// Handler for creating a new customer
pub struct CreateCustomerHandler {
    repository: Arc<dyn CustomerRepository>,
}

impl CreateCustomerHandler {
    pub fn new(repository: Arc<dyn CustomerRepository>) -> Self {
        Self { repository }
    }

    pub fn handle(&self, cmd: CreateCustomerCommand) -> Result<(), ApplicationError> {
        let now = Utc::now();
        let customer = Customer::create(cmd.customer_id, now);

        self.repository.save(&customer)?;

        Ok(())
    }
}

/// Handler for searching for a customer by ID
pub struct GetCustomerHandler {
    repository: Arc<dyn CustomerRepository>,
}

impl GetCustomerHandler {
    pub fn new(repository: Arc<dyn CustomerRepository>) -> Self {
        Self { repository }
    }

    pub fn handle(&self, customer_id: CustomerId) -> Result<Option<Customer>, ApplicationError> {
        Ok(self.repository.find_by_id(customer_id)?)
    }
}
