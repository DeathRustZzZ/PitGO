use shared::aggregate::AggregateVersion;

use crate::{customer_contact_book::value_objects::phone_contact::PhoneContact, CustomerId};

#[derive(Debug)]
pub struct CustomerContactBook {
    customer_id: CustomerId,
    phone_contact: PhoneContact,
    // email_contact: EmailContact,
    // telegram_contact: TelegramContact,
    // primary_contact: PrimaryContact,
    aggregate_version: AggregateVersion,
}
