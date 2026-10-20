use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum VeloError {
    #[error("Player error: {0}")]
    Player(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("{0}")]
    Summary(String),

    #[error("{message}")]
    Model {
        code: ModelErrorCode,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelErrorCode {
    Auth,
    Connection,
    RateLimit,
    Provider,
    Context,
    Timeout,
    Empty,
    Incomplete,
    Cancelled,
}

pub type Result<T> = std::result::Result<T, VeloError>;
