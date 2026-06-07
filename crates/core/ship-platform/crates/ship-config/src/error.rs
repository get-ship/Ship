//! Error types for the config service.

use connectrpc::ConnectError;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("validation: {0}")]
    Validation(String),
    /// Authority-gate rejection — the caller lacks operator authority for the
    /// requested scope write.
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("fs: {0}")]
    Fs(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<ConfigError> for ConnectError {
    fn from(e: ConfigError) -> Self {
        match e {
            ConfigError::NotFound(m) => ConnectError::not_found(m),
            ConfigError::AlreadyExists(m) => ConnectError::invalid_argument(m),
            ConfigError::Validation(m) => ConnectError::invalid_argument(m),
            ConfigError::Forbidden(m) => ConnectError::permission_denied(m),
            ConfigError::Fs(m) => ConnectError::internal(m),
            ConfigError::Internal(m) => ConnectError::internal(m),
        }
    }
}

impl From<ship_substrate::ports::FsError> for ConfigError {
    fn from(e: ship_substrate::ports::FsError) -> Self {
        ConfigError::Fs(e.to_string())
    }
}

impl From<ship_substrate::ports::DataStoreError> for ConfigError {
    fn from(e: ship_substrate::ports::DataStoreError) -> Self {
        ConfigError::Internal(e.to_string())
    }
}

impl From<ship_substrate::errors::KernelError> for ConfigError {
    fn from(e: ship_substrate::errors::KernelError) -> Self {
        ConfigError::Internal(e.to_string())
    }
}
