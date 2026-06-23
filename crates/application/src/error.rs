use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositoryError {
    /// Error when the entity already exists in the repository
    #[error("optimistic lock conflict: expected version {expected}, found {actual}")]
    VersionConflict { expected: u64, actual: u64 },

    /// Error when there is a storage failure in the repository
    #[error("storage failure: {0}")]
    StorageFailure(String),
}
