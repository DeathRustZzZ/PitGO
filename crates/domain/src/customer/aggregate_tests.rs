use chrono::Duration;

use shared::aggregate::ChangeOutcome;

use crate::customer::error::{CustomerActivationError, CustomerError};
use crate::customer::event::CustomerEvent;
use crate::customer::permit::ActivationPermit;
use crate::customer::state::CustomerStatus;
use crate::ids::CustomerId;

use super::Customer;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

fn valid_permit(customer: &Customer, now: chrono::DateTime<chrono::Utc>) -> ActivationPermit {
    ActivationPermit::new(
        customer.id(),
        customer.version(),
        now,
        now + Duration::minutes(5),
    )
}

#[test]
fn create_sets_draft_status() {
    let id = CustomerId::new();
    let customer = Customer::create(id, now());
    assert_eq!(customer.status(), &CustomerStatus::Draft);
}

#[test]
fn create_assigns_correct_id() {
    let id = CustomerId::new();
    let customer = Customer::create(id, now());
    assert_eq!(customer.id(), id);
}

#[test]
fn create_raises_customer_created_event() {
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now());
    let events = customer.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].payload, CustomerEvent::Created(_)));
}

#[test]
fn create_sets_version_to_one() {
    let id = CustomerId::new();
    let customer = Customer::create(id, now());
    assert_eq!(customer.version().value(), 1);
}

#[test]
fn activate_from_draft_returns_changed() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);
    let permit = valid_permit(&customer, now);

    let outcome = customer.activate(permit, now).unwrap();
    assert_eq!(outcome, ChangeOutcome::Changed);
}

#[test]
fn activate_from_draft_sets_active_status() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);
    let permit = valid_permit(&customer, now);

    customer.activate(permit, now).unwrap();
    assert_eq!(customer.status(), &CustomerStatus::Active);
}

#[test]
fn activate_increments_version() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);
    let permit = valid_permit(&customer, now);

    let version_before = customer.version();
    customer.activate(permit, now).unwrap();
    assert_eq!(customer.version(), version_before.next());
}

#[test]
fn activate_raises_customer_activated_event() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);
    customer.pull_pending_events();

    let permit = valid_permit(&customer, now);
    customer.activate(permit, now).unwrap();

    let events = customer.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].payload, CustomerEvent::Activated(_)));
}

#[test]
fn activate_updates_updated_at() {
    let created_at = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, created_at);

    let activated_at = created_at + Duration::seconds(10);
    let permit = valid_permit(&customer, activated_at);
    customer.activate(permit, activated_at).unwrap();

    assert_eq!(customer.updated_at(), activated_at);
}

#[test]
fn activate_already_active_returns_no_change() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);

    let permit = valid_permit(&customer, now);
    customer.activate(permit, now).unwrap();

    let version_after_activate = customer.version();
    let permit2 = valid_permit(&customer, now);
    let outcome = customer.activate(permit2, now).unwrap();

    assert_eq!(outcome, ChangeOutcome::NoChange);
    assert_eq!(customer.version(), version_after_activate);
}

#[test]
fn activate_no_change_emits_no_events() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);

    let permit = valid_permit(&customer, now);
    customer.activate(permit, now).unwrap();
    customer.pull_pending_events();

    let permit2 = valid_permit(&customer, now);
    customer.activate(permit2, now).unwrap();

    assert!(customer.pull_pending_events().is_empty());
}

#[test]
fn activate_with_expired_permit_returns_error() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);

    let expired_permit = ActivationPermit::new(
        customer.id(),
        customer.version(),
        now - Duration::minutes(10),
        now - Duration::minutes(5),
    );

    let err = customer.activate(expired_permit, now).unwrap_err();
    assert!(matches!(
        err,
        CustomerError::Activation(CustomerActivationError::PermitExpired)
    ));
}

#[test]
fn activate_with_wrong_customer_id_returns_error() {
    let now = now();
    let mut customer = Customer::create(CustomerId::new(), now);

    let wrong_permit = ActivationPermit::new(
        CustomerId::new(),
        customer.version(),
        now,
        now + Duration::minutes(5),
    );

    let err = customer.activate(wrong_permit, now).unwrap_err();
    assert!(matches!(
        err,
        CustomerError::Activation(CustomerActivationError::PermitCustomerIdMismatch)
    ));
}

#[test]
fn activate_with_stale_version_returns_error() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);

    let stale_permit = ActivationPermit::new(
        customer.id(),
        shared::aggregate::AggregateVersion::INITIAL,
        now,
        now + Duration::minutes(5),
    );

    let err = customer.activate(stale_permit, now).unwrap_err();
    assert!(matches!(
        err,
        CustomerError::Activation(CustomerActivationError::PermitVersionMismatch)
    ));
}

#[test]
fn pull_pending_events_clears_buffer() {
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now());
    customer.pull_pending_events();
    assert!(customer.pull_pending_events().is_empty());
}

#[test]
fn pull_pending_events_returns_all_accumulated_events() {
    let now = now();
    let id = CustomerId::new();
    let mut customer = Customer::create(id, now);
    let permit = valid_permit(&customer, now);
    customer.activate(permit, now).unwrap();

    let events = customer.pull_pending_events();
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0].payload, CustomerEvent::Created(_)));
    assert!(matches!(events[1].payload, CustomerEvent::Activated(_)));
}
