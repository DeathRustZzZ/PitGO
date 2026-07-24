use chrono::{TimeZone, Utc};
use shared::aggregate::AggregateVersion;

use crate::{
    customer_contact_book::{
        aggregate::CustomerContactBook,
        event::{CustomerContactBookCreatedV1, CustomerContactBookEvent},
        value_objects::{phone_contact::PhoneContact, phone_number::PhoneNumber},
    },
    ids::CustomerId,
};

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 24, 12, 0, 0).unwrap()
}

fn phone_contact() -> PhoneContact {
    PhoneContact::new(PhoneNumber::parse("+375291234567").unwrap())
}

#[test]

fn new_assigns_customer_id() {
    let customer_id = CustomerId::new();

    let contact_book = CustomerContactBook::new(customer_id, phone_contact(), now());

    assert_eq!(contact_book.customer_id(), customer_id);
}

#[test]
fn new_sets_phone_contact() {
    let contact_book = CustomerContactBook::new(CustomerId::new(), phone_contact(), now());

    assert_eq!(
        contact_book.phone_contact().number().as_str(),
        "+375291234567"
    );
}

#[test]
fn new_sets_version_to_one() {
    let contact_book = CustomerContactBook::new(CustomerId::new(), phone_contact(), now());

    assert_eq!(contact_book.version(), AggregateVersion::from(1));
}

#[test]
fn new_raises_contact_book_created_event() {
    let customer_id = CustomerId::new();
    let mut contact_book = CustomerContactBook::new(customer_id, phone_contact(), now());

    let events = contact_book.pull_pending_events();

    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].payload,
        CustomerContactBookEvent::Created(CustomerContactBookCreatedV1 { customer_id })
    );
}

#[test]
fn pull_pending_events_clears_buffer() {
    let mut contact_book = CustomerContactBook::new(CustomerId::new(), phone_contact(), now());

    contact_book.pull_pending_events();

    assert!(contact_book.pull_pending_events().is_empty());
}
