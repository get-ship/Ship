#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("actor '{actor_id}' not found")]
    ActorNotFound { actor_id: String },
    #[error("actor '{actor_id}' already exists")]
    ActorAlreadyExists { actor_id: String },
    #[error("namespace violation: '{namespace}' not in write scope for '{actor_id}'")]
    NamespaceViolation { namespace: String, actor_id: String },
    #[error("mailbox full for actor '{actor_id}'")]
    MailboxFull { actor_id: String },
    #[error("persistence error: {0}")]
    Persistence(#[from] anyhow::Error),
}

impl From<ship_kernel::KernelError> for KernelError {
    fn from(e: ship_kernel::KernelError) -> Self {
        match e {
            ship_kernel::KernelError::ActorNotFound { actor_id } => {
                KernelError::ActorNotFound { actor_id }
            }
            ship_kernel::KernelError::ActorAlreadyExists { actor_id } => {
                KernelError::ActorAlreadyExists { actor_id }
            }
            ship_kernel::KernelError::NamespaceViolation {
                namespace,
                actor_id,
            } => KernelError::NamespaceViolation {
                namespace,
                actor_id,
            },
            ship_kernel::KernelError::MailboxFull { actor_id } => {
                KernelError::MailboxFull { actor_id }
            }
            ship_kernel::KernelError::ValidationFailed { reason } => {
                KernelError::Persistence(anyhow::anyhow!("validation failed: {reason}"))
            }
            ship_kernel::KernelError::Persistence(e) => KernelError::Persistence(e),
        }
    }
}
