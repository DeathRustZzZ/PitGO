use crate::ids::CustomerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerContactBookCreatedV1 {
    pub customer_id: CustomerId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomerContactBookEvent {
    Created(CustomerContactBookCreatedV1),
}
