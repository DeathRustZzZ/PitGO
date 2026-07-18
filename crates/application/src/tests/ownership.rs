use std::sync::{Arc, Mutex};

use domain::vehicle_ownership::aggregate::VehicleOwnership;
use domain::vehicle_ownership::error::OwnershipError;
use domain::vehicle_ownership::state::OwnershipType;
use domain::{CustomerId, VehicleId, VehicleOwnershipId};

use crate::error::{ApplicationError, RepositoryError};
use crate::ownership::commands::StartVehicleOwnershipCommand;
use crate::ownership::handlers::StartVehicleOwnershipHandler;
use crate::ownership::ports::VehicleOwnershipRepository;

// ── Mock ──────────────────────────────────────────────────────────────────────

struct MockOwnershipRepository {
    has_active: bool,
    has_active_error: Option<RepositoryError>,
    save_error: Option<RepositoryError>,
    saved_ids: Mutex<Vec<VehicleOwnershipId>>,
}

impl MockOwnershipRepository {
    fn ok(has_active: bool) -> Arc<Self> {
        Arc::new(Self {
            has_active,
            has_active_error: None,
            save_error: None,
            saved_ids: Mutex::new(vec![]),
        })
    }

    fn failing_on_check(error: RepositoryError) -> Arc<Self> {
        Arc::new(Self {
            has_active: false,
            has_active_error: Some(error),
            save_error: None,
            saved_ids: Mutex::new(vec![]),
        })
    }

    fn failing_on_save(error: RepositoryError) -> Arc<Self> {
        Arc::new(Self {
            has_active: false,
            has_active_error: None,
            save_error: Some(error),
            saved_ids: Mutex::new(vec![]),
        })
    }

    fn saved_ids(&self) -> Vec<VehicleOwnershipId> {
        self.saved_ids.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl VehicleOwnershipRepository for MockOwnershipRepository {
    async fn has_open_ownership(&self, _vehicle_id: VehicleId) -> Result<bool, RepositoryError> {
        if let Some(ref err) = self.has_active_error {
            return Err(err.clone());
        }

        Ok(self.has_active)
    }

    async fn save(&self, ownership: &VehicleOwnership) -> Result<(), RepositoryError> {
        if let Some(ref err) = self.save_error {
            return Err(err.clone());
        }

        self.saved_ids.lock().unwrap().push(ownership.id());

        Ok(())
    }

    async fn find_by_id(
        &self,
        _id: VehicleOwnershipId,
    ) -> Result<Option<VehicleOwnership>, RepositoryError> {
        Ok(None)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cmd() -> StartVehicleOwnershipCommand {
    StartVehicleOwnershipCommand {
        ownership_id: VehicleOwnershipId::new(),
        vehicle_id: VehicleId::new(),
        owner_customer_id: CustomerId::new(),
        ownership_type: OwnershipType::Private,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn handle_returns_ok_when_no_active_ownership() {
    let repo = MockOwnershipRepository::ok(false);

    let handler =
        StartVehicleOwnershipHandler::new(Arc::clone(&repo) as Arc<dyn VehicleOwnershipRepository>);

    assert!(handler.handle(cmd()).await.is_ok());
}

#[tokio::test]
async fn handle_saves_ownership_with_correct_id() {
    let repo = MockOwnershipRepository::ok(false);

    let handler =
        StartVehicleOwnershipHandler::new(Arc::clone(&repo) as Arc<dyn VehicleOwnershipRepository>);

    let command = cmd();
    let expected_id = command.ownership_id;

    handler.handle(command).await.unwrap();

    assert_eq!(repo.saved_ids(), vec![expected_id]);
}

#[tokio::test]
async fn handle_returns_domain_error_when_active_ownership_exists() {
    let repo = MockOwnershipRepository::ok(true);

    let handler = StartVehicleOwnershipHandler::new(repo as Arc<dyn VehicleOwnershipRepository>);

    let err = handler.handle(cmd()).await.unwrap_err();

    assert!(matches!(
        err,
        ApplicationError::Ownership(OwnershipError::ActiveOwnershipAlreadyExists)
    ));
}

#[tokio::test]
async fn handle_does_not_save_when_active_ownership_exists() {
    let repo = MockOwnershipRepository::ok(true);

    let handler =
        StartVehicleOwnershipHandler::new(Arc::clone(&repo) as Arc<dyn VehicleOwnershipRepository>);

    let _ = handler.handle(cmd()).await;

    assert!(repo.saved_ids().is_empty());
}

#[tokio::test]
async fn handle_propagates_has_active_ownership_error() {
    let repo = MockOwnershipRepository::failing_on_check(RepositoryError::StorageFailure(
        "connection lost".into(),
    ));

    let handler = StartVehicleOwnershipHandler::new(repo as Arc<dyn VehicleOwnershipRepository>);

    let err = handler.handle(cmd()).await.unwrap_err();

    assert!(matches!(
        err,
        ApplicationError::Repository(RepositoryError::StorageFailure(_))
    ));
}

#[tokio::test]
async fn handle_propagates_save_error() {
    let repo = MockOwnershipRepository::failing_on_save(RepositoryError::StorageFailure(
        "disk full".into(),
    ));

    let handler = StartVehicleOwnershipHandler::new(repo as Arc<dyn VehicleOwnershipRepository>);

    let err = handler.handle(cmd()).await.unwrap_err();

    assert!(matches!(
        err,
        ApplicationError::Repository(RepositoryError::StorageFailure(_))
    ));
}

#[tokio::test]
async fn handle_does_not_save_on_save_error() {
    let repo = MockOwnershipRepository::failing_on_save(RepositoryError::StorageFailure(
        "disk full".into(),
    ));

    let handler =
        StartVehicleOwnershipHandler::new(Arc::clone(&repo) as Arc<dyn VehicleOwnershipRepository>);

    let _ = handler.handle(cmd()).await;

    assert!(repo.saved_ids().is_empty());
}
