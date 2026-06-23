use domain::CustomerId;

/// Command to create a new customer.
#[derive(Debug, Clone, Copy)]
pub struct CreateCustomerCommand {
    pub customer_id: CustomerId,
}
