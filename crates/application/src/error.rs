use domain::vehicle_ownership::OwnershipError;
use thiserror::Error;

#[derive(Debug, Error)]

/// ApplicationError is the top-level error type for the application layer
pub enum ApplicationError {
    /// Error when the entity already exists in the repository
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    /// Error when there is a domain-specific error related to ownership
    #[error(transparent)]
    Ownership(#[from] OwnershipError),
}

/// RepositoryError represents errors that can occur in the repository layer
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositoryError {
    /// Error when the entity already exists in the repository
    #[error("optimistic lock conflict: expected version {expected}, found {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    /// Error when there is a storage failure in the repository
    #[error("storage failure: {0}")]
    StorageFailure(String),
}
