use std::sync::{Arc, Mutex};

use domain::CustomerId;
use domain::customer::aggregate::Customer;

use crate::customer::commands::CreateCustomerCommand;
use crate::customer::handlers::CreateCustomerHandler;
use crate::customer::ports::CustomerRepository;
use crate::error::{ApplicationError, RepositoryError};

// ── Mock ──────────────────────────────────────────────────────────────────────

struct MockCustomerRepository {
    saved_ids: Mutex<Vec<CustomerId>>,
    save_error: Option<RepositoryError>,
}

impl MockCustomerRepository {
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

    fn saved_ids(&self) -> Vec<CustomerId> {
        self.saved_ids.lock().unwrap().clone()
    }
}

impl CustomerRepository for MockCustomerRepository {
    fn save(&self, customer: &Customer) -> Result<(), RepositoryError> {
        if let Some(ref err) = self.save_error {
            return Err(err.clone());
        }
        self.saved_ids.lock().unwrap().push(customer.id());
        Ok(())
    }

    fn find_by_id(&self, _id: CustomerId) -> Result<Option<Customer>, RepositoryError> {
        Ok(None)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cmd(id: CustomerId) -> CreateCustomerCommand {
    CreateCustomerCommand { customer_id: id }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn handle_returns_ok_on_success() {
    let repo = MockCustomerRepository::ok();
    let handler = CreateCustomerHandler::new(Arc::clone(&repo) as Arc<dyn CustomerRepository>);
    assert!(handler.handle(cmd(CustomerId::new())).is_ok());
}

#[test]
fn handle_saves_customer_with_correct_id() {
    let repo = MockCustomerRepository::ok();
    let handler = CreateCustomerHandler::new(Arc::clone(&repo) as Arc<dyn CustomerRepository>);
    let id = CustomerId::new();

    handler.handle(cmd(id)).unwrap();

    assert_eq!(repo.saved_ids(), vec![id]);
}

#[test]
fn handle_does_not_save_on_repository_error() {
    let repo = MockCustomerRepository::failing(RepositoryError::StorageFailure("io error".into()));
    let handler = CreateCustomerHandler::new(Arc::clone(&repo) as Arc<dyn CustomerRepository>);

    let _ = handler.handle(cmd(CustomerId::new()));

    assert!(repo.saved_ids().is_empty());
}

#[test]
fn handle_propagates_storage_failure() {
    let repo = MockCustomerRepository::failing(RepositoryError::StorageFailure("disk full".into()));
    let handler = CreateCustomerHandler::new(repo as Arc<dyn CustomerRepository>);

    let err = handler.handle(cmd(CustomerId::new())).unwrap_err();

    assert!(matches!(
        err,
        ApplicationError::Repository(RepositoryError::StorageFailure(_))
    ));
}

#[test]
fn handle_propagates_version_conflict() {
    let repo = MockCustomerRepository::failing(RepositoryError::VersionConflict {
        expected: 1,
        actual: 2,
    });
    let handler = CreateCustomerHandler::new(repo as Arc<dyn CustomerRepository>);

    let err = handler.handle(cmd(CustomerId::new())).unwrap_err();

    assert!(matches!(
        err,
        ApplicationError::Repository(RepositoryError::VersionConflict {
            expected: 1,
            actual: 2
        })
    ));
}
