// Error types for Paradigm core
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParadigmError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Consensus error: {0}")]
    Consensus(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid task")]
    InvalidTask,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid address")]
    InvalidAddress,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for ParadigmError {
    fn from(err: anyhow::Error) -> Self {
        ParadigmError::Other(err.to_string())
    }
}

impl From<&str> for ParadigmError {
    fn from(err: &str) -> Self {
        ParadigmError::Other(err.to_string())
    }
}
