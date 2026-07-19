//! Unit tests for the [`Vehicle`] aggregate.
//!
//! Verify that creation and activation each raise exactly one event and advance
//! the version by one, that activating an already-`Active` vehicle is an
//! idempotent `NoChange` rather than an error, and that the permit's local
//! conditions — matching `vehicle_id` and expiry — are enforced.
//!
//! Юнит-тесты агрегата [`Vehicle`].
//!
//! Проверяют, что создание и активация порождают ровно одно событие и
//! увеличивают версию на единицу; что активация уже активного автомобиля даёт
//! идемпотентный `NoChange`, а не ошибку; и что локальные условия permit —
//! совпадение `vehicle_id` и срок действия — действительно проверяются.

use chrono::Duration;

use shared::aggregate::ChangeOutcome;

use crate::ids::VehicleId;
use crate::vehicle::error::{VehicleActivationError, VehicleError};
use crate::vehicle::event::VehicleEvent;
use crate::vehicle::permit::VehicleActivationPermit;
use crate::vehicle::state::VehicleStatus;

use super::Vehicle;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

fn valid_permit(vehicle: &Vehicle, now: chrono::DateTime<chrono::Utc>) -> VehicleActivationPermit {
    VehicleActivationPermit::new(vehicle.id(), now, now + Duration::minutes(5))
}

#[test]
fn create_sets_draft_status() {
    let vehicle = Vehicle::create(VehicleId::new(), now());
    assert_eq!(vehicle.status(), &VehicleStatus::Draft);
}

#[test]
fn create_assigns_correct_id() {
    let id = VehicleId::new();
    let vehicle = Vehicle::create(id, now());
    assert_eq!(vehicle.id(), id);
}

#[test]
fn create_raises_vehicle_created_event() {
    let mut vehicle = Vehicle::create(VehicleId::new(), now());
    let events = vehicle.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].payload, VehicleEvent::Created(_)));
}

#[test]
fn create_sets_version_to_one() {
    let vehicle = Vehicle::create(VehicleId::new(), now());
    assert_eq!(vehicle.version().value(), 1);
}

#[test]
fn activate_from_draft_returns_changed() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let permit = valid_permit(&vehicle, now);
    let outcome = vehicle.activate(permit, now).unwrap();
    assert_eq!(outcome, ChangeOutcome::Changed);
}

#[test]
fn activate_from_draft_sets_active_status() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let permit = valid_permit(&vehicle, now);
    vehicle.activate(permit, now).unwrap();
    assert_eq!(vehicle.status(), &VehicleStatus::Active);
}

#[test]
fn activate_increments_version() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let version_before = vehicle.version();
    let permit = valid_permit(&vehicle, now);
    vehicle.activate(permit, now).unwrap();
    assert_eq!(vehicle.version(), version_before.next());
}

#[test]
fn activate_raises_vehicle_activated_event() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    vehicle.pull_pending_events();
    let permit = valid_permit(&vehicle, now);
    vehicle.activate(permit, now).unwrap();
    let events = vehicle.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].payload, VehicleEvent::Activated(_)));
}

#[test]
fn activate_updates_updated_at() {
    let created_at = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), created_at);
    let activated_at = created_at + Duration::seconds(10);
    let permit = valid_permit(&vehicle, activated_at);
    vehicle.activate(permit, activated_at).unwrap();
    assert_eq!(vehicle.updated_at(), activated_at);
}

#[test]
fn activate_already_active_returns_no_change() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let permit = valid_permit(&vehicle, now);
    vehicle.activate(permit, now).unwrap();

    let version_after = vehicle.version();
    let permit2 = valid_permit(&vehicle, now);
    let outcome = vehicle.activate(permit2, now).unwrap();

    assert_eq!(outcome, ChangeOutcome::NoChange);
    assert_eq!(vehicle.version(), version_after);
}

#[test]
fn activate_no_change_emits_no_events() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let permit = valid_permit(&vehicle, now);
    vehicle.activate(permit, now).unwrap();
    vehicle.pull_pending_events();

    let permit2 = valid_permit(&vehicle, now);
    vehicle.activate(permit2, now).unwrap();
    assert!(vehicle.pull_pending_events().is_empty());
}

#[test]
fn activate_with_wrong_vehicle_id_returns_error() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let wrong_permit =
        VehicleActivationPermit::new(VehicleId::new(), now, now + Duration::minutes(5));
    let err = vehicle.activate(wrong_permit, now).unwrap_err();
    assert!(matches!(
        err,
        VehicleError::Activation(VehicleActivationError::PermitVehicleIdMismatch)
    ));
}

#[test]
fn activate_with_expired_permit_returns_error() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let expired_permit = VehicleActivationPermit::new(
        vehicle.id(),
        now - Duration::minutes(10),
        now - Duration::minutes(5),
    );
    let err = vehicle.activate(expired_permit, now).unwrap_err();
    assert!(matches!(
        err,
        VehicleError::Activation(VehicleActivationError::PermitExpired)
    ));
}

#[test]
fn pull_pending_events_drains_buffer() {
    let now = now();
    let mut vehicle = Vehicle::create(VehicleId::new(), now);
    let permit = valid_permit(&vehicle, now);
    vehicle.activate(permit, now).unwrap();

    let events = vehicle.pull_pending_events();
    assert_eq!(events.len(), 2);
    assert!(vehicle.pull_pending_events().is_empty());
}
