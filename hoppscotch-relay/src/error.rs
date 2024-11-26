use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum RelayError {
    #[error("Invalid method")]
    InvalidMethod,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Invalid headers")]
    InvalidHeaders,
    #[error("Request cancelled")]
    RequestCancelled,
    #[error("Request run error: {0}")]
    RequestRunError(String),
}

pub type RelayResult<T> = std::result::Result<T, RelayError>;
