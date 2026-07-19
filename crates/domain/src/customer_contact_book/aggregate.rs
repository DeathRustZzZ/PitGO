use crate::{customer_contact_book::value_objects::phone_contact::PhoneContact, CustomerId};

#[derive(Debug)]
pub struct CustomerContactBook {
    customer_id: CustomerId,
    phone_contact: PhoneContact,
}
