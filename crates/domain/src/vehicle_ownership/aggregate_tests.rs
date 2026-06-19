use chrono::Duration;

use shared::aggregate::ChangeOutcome;

use crate::ids::{CustomerId, VehicleId, VehicleOwnershipId};
use crate::vehicle_ownership::error::OwnershipError;
use crate::vehicle_ownership::event::VehicleOwnershipEvent;
use crate::vehicle_ownership::snapshot::OwnershipEligibilitySnapshot;
use crate::vehicle_ownership::state::{OwnershipStatus, OwnershipType};

use super::VehicleOwnership;

fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

fn free_snapshot(vehicle_id: VehicleId) -> OwnershipEligibilitySnapshot {
    OwnershipEligibilitySnapshot::new(vehicle_id, false)
}

fn occupied_snapshot(vehicle_id: VehicleId) -> OwnershipEligibilitySnapshot {
    OwnershipEligibilitySnapshot::new(vehicle_id, true)
}

fn start_ownership(vehicle_id: VehicleId) -> VehicleOwnership {
    VehicleOwnership::start(
        VehicleOwnershipId::new(),
        vehicle_id,
        CustomerId::new(),
        OwnershipType::Private,
        free_snapshot(vehicle_id),
        now(),
    )
    .unwrap()
}

#[test]
fn start_creates_pending_verification_status() {
    let ownership = start_ownership(VehicleId::new());
    assert_eq!(ownership.status(), &OwnershipStatus::PendingVerification);
}

#[test]
fn start_raises_ownership_started_event() {
    let mut ownership = start_ownership(VehicleId::new());
    let events = ownership.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].payload,
        VehicleOwnershipEvent::Started(_)
    ));
}

#[test]
fn start_sets_version_to_one() {
    let ownership = start_ownership(VehicleId::new());
    assert_eq!(ownership.version().value(), 1);
}

#[test]
fn start_with_active_ownership_returns_error() {
    let vehicle_id = VehicleId::new();
    let err = VehicleOwnership::start(
        VehicleOwnershipId::new(),
        vehicle_id,
        CustomerId::new(),
        OwnershipType::Private,
        occupied_snapshot(vehicle_id),
        now(),
    )
    .unwrap_err();
    assert!(matches!(err, OwnershipError::ActiveOwnershipAlreadyExists));
}

#[test]
fn verify_changes_pending_to_active() {
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now()).unwrap();
    assert_eq!(ownership.status(), &OwnershipStatus::Active);
}

#[test]
fn verify_returns_changed() {
    let mut ownership = start_ownership(VehicleId::new());
    let outcome = ownership.verify(now()).unwrap();
    assert_eq!(outcome, ChangeOutcome::Changed);
}

#[test]
fn verify_raises_verified_event() {
    let mut ownership = start_ownership(VehicleId::new());
    ownership.pull_pending_events();
    ownership.verify(now()).unwrap();
    let events = ownership.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].payload,
        VehicleOwnershipEvent::Verified(_)
    ));
}

#[test]
fn verify_increments_version() {
    let mut ownership = start_ownership(VehicleId::new());
    let version_before = ownership.version();
    ownership.verify(now()).unwrap();
    assert_eq!(ownership.version(), version_before.next());
}

#[test]
fn verify_active_returns_no_change() {
    let now = now();
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now).unwrap();
    let version_after = ownership.version();

    let outcome = ownership.verify(now).unwrap();
    assert_eq!(outcome, ChangeOutcome::NoChange);
    assert_eq!(ownership.version(), version_after);
}

#[test]
fn verify_active_no_change_emits_no_events() {
    let now = now();
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now).unwrap();
    ownership.pull_pending_events();

    ownership.verify(now).unwrap();
    assert!(ownership.pull_pending_events().is_empty());
}

#[test]
fn end_active_changes_to_ended() {
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now()).unwrap();
    ownership.end(now()).unwrap();
    assert_eq!(ownership.status(), &OwnershipStatus::Ended);
}

#[test]
fn end_active_returns_changed() {
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now()).unwrap();
    let outcome = ownership.end(now()).unwrap();
    assert_eq!(outcome, ChangeOutcome::Changed);
}

#[test]
fn end_raises_ended_event() {
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now()).unwrap();
    ownership.pull_pending_events();

    ownership.end(now()).unwrap();
    let events = ownership.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].payload, VehicleOwnershipEvent::Ended(_)));
}

#[test]
fn end_closes_period() {
    let started_at = now();
    let vehicle_id = VehicleId::new();
    let mut ownership = VehicleOwnership::start(
        VehicleOwnershipId::new(),
        vehicle_id,
        CustomerId::new(),
        OwnershipType::Private,
        free_snapshot(vehicle_id),
        started_at,
    )
    .unwrap();
    ownership.verify(started_at).unwrap();

    let ended_at = started_at + Duration::days(30);
    ownership.end(ended_at).unwrap();

    assert_eq!(ownership.period().ended_at, Some(ended_at));
}

#[test]
fn end_ended_returns_no_change() {
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now()).unwrap();
    ownership.end(now()).unwrap();
    let version_after = ownership.version();

    let outcome = ownership.end(now()).unwrap();
    assert_eq!(outcome, ChangeOutcome::NoChange);
    assert_eq!(ownership.version(), version_after);
}

#[test]
fn end_ended_emits_no_events() {
    let mut ownership = start_ownership(VehicleId::new());
    ownership.verify(now()).unwrap();
    ownership.end(now()).unwrap();
    ownership.pull_pending_events();

    ownership.end(now()).unwrap();
    assert!(ownership.pull_pending_events().is_empty());
}

#[test]
fn end_pending_verification_returns_error() {
    let mut ownership = start_ownership(VehicleId::new());
    let err = ownership.end(now()).unwrap_err();
    assert!(matches!(err, OwnershipError::StatusDoesNotAllow(_)));
}

#[test]
fn pull_pending_events_drains_buffer() {
    let mut ownership = start_ownership(VehicleId::new());
    let events = ownership.pull_pending_events();
    assert_eq!(events.len(), 1);
    assert!(ownership.pull_pending_events().is_empty());
}

#[test]
fn full_lifecycle_emits_three_events() {
    let mut ownership = start_ownership(VehicleId::new());
    let now = now();
    ownership.verify(now).unwrap();
    ownership.end(now).unwrap();

    let events = ownership.pull_pending_events();
    assert_eq!(events.len(), 3);
    assert!(matches!(
        events[0].payload,
        VehicleOwnershipEvent::Started(_)
    ));
    assert!(matches!(
        events[1].payload,
        VehicleOwnershipEvent::Verified(_)
    ));
    assert!(matches!(events[2].payload, VehicleOwnershipEvent::Ended(_)));
}
