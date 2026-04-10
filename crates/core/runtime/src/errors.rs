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

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Workspace not found for branch '{branch}'")]
    NotFound { branch: String },
    #[error("workspace already exists: branch '{branch}'")]
    AlreadyExists { branch: String },
    #[error("invalid status transition: '{current}' → '{attempted}'")]
    InvalidStatus { current: String, attempted: String },
    #[error("session already active: '{session_id}'")]
    SessionActive { session_id: String },
    #[error("compile failed: {error}")]
    CompileFailed { error: String },
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
}
