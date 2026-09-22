use connectrpc::ConnectError;

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("payload too large: {0}")]
    PayloadTooLarge(String),
    #[error("blob: {0}")]
    Blob(String),
    #[error("internal: {0}")]
    Internal(String),
}

/// Hard byte cap on the `Create` payload (artifacts are documents and outputs,
/// not bulk media — proto header in `artifacts.proto` promises an enforced cap).
pub const MAX_PAYLOAD_BYTES: usize = 8 * 1024 * 1024;

impl From<ArtifactError> for ConnectError {
     fn from(e: ArtifactError) -> Self {
         match e {
             ArtifactError::NotFound(m) => ConnectError::not_found(m),
             ArtifactError::Validation(m) => ConnectError::invalid_argument(m),
            ArtifactError::PayloadTooLarge(m) => ConnectError::invalid_argument(m),
             ArtifactError::Blob(m) => ConnectError::internal(m),
             ArtifactError::Internal(m) => ConnectError::internal(m),
         }
     }
 }

impl From<ship_substrate::ports::DataStoreError> for ArtifactError {
    fn from(e: ship_substrate::ports::DataStoreError) -> Self {
        ArtifactError::Internal(e.to_string())
    }
}

impl From<ship_substrate::errors::KernelError> for ArtifactError {
    fn from(e: ship_substrate::errors::KernelError) -> Self {
        ArtifactError::Internal(e.to_string())
    }
}

impl From<anyhow::Error> for ArtifactError {
    fn from(e: anyhow::Error) -> Self {
        ArtifactError::Internal(e.to_string())
    }
}
