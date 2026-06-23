use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositoryError {
    #[error("Connection error")]
    Connection,

    #[error("Concurrency conflict")]
    ConcurrencyConflict,

    #[error("{0}")]
    Infrastructure(String),
}
