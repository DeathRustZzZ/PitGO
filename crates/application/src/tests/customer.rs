use std::sync::{Arc, Mutex};

use async_trait::async_trait;
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
            saved_ids: Mutex::new(Vec::new()),
            save_error: None,
        })
    }

    fn failing(error: RepositoryError) -> Arc<Self> {
        Arc::new(Self {
            saved_ids: Mutex::new(Vec::new()),
            save_error: Some(error),
        })
    }

    fn saved_ids(&self) -> Vec<CustomerId> {
        self.saved_ids
            .lock()
            .expect("mock saved_ids mutex was poisoned")
            .clone()
    }
}

#[async_trait]
impl CustomerRepository for MockCustomerRepository {
    async fn save(&self, customer: &Customer) -> Result<(), RepositoryError> {
        if let Some(error) = &self.save_error {
            return Err(error.clone());
        }

        self.saved_ids
            .lock()
            .expect("mock saved_ids mutex was poisoned")
            .push(customer.id());

        Ok(())
    }

    async fn find_by_id(&self, _id: CustomerId) -> Result<Option<Customer>, RepositoryError> {
        Ok(None)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn cmd(id: CustomerId) -> CreateCustomerCommand {
    CreateCustomerCommand { customer_id: id }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn handle_returns_ok_on_success() {
    let repo = MockCustomerRepository::ok();

    let handler = CreateCustomerHandler::new(Arc::clone(&repo) as Arc<dyn CustomerRepository>);

    let result = handler.handle(cmd(CustomerId::new())).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn handle_saves_customer_with_correct_id() {
    let repo = MockCustomerRepository::ok();

    let handler = CreateCustomerHandler::new(Arc::clone(&repo) as Arc<dyn CustomerRepository>);

    let id = CustomerId::new();

    handler
        .handle(cmd(id))
        .await
        .expect("customer creation should succeed");

    assert_eq!(repo.saved_ids(), vec![id]);
}

#[tokio::test]
async fn handle_does_not_save_on_repository_error() {
    let repo = MockCustomerRepository::failing(RepositoryError::StorageFailure("io error".into()));

    let handler = CreateCustomerHandler::new(Arc::clone(&repo) as Arc<dyn CustomerRepository>);

    let result = handler.handle(cmd(CustomerId::new())).await;

    assert!(result.is_err());
    assert!(repo.saved_ids().is_empty());
}

#[tokio::test]
async fn handle_propagates_storage_failure() {
    let repo = MockCustomerRepository::failing(RepositoryError::StorageFailure("disk full".into()));

    let handler = CreateCustomerHandler::new(repo as Arc<dyn CustomerRepository>);

    let error = handler
        .handle(cmd(CustomerId::new()))
        .await
        .expect_err("handler should propagate repository error");

    assert!(matches!(
        error,
        ApplicationError::Repository(RepositoryError::StorageFailure(_))
    ));
}

#[tokio::test]
async fn handle_propagates_version_conflict() {
    let repo = MockCustomerRepository::failing(RepositoryError::VersionConflict {
        expected: 1,
        actual: 2,
    });

    let handler = CreateCustomerHandler::new(repo as Arc<dyn CustomerRepository>);

    let error = handler
        .handle(cmd(CustomerId::new()))
        .await
        .expect_err("handler should propagate version conflict");

    assert!(matches!(
        error,
        ApplicationError::Repository(RepositoryError::VersionConflict {
            expected: 1,
            actual: 2,
        })
    ));
}
