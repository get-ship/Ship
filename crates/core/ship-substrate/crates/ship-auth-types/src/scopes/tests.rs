//! Tests for the `scopes` module — types, role tables, and the session
//! surface (registry, minters, token formatters).

use std::sync::Arc;
use tokio::sync::Mutex;

use super::roles::{grants_for_role, scopes_for_role};
use super::session::{
    drop_session, generate_session_token, is_session_token, mint_session, operator_session,
    session_registry, set_session_registry, space_window_session, DaemonSession, SessionRegistry,
};
use super::types::Scope;

fn session_bound_to(pid: Option<u32>) -> DaemonSession {
    DaemonSession {
        token: "ship_sess_x".to_string(),
        agent_id: "win".to_string(),
        scopes: vec![],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid,
        created_at: std::time::Instant::now(),
        expires_at_unix: None,
        ephemeral: true,
        is_operator_cred: false,
    }
}

#[test]
fn operator_session_is_the_pid_attested_operator_cred() {
    let s = operator_session(4242, "ship_sess_op".to_string());
    // The one elevation source: an operator cred, pid-attested.
    assert!(s.is_operator_cred, "operator_session is THE operator cred");
    assert_eq!(s.pid, Some(4242), "the cred is bound to the shell process");
    assert!(s.pid_authorizes(Some(4242)), "the bound process is authorized");
    assert!(!s.pid_authorizes(Some(1)), "a stolen cred from another process is rejected");
    assert!(!s.pid_authorizes(None), "fails closed over an unattested transport (TCP)");
    // Ephemeral — pid-attested creds are never persisted.
    assert!(s.ephemeral, "operator creds are never persisted");
    // The operator clears the grant axis (no per-service grants needed).
    assert!(s.grants.is_empty(), "operator clears the gate's grant axis");
}

#[test]
fn space_window_session_is_pid_bound_c2_never_operator() {
    let s = space_window_session("acme-space", 4242, "ship_sess_win".to_string());
    // Bound to the registering process.
    assert_eq!(s.pid, Some(4242));
    assert!(s.pid_authorizes(Some(4242)));
    assert!(!s.pid_authorizes(Some(1)));
    // Structurally never the operator: no Full scope (Full would flip
    // is_operator and clear C3).
    assert!(
        !s.scopes.contains(&Scope::Full),
        "a space window token must never carry Scope::Full"
    );
    assert!(
        !s.scopes.contains(&Scope::AdminDaemon),
        "a space window token must not carry AdminDaemon"
    );
    // Has the full C2 elevated set (operator's own profile).
    assert_eq!(s.grants, grants_for_role(Some("contributor")));
    assert!(s.ephemeral, "window tokens are never persisted");
    // NEVER the operator cred — that is the separate `operator_session`.
    assert!(!s.is_operator_cred, "a space window is C2-capped, never the operator cred");
}

#[test]
fn unbound_session_passes_regardless_of_peer_pid() {
    // CLI daemon token, agent peer session, legacy session — not pid-bound.
    let s = session_bound_to(None);
    assert!(s.pid_authorizes(Some(1234)));
    assert!(s.pid_authorizes(None));
}

#[test]
fn bound_session_requires_matching_peer_pid() {
    let s = session_bound_to(Some(4242));
    assert!(s.pid_authorizes(Some(4242)), "the bound process is authorized");
    assert!(!s.pid_authorizes(Some(9999)), "a different process is rejected (stolen token)");
}

#[test]
fn bound_session_fails_closed_without_peer_pid() {
    // A pid-bound window token presented over a transport that can't
    // attest a pid (TCP, no SO_PEERCRED) must be rejected — the binding
    // can't be stripped by switching transports.
    let s = session_bound_to(Some(4242));
    assert!(!s.pid_authorizes(None));
}

#[test]
fn scope_full_grants_everything() {
    let granted = vec![Scope::Full];
    assert!(Scope::CreateRuns.is_granted_by(&granted));
    assert!(Scope::AdminDaemon.is_granted_by(&granted));
    assert!(Scope::MeshBroadcast.is_granted_by(&granted));
}

#[test]
fn scope_specific_grant() {
    let granted = vec![Scope::ReadWorkspace, Scope::LogProgress];
    assert!(Scope::ReadWorkspace.is_granted_by(&granted));
    assert!(Scope::LogProgress.is_granted_by(&granted));
    assert!(!Scope::CreateRuns.is_granted_by(&granted));
}

#[test]
fn scopes_for_worker_role() {
    let scopes = scopes_for_role(Some("worker"));
    assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
    assert!(Scope::LogProgress.is_granted_by(&scopes));
    assert!(!Scope::CreateRuns.is_granted_by(&scopes));
    assert!(!Scope::ManageRuns.is_granted_by(&scopes));
}

#[test]
fn scopes_for_commander_role() {
    let scopes = scopes_for_role(Some("commander"));
    assert!(Scope::CreateRuns.is_granted_by(&scopes));
    assert!(Scope::ManageRuns.is_granted_by(&scopes));
    assert!(Scope::MeshBroadcast.is_granted_by(&scopes));
}

#[test]
fn scopes_for_contributor_role() {
    let scopes = scopes_for_role(Some("contributor"));
    assert!(Scope::Full.is_granted_by(&scopes));
    assert!(Scope::AdminDaemon.is_granted_by(&scopes));
}

#[test]
fn designer_role_grants_sui_authoring_but_not_orchestration() {
    let scopes = scopes_for_role(Some("designer"));
    assert!(Scope::SuiAuthoring.is_granted_by(&scopes));
    assert!(Scope::WriteWorkspace.is_granted_by(&scopes));
    // Designers are not orchestrators.
    assert!(!Scope::CreateRuns.is_granted_by(&scopes));
    assert!(!Scope::ManageRuns.is_granted_by(&scopes));
}

#[test]
fn non_designer_roles_lack_sui_authoring() {
    assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(Some("worker"))));
    assert!(!Scope::SuiAuthoring.is_granted_by(&scopes_for_role(Some("commander"))));
    // Full bypasses, so contributor implicitly has it.
    assert!(Scope::SuiAuthoring.is_granted_by(&scopes_for_role(Some("contributor"))));
}

#[test]
fn scopes_for_none_role_defaults_to_worker() {
    let worker = scopes_for_role(Some("worker"));
    let none = scopes_for_role(None);
    assert_eq!(worker, none);
}

#[test]
fn session_token_format() {
    let token = generate_session_token().unwrap();
    assert!(is_session_token(&token));
}

#[tokio::test]
async fn mint_session_registers_scoped_bound_token() {
    let registry: SessionRegistry =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    let (token, scopes) = mint_session(
        &registry,
        "rust-lane".into(),
        Some("proj-a".into()),
        Some("worker"),
        Some(3600),
    )
    .await
    .unwrap();

    assert!(is_session_token(&token));
    assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
    assert!(!Scope::Full.is_granted_by(&scopes)); // per-process != root

    let reg = registry.lock().await;
    let session = reg.get(&token).expect("registered");
    assert_eq!(session.workspace_id.as_deref(), Some("proj-a"));
    assert_eq!(session.agent_id, "rust-lane");
    assert!(session.ephemeral);
    assert!(session.expires_at_unix.is_some());
}

#[tokio::test]
async fn global_registry_set_then_mint_visible() {
    // Order-independent across the test binary: whoever wins the
    // OnceLock::set, `session_registry()` returns *a* registry; minting
    // into that handle and reading it back must hold.
    let reg: SessionRegistry = Arc::new(Mutex::new(std::collections::HashMap::new()));
    set_session_registry(reg.clone());
    let installed = session_registry().expect("a registry is installed");
    let (token, _) =
        mint_session(&installed, "a".into(), Some("p".into()), Some("worker"), None)
            .await
            .unwrap();
    assert!(installed.lock().await.contains_key(&token));
}

#[tokio::test]
async fn drop_session_invalidates_token() {
    let registry: SessionRegistry =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    let (token, _) =
        mint_session(&registry, "a".into(), None, Some("worker"), None)
            .await
            .unwrap();
    assert!(registry.lock().await.contains_key(&token));
    drop_session(&registry, &token).await;
    assert!(!registry.lock().await.contains_key(&token));
}

#[test]
fn worker_can_write_run_artifacts_but_not_project() {
    let scopes = scopes_for_role(Some("worker"));
    assert!(Scope::WriteArtifactsRun.is_granted_by(&scopes));
    assert!(!Scope::WriteArtifactsProject.is_granted_by(&scopes));
}

#[test]
fn worker_cannot_promote_artifacts() {
    let scopes = scopes_for_role(Some("worker"));
    assert!(!Scope::PromoteArtifact.is_granted_by(&scopes));
}

#[test]
fn only_contributor_and_full_can_promote_artifacts() {
    assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("commander"))));
    assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("lead"))));
    assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("worker"))));
    assert!(Scope::PromoteArtifact.is_granted_by(&[Scope::Full]));
}

#[test]
fn worker_can_read_project_artifacts_but_not_write() {
    let scopes = scopes_for_role(Some("worker"));
    assert!(Scope::ReadArtifactsProject.is_granted_by(&scopes));
    assert!(!Scope::WriteArtifactsProject.is_granted_by(&scopes));
    assert!(!Scope::DeleteArtifactsProject.is_granted_by(&scopes));
}

// SHORTCOMING — the grant model above is correct, but NOTHING enforces it at the artifact write
// wire (ship-artifacts/src/wire.rs patch/promote/delete never check caller scopes vs the target
// scope), so an agent can currently mutate/promote PROJECT-scoped artifacts. This #[ignore]'d test
// pins the intent and fails-until-implemented; the dedicated artifacts authz pass must wire a
// per-scope check, replace the panic with a wire-level denial assertion, and remove #[ignore].
#[test]
#[ignore = "enforcement not wired — dedicated artifacts authz pass"]
fn agent_is_denied_project_scoped_artifact_write() {
    let agent = scopes_for_role(Some("worker"));
    assert!(!Scope::WriteArtifactsProject.is_granted_by(&agent), "agents must not be granted project write");
    assert!(!Scope::DeleteArtifactsProject.is_granted_by(&agent), "agents must not be granted project delete");
    panic!("artifacts authz: the write wire does not enforce the above — agent project-artifact writes are NOT denied today");
}
