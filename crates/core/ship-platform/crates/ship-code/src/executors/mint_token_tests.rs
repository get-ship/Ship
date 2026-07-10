//! Tests for the sandbox scoped-token seam (`mint_scoped_token`).
//!
//! The security contract — a sandboxed agent gets a *scoped* per-process token,
//! never the daemon/Full token — is enforced at two layers:
//!
//!   1. `ship_auth::scopes` proves `mint_session` yields non-Full
//!      scopes and `drop_session` revokes them (its own unit tests).
//!   2. Here we pin the supervisor seam: `mint_scoped_token` injects that
//!      minted token into `LaunchCommand.env["SHIP_TOKEN"]` and the value is
//!      genuinely scoped — so the operator/Full token can never reach a sandbox
//!      via the spawn path (the "root token must never enter a container" rule).

use std::collections::HashMap;
use std::sync::Arc;

use ship_auth::scopes::{self, Scope};
use crate::dispatch_ports::{LaunchCommand, LaunchProfile, SessionContext};

use super::process::mint_scoped_token;

fn ctx_for(agent: &str) -> SessionContext {
    let tmp = tempfile::tempdir().expect("tempdir");
    SessionContext {
        session_id: "job-tok".into(),
        slug: "slug-tok".into(),
        agent: agent.into(),
        branch: "main".into(),
        spec_path: ".artifacts/job-spec.md".into(),
        spec_content: None,
        work_dir: tmp.keep(),
        env: HashMap::new(),
        launch: LaunchProfile::default(),
        exit_tx: None,
        step: None,
        invocation: crate::dispatch_ports::InvocationMode::Print,
        keepalive: false,
        // Test fixture — no per-call fork source.
        repo_path: None,
        base_ref: None,
    }
}

#[tokio::test]
async fn mint_scoped_token_injects_a_scoped_not_full_token() {
    // Install a process-wide registry (no-op if another test already did;
    // either way `session_registry()` is Some afterward).
    scopes::set_session_registry(Arc::new(tokio::sync::Mutex::new(HashMap::new())));

    let ctx = ctx_for("agent-tok");
    let mut launch = LaunchCommand { bin: "claude".into(), args: vec![], env: HashMap::new() };

    let token = mint_scoped_token(&ctx, &mut launch).await.expect("token minted");

    // The injected token is the minted *session* token — never the 64-hex
    // daemon/Full token.
    assert!(
        token.starts_with("ship_sess_"),
        "expected a scoped session token, got {token}"
    );
    assert_eq!(
        launch.env.get("SHIP_TOKEN"),
        Some(&token),
        "SHIP_TOKEN must carry the minted token"
    );

    // And it is genuinely scoped (worker role, not Full) in the registry.
    let registry = scopes::session_registry().expect("registry installed");
    let guard = registry.lock().await;
    let session = guard.get(&token).expect("minted session present in registry");
    assert!(
        !session.scopes.contains(&Scope::Full),
        "a sandbox token must never hold Scope::Full"
    );
}
