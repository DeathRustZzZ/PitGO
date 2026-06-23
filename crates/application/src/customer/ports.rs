use crate::error::RepositoryError;
use domain::CustomerId;
use domain::customer::aggregate::Customer;

pub trait CustomerRepository {
    fn save(&self, customer: &mut Customer) -> Result<(), RepositoryError>;

    fn find_by_id(&self, customer_id: CustomerId) -> Result<Option<Customer>, RepositoryError>;
}
