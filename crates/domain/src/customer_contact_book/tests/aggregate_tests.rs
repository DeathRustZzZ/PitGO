use chrono::{TimeZone, Utc};
use shared::aggregate::{AggregateVersion, ChangeOutcome};

use crate::{
    customer_contact_book::{
        aggregate::CustomerContactBook,
        error::CustomerContactBookError,
        event::{CustomerContactBookCreatedV1, CustomerContactBookEvent, PhoneAddedV1},
        value_objects::{phone_contact::VerificationStatus, phone_number::PhoneNumber},
    },
    ids::CustomerId,
};

fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 24, 12, 0, 0).unwrap()
}

fn phone_number() -> PhoneNumber {
    PhoneNumber::parse("+375291234567").unwrap()
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

#[test]
fn add_phone_sets_unverified_phone_contact() {
    let mut contact_book = CustomerContactBook::new(CustomerId::new(), now());

    let outcome = contact_book.add_phone(phone_number(), now()).unwrap();

    assert_eq!(outcome, ChangeOutcome::Changed);
    assert_eq!(
        contact_book.phone_contact().unwrap().number().as_str(),
        "+375291234567"
    );
    assert!(matches!(
        contact_book.phone_contact().unwrap().verification(),
        VerificationStatus::Unverified
    ));
}

#[test]
fn add_phone_increments_version_and_raises_event() {
    let customer_id = CustomerId::new();
    let mut contact_book = CustomerContactBook::new(customer_id, now());
    contact_book.pull_pending_events();

    contact_book.add_phone(phone_number(), now()).unwrap();
    let events = contact_book.pull_pending_events();

    assert_eq!(contact_book.version(), AggregateVersion::from(2));
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].payload,
        CustomerContactBookEvent::PhoneAdded(PhoneAddedV1 {
            customer_id,
            phone_number: phone_number(),
        })
    );
}

#[test]
fn add_phone_when_phone_exists_returns_error_without_changes() {
    let mut contact_book = CustomerContactBook::new(CustomerId::new(), now());
    contact_book.add_phone(phone_number(), now()).unwrap();
    contact_book.pull_pending_events();
    let version = contact_book.version();

    let result = contact_book.add_phone(PhoneNumber::parse("+375331234567").unwrap(), now());

    assert_eq!(result, Err(CustomerContactBookError::PhoneAlreadyExists));
    assert_eq!(contact_book.version(), version);
    assert_eq!(
        contact_book.phone_contact().unwrap().number().as_str(),
        "+375291234567"
    );
    assert!(contact_book.pull_pending_events().is_empty());
}
