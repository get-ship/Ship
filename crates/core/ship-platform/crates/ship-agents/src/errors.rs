//! Product/workspace errors for the agent-session domain.
//!
//! `WorkspaceError` lives here (not in the L0 engine): it models branch /
//! session / compile failures owned by agent dispatch, not kernel plumbing.
//! Kernel errors stay in `ship_substrate::errors::KernelError`.

use thiserror::Error;

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

#[cfg(test)]
mod tests {
    use super::WorkspaceError;

    #[test]
    fn workspace_error_display_is_readable() {
        let err = WorkspaceError::NotFound {
            branch: "feat/x".into(),
        };
        assert_eq!(err.to_string(), "Workspace not found for branch 'feat/x'");

        let err = WorkspaceError::InvalidStatus {
            current: "active".into(),
            attempted: "deleted".into(),
        };
        assert!(err.to_string().contains("invalid status transition"));
    }
}
