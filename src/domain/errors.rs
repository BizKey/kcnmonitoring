use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type DomainResult<T> = Result<T, DomainError>;

impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        DomainError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for DomainError {
    fn from(err: sqlx::Error) -> Self {
        DomainError::Database(err.to_string())
    }
}

impl From<reqwest::Error> for DomainError {
    fn from(err: reqwest::Error) -> Self {
        DomainError::Api(err.to_string())
    }
}
