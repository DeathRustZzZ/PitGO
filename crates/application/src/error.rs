use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Repository operation failed")]
    Failed,
}
