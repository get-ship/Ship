#[cfg(test)]
mod tests {
    use crate::workspace::helpers::*;
    use crate::workspace::types::*;

    #[test]
    fn workspace_status_default_is_idle() {
        assert_eq!(WorkspaceStatus::default(), WorkspaceStatus::Idle);
    }

    #[test]
    fn workspace_status_display() {
        assert_eq!(WorkspaceStatus::Idle.to_string(), "idle");
        assert_eq!(WorkspaceStatus::Active.to_string(), "active");
    }

    #[test]
    fn workspace_branch_key_validation_rejects_empty_values() {
        let err = ensure_branch_key("   ").unwrap_err();
        assert!(
            err.to_string()
                .contains("Workspace branch/key cannot be empty")
        );
    }
}
