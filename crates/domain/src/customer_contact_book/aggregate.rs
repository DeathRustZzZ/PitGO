use chrono::{DateTime, Utc};
use shared::aggregate::AggregateVersion;
use shared::event::PendingEvent;

use crate::{
    customer_contact_book::{
        event::{CustomerContactBookCreatedV1, CustomerContactBookEvent},
        value_objects::phone_contact::PhoneContact,
    },
    CustomerId,
};

#[derive(Debug)]
pub struct CustomerContactBook {
    customer_id: CustomerId,
    phone_contact: PhoneContact,
    // email_contact: EmailContact,
    // telegram_contact: TelegramContact,
    // primary_contact: PrimaryContact,
    aggregate_version: AggregateVersion,
    pending_events: Vec<PendingEvent<CustomerContactBookEvent>>,
}

impl CustomerContactBook {
    pub fn new(customer_id: CustomerId, phone_contact: PhoneContact, now: DateTime<Utc>) -> Self {
        let mut contact_book = Self {
            customer_id,
            phone_contact,
            aggregate_version: AggregateVersion::INITIAL,
            pending_events: Vec::new(),
        };
        contact_book.raise(
            CustomerContactBookEvent::Created(CustomerContactBookCreatedV1 { customer_id }),
            now,
        );
        contact_book
    }

    pub fn pull_pending_events(&mut self) -> Vec<PendingEvent<CustomerContactBookEvent>> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn customer_id(&self) -> CustomerId {
        self.customer_id
    }

    pub fn phone_contact(&self) -> &PhoneContact {
        &self.phone_contact
    }

    pub fn version(&self) -> AggregateVersion {
        self.aggregate_version
    }

    fn raise(&mut self, event: CustomerContactBookEvent, occurred_at: DateTime<Utc>) {
        self.pending_events
            .push(PendingEvent::new(event, occurred_at));
        self.aggregate_version = self.aggregate_version.next();
    }
}

#[cfg(test)]
#[path = "tests/aggregate_tests.rs"]
mod aggregate_tests;
