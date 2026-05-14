use crate::errors::{KernelError, WorkspaceError};
use crate::events::kernel_router::{ActorConfig, KernelRouter};
use tempfile::tempdir;

fn actor_config(ns: &str) -> ActorConfig {
    ActorConfig {
        namespace: ns.to_string(),
        write_namespaces: vec![format!("{ns}.")],
        read_namespaces: vec![format!("{ns}.")],
        subscribe_namespaces: vec![format!("{ns}.")],
    }
}

fn setup() -> (tempfile::TempDir, KernelRouter) {
    let tmp = tempdir().unwrap();
    let router = KernelRouter::new(crate::ShipHome::project_ship_dir(tmp.path())).unwrap();
    (tmp, router)
}

// ── KernelError ──────────────────────────────────────────────────────────────

#[test]
fn spawn_existing_actor_returns_already_exists() {
    let (_tmp, mut router) = setup();
    router.spawn_actor("agent.a", actor_config("agent.a")).unwrap();
    let err = router.spawn_actor("agent.a", actor_config("agent.a")).unwrap_err();
    assert!(
        matches!(err, KernelError::ActorAlreadyExists { ref actor_id } if actor_id == "agent.a"),
        "expected ActorAlreadyExists, got: {err:?}"
    );
}

#[test]
fn stop_missing_actor_returns_not_found() {
    let (_tmp, mut router) = setup();
    let err = router.stop_actor("agent.ghost").unwrap_err();
    assert!(
        matches!(err, KernelError::ActorNotFound { ref actor_id } if actor_id == "agent.ghost"),
        "expected ActorNotFound, got: {err:?}"
    );
}

#[test]
fn kernel_error_display_is_readable() {
    let err = KernelError::ActorNotFound { actor_id: "x".into() };
    assert_eq!(err.to_string(), "actor 'x' not found");

    let err = KernelError::ActorAlreadyExists { actor_id: "y".into() };
    assert_eq!(err.to_string(), "actor 'y' already exists");
}

// ── WorkspaceError ───────────────────────────────────────────────────────────

#[test]
fn workspace_error_display_is_readable() {
    let err = WorkspaceError::NotFound { branch: "feat/x".into() };
    assert_eq!(err.to_string(), "Workspace not found for branch 'feat/x'");

    let err = WorkspaceError::InvalidStatus {
        current: "active".into(),
        attempted: "deleted".into(),
    };
    assert!(err.to_string().contains("invalid status transition"));
}
