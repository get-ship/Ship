//! Error types for the space service.

use connectrpc::ConnectError;

#[derive(Debug, thiserror::Error)]
pub enum SpaceError {
    /// Invalid request input (→ invalid_argument / 400).
    #[error("bad args: {0}")]
    BadArgs(String),
    /// The named entity does not exist (→ not_found / 404).
    #[error("not found: {0}")]
    NotFound(String),
    /// A required precondition is unmet, e.g. no space identity (→ failed_precondition / 409).
    #[error("conflict: {0}")]
    Conflict(String),
    /// An internal failure (→ internal / 500).
    #[error("internal: {0}")]
    Internal(String),
}

impl From<SpaceError> for ConnectError {
    fn from(e: SpaceError) -> Self {
        match e {
            SpaceError::BadArgs(m) => ConnectError::invalid_argument(m),
            SpaceError::NotFound(m) => ConnectError::not_found(m),
            SpaceError::Conflict(m) => ConnectError::failed_precondition(m),
            SpaceError::Internal(m) => ConnectError::internal(m),
        }
    }
}

impl From<ship_substrate::ports::DataStoreError> for SpaceError {
    fn from(e: ship_substrate::ports::DataStoreError) -> Self {
        SpaceError::Internal(e.to_string())
    }
}

impl From<ship_substrate::errors::KernelError> for SpaceError {
    fn from(e: ship_substrate::errors::KernelError) -> Self {
        SpaceError::Internal(e.to_string())
    }
}
