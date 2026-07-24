use crate::{customer_contact_book::value_objects::phone_number::PhoneNumber, ids::CustomerId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerContactBookCreatedV1 {
    pub customer_id: CustomerId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomerContactBookEvent {
    Created(CustomerContactBookCreatedV1),
    PhoneAdded(PhoneAddedV1),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneAddedV1 {
    pub customer_id: CustomerId,
    pub phone_number: PhoneNumber,
}
