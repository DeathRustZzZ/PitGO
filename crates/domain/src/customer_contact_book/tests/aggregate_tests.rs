use chrono::{TimeZone, Utc};
use shared::aggregate::AggregateVersion;

use crate::{
    customer_contact_book::{
        aggregate::CustomerContactBook,
        event::{CustomerContactBookCreatedV1, CustomerContactBookEvent},
    },
    ids::CustomerId,
};

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 24, 12, 0, 0).unwrap()
}

#[test]
fn new_assigns_customer_id() {
    let customer_id = CustomerId::new();

    let contact_book = CustomerContactBook::new(customer_id, now());

    assert_eq!(contact_book.customer_id(), customer_id);
}

#[test]
fn new_has_no_phone_contact() {
    // Новая контактная книга создаётся без телефона.
    // A new contact book is created without a phone.
    let contact_book = CustomerContactBook::new(CustomerId::new(), now());

    assert!(contact_book.phone_contact().is_none());
}

#[test]
fn new_sets_version_to_one() {
    let contact_book = CustomerContactBook::new(CustomerId::new(), now());

    assert_eq!(contact_book.version(), AggregateVersion::from(1));
}

#[test]
fn new_raises_contact_book_created_event() {
    let customer_id = CustomerId::new();
    let mut contact_book = CustomerContactBook::new(customer_id, now());

    let events = contact_book.pull_pending_events();

    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].payload,
        CustomerContactBookEvent::Created(CustomerContactBookCreatedV1 { customer_id })
    );
}

#[test]
fn pull_pending_events_clears_buffer() {
    let mut contact_book = CustomerContactBook::new(CustomerId::new(), now());

    contact_book.pull_pending_events();

    assert!(contact_book.pull_pending_events().is_empty());
}
