use connectrpc::ConnectError;

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("blob: {0}")]
    Blob(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<ArtifactError> for ConnectError {
    fn from(e: ArtifactError) -> Self {
        match e {
            ArtifactError::NotFound(m) => ConnectError::not_found(m),
            ArtifactError::Validation(m) => ConnectError::invalid_argument(m),
            ArtifactError::Blob(m) => ConnectError::internal(m),
            ArtifactError::Internal(m) => ConnectError::internal(m),
        }
    }
}

impl From<ship_substrate_core::ports::DataStoreError> for ArtifactError {
    fn from(e: ship_substrate_core::ports::DataStoreError) -> Self {
        ArtifactError::Internal(e.to_string())
    }
}

impl From<ship_substrate_core::errors::KernelError> for ArtifactError {
    fn from(e: ship_substrate_core::errors::KernelError) -> Self {
        ArtifactError::Internal(e.to_string())
    }
}

impl From<anyhow::Error> for ArtifactError {
    fn from(e: anyhow::Error) -> Self {
        ArtifactError::Internal(e.to_string())
    }
}
