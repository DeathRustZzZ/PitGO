use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use domain::VehicleId;
use domain::vehicle::aggregate::Vehicle;

use crate::error::{ApplicationError, RepositoryError};
use crate::vehicle::commands::CreateVehicleCommand;
use crate::vehicle::handlers::CreateVehicleHandler;
use crate::vehicle::ports::VehicleRepository;

// ── Mock ──────────────────────────────────────────────────────────────────────

struct MockVehicleRepository {
    saved_ids: Mutex<Vec<VehicleId>>,
    save_error: Option<RepositoryError>,
}

impl MockVehicleRepository {
    fn ok() -> Arc<Self> {
        Arc::new(Self {
            saved_ids: Mutex::new(vec![]),
            save_error: None,
        })
    }

    fn failing(error: RepositoryError) -> Arc<Self> {
        Arc::new(Self {
            saved_ids: Mutex::new(vec![]),
            save_error: Some(error),
        })
    }

    fn saved_ids(&self) -> Vec<VehicleId> {
        self.saved_ids.lock().unwrap().clone()
    }
}

#[async_trait]
impl VehicleRepository for MockVehicleRepository {
    async fn save(&self, vehicle: &Vehicle) -> Result<(), RepositoryError> {
        if let Some(ref err) = self.save_error {
            return Err(err.clone());
        }
        self.saved_ids.lock().unwrap().push(vehicle.id());
        Ok(())
    }

    async fn find_by_id(&self, _id: VehicleId) -> Result<Option<Vehicle>, RepositoryError> {
        Ok(None)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cmd(id: VehicleId) -> CreateVehicleCommand {
    CreateVehicleCommand { vehicle_id: id }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn handle_returns_ok_on_success() {
    let repo = MockVehicleRepository::ok();
    let handler = CreateVehicleHandler::new(Arc::clone(&repo) as Arc<dyn VehicleRepository>);
    assert!(handler.handle(cmd(VehicleId::new())).await.is_ok());
}

#[tokio::test]
async fn handle_saves_vehicle_with_correct_id() {
    let repo = MockVehicleRepository::ok();
    let handler = CreateVehicleHandler::new(Arc::clone(&repo) as Arc<dyn VehicleRepository>);
    let id = VehicleId::new();

    handler
        .handle(cmd(id))
        .await
        .expect("vehicle creation should succeed");

    assert_eq!(repo.saved_ids(), vec![id]);
}

#[tokio::test]
async fn handle_does_not_save_on_repository_error() {
    let repo = MockVehicleRepository::failing(RepositoryError::StorageFailure("io error".into()));
    let handler = CreateVehicleHandler::new(Arc::clone(&repo) as Arc<dyn VehicleRepository>);

    let _ = handler.handle(cmd(VehicleId::new())).await;

    assert!(repo.saved_ids().is_empty());
}

#[tokio::test]
async fn handle_propagates_storage_failure() {
    let repo = MockVehicleRepository::failing(RepositoryError::StorageFailure("disk full".into()));
    let handler = CreateVehicleHandler::new(repo as Arc<dyn VehicleRepository>);

    let err = handler.handle(cmd(VehicleId::new())).await.unwrap_err();

    assert!(matches!(
        err,
        ApplicationError::Repository(RepositoryError::StorageFailure(_))
    ));
}

#[tokio::test]
async fn handle_propagates_version_conflict() {
    let repo = MockVehicleRepository::failing(RepositoryError::VersionConflict {
        expected: 1,
        actual: 2,
    });
    let handler = CreateVehicleHandler::new(repo as Arc<dyn VehicleRepository>);

    let err = handler.handle(cmd(VehicleId::new())).await.unwrap_err();

    assert!(matches!(
        err,
        ApplicationError::Repository(RepositoryError::VersionConflict {
            expected: 1,
            actual: 2
        })
    ));
}
