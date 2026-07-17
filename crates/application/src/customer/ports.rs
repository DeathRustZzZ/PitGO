use crate::error::RepositoryError;
use domain::CustomerId;
use domain::customer::aggregate::Customer;

#[async_trait::async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn save(&self, customer: &Customer) -> Result<(), RepositoryError>;

    async fn find_by_id(
        &self,
        customer_id: CustomerId,
    ) -> Result<Option<Customer>, RepositoryError>;
}
