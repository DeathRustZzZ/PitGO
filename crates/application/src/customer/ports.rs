use crate::error::RepositoryError;
use domain::CustomerId;
use domain::customer::aggregate::Customer;

pub trait CustomerRepository: Send + Sync {
    fn save(&self, customer: &Customer) -> Result<(), RepositoryError>;

    fn find_by_id(&self, customer_id: CustomerId) -> Result<Option<Customer>, RepositoryError>;
}
