//! Error types for the collection service.

use connectrpc::ConnectError;

#[derive(Debug, thiserror::Error)]
pub enum CollectionError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("reserved id: {0}")]
    ReservedId(String),
    #[error("unsupported query: {0}")]
    UnsupportedQuery(String),
    #[error("named query not found: {0}")]
    NamedQueryNotFound(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<CollectionError> for ConnectError {
    fn from(e: CollectionError) -> Self {
        match e {
            CollectionError::NotFound(m) => ConnectError::not_found(m),
            CollectionError::Validation(m) => ConnectError::invalid_argument(m),
            CollectionError::ReservedId(m) => ConnectError::invalid_argument(m),
            CollectionError::UnsupportedQuery(m) => ConnectError::invalid_argument(m),
            CollectionError::NamedQueryNotFound(m) => ConnectError::not_found(m),
            CollectionError::Internal(m) => ConnectError::internal(m),
        }
    }
}

impl From<ship_substrate_core::ports::DataStoreError> for CollectionError {
    fn from(e: ship_substrate_core::ports::DataStoreError) -> Self {
        CollectionError::Internal(e.to_string())
    }
}

impl From<ship_substrate_core::errors::KernelError> for CollectionError {
    fn from(e: ship_substrate_core::errors::KernelError) -> Self {
        CollectionError::Internal(e.to_string())
    }
}
