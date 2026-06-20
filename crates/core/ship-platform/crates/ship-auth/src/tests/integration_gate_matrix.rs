/// Gate-matrix integration tests — CallerKind × Scope × grants tier model.
///
/// Covers:
/// - `Scope::Full` bypass (the C2/C3 operator escape hatch)
/// - Space-window session structural invariants (no Full, no AdminDaemon, C2-capped)
/// - `pid_authorizes` enforcement — bound token, unbound token, no-pid transport
/// - `scopes_for_role` and `grants_for_role` per CallerKind shape
/// - `McpAuth::allows` with Full and specific scopes
use crate::scopes::{
    DaemonSession, McpAuth, Scope, grants_for_role, scopes_for_role,
    space_window_session, space_window_scopes, generate_session_token,
    is_session_token,
};

fn make_session(scopes: Vec<Scope>, pid: Option<u32>) -> DaemonSession {
    DaemonSession {
        token: "ship_sess_test".to_string(),
        agent_id: "test-agent".to_string(),
        scopes,
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

// ── Scope::Full bypass ────────────────────────────────────────────────────────

#[test]
fn auth_gate_matrix_full_scope_grants_any_scope() {
    let granted = vec![Scope::Full];
    // Every scope is satisfied by Full.
    assert!(Scope::ReadWorkspace.is_granted_by(&granted));
    assert!(Scope::AdminDaemon.is_granted_by(&granted));
    assert!(Scope::ManageRuns.is_granted_by(&granted));
    assert!(Scope::SuiAuthoring.is_granted_by(&granted));
    assert!(Scope::PromoteArtifact.is_granted_by(&granted));
}

#[test]
fn auth_gate_matrix_no_full_scope_denies_ungranted() {
    let granted = vec![Scope::ReadWorkspace, Scope::LogProgress];
    assert!(!Scope::AdminDaemon.is_granted_by(&granted));
    assert!(!Scope::CreateRuns.is_granted_by(&granted));
    assert!(!Scope::WriteWorkspace.is_granted_by(&granted));
}

#[test]
fn auth_gate_matrix_empty_grant_set_denies_all() {
    let granted: Vec<Scope> = vec![];
    assert!(!Scope::ReadWorkspace.is_granted_by(&granted));
    assert!(!Scope::Full.is_granted_by(&granted));
}

// ── Caller roles → scope sets ─────────────────────────────────────────────────

#[test]
fn auth_gate_matrix_contributor_role_carries_full() {
    let scopes = scopes_for_role(Some("contributor"));
    // The contributor role is the operator's own profile — Full scope.
    assert!(Scope::Full.is_granted_by(&scopes));
}

#[test]
fn auth_gate_matrix_worker_role_has_no_full_no_orchestration() {
    let scopes = scopes_for_role(Some("worker"));
    assert!(!Scope::Full.is_granted_by(&scopes));
    assert!(!Scope::CreateRuns.is_granted_by(&scopes));
    assert!(!Scope::ManageRuns.is_granted_by(&scopes));
    // Workers do get workspace read and run read.
    assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
    assert!(Scope::ReadRuns.is_granted_by(&scopes));
}

#[test]
fn auth_gate_matrix_commander_role_can_orchestrate_not_admin() {
    let scopes = scopes_for_role(Some("commander"));
    assert!(Scope::CreateRuns.is_granted_by(&scopes));
    assert!(Scope::ManageRuns.is_granted_by(&scopes));
    assert!(Scope::MeshBroadcast.is_granted_by(&scopes));
    // Commanders are not the operator.
    assert!(!Scope::Full.is_granted_by(&scopes));
    assert!(!Scope::AdminDaemon.is_granted_by(&scopes));
}

#[test]
fn auth_gate_matrix_designer_role_has_sui_authoring_not_orchestration() {
    let scopes = scopes_for_role(Some("designer"));
    assert!(Scope::SuiAuthoring.is_granted_by(&scopes));
    assert!(Scope::WriteWorkspace.is_granted_by(&scopes));
    assert!(!Scope::CreateRuns.is_granted_by(&scopes));
    assert!(!Scope::ManageRuns.is_granted_by(&scopes));
    assert!(!Scope::Full.is_granted_by(&scopes));
}

#[test]
fn auth_gate_matrix_none_role_matches_worker() {
    let worker = scopes_for_role(Some("worker"));
    let default = scopes_for_role(None);
    assert_eq!(worker, default);
}

// ── grants_for_role (C2 elevated services) ────────────────────────────────────

#[test]
fn auth_gate_matrix_commander_grants_only_agents_service() {
    let grants = grants_for_role(Some("commander"));
    assert_eq!(grants, vec!["agents".to_string()]);
}

#[test]
fn auth_gate_matrix_worker_grants_empty_c1_baseline_only() {
    assert!(grants_for_role(Some("worker")).is_empty());
    assert!(grants_for_role(None).is_empty());
}

#[test]
fn auth_gate_matrix_contributor_grants_all_elevated_services() {
    let grants = grants_for_role(Some("contributor"));
    // The contributor role (operator's own profile) should carry all elevated services.
    // The orchestration domain is plural `agents` (AgentsService proto) — singular
    // `agent` is dead since the agents-control 500s fix.
    assert!(grants.contains(&"agents".to_string()));
    assert!(grants.contains(&"skills".to_string()));
}

// ── Space-window structural invariants ───────────────────────────────────────

#[test]
fn auth_gate_matrix_space_window_never_carries_full_scope() {
    let s = space_window_session("space-abc", 9999, "ship_sess_win".to_string());
    assert!(
        !s.scopes.contains(&Scope::Full),
        "space window must never carry Full (would flip is_operator)"
    );
    assert!(
        !s.scopes.contains(&Scope::AdminDaemon),
        "space window must not carry AdminDaemon"
    );
}

#[test]
fn auth_gate_matrix_space_window_pid_bound_fail_closed_on_wrong_pid() {
    let s = space_window_session("space-abc", 1234, "ship_sess_win".to_string());
    assert!(s.pid_authorizes(Some(1234)), "correct pid authorized");
    assert!(!s.pid_authorizes(Some(9999)), "stolen token from different process rejected");
    // Fail closed when no pid available (TCP transport without SO_PEERCRED).
    assert!(!s.pid_authorizes(None), "no-pid transport must be rejected for bound token");
}

#[test]
fn auth_gate_matrix_space_window_c2_grants_are_contributor_set() {
    let s = space_window_session("space-abc", 1, "ship_sess_win".to_string());
    let expected = grants_for_role(Some("contributor"));
    assert_eq!(s.grants, expected, "space window has the full C2 elevated set");
}

#[test]
fn auth_gate_matrix_space_window_is_ephemeral_never_persisted() {
    let s = space_window_session("space-abc", 1, "ship_sess_win".to_string());
    assert!(s.ephemeral, "space window tokens are never persisted");
    // No expiry unix — ephemeral, lives for the window's run.
    assert!(s.expires_at_unix.is_none());
}

#[test]
fn auth_gate_matrix_space_window_agent_id_encodes_space() {
    let s = space_window_session("acme", 1, "ship_sess_win".to_string());
    // agent_id is "space:<space_id>" — a stable space-surface identity. (The
    // window is C2-capped because it carries no operator cred, not because of
    // this id shape — S6 #124.)
    assert_eq!(s.agent_id, "space:acme");
    assert!(!s.is_operator_cred, "a space window is never the operator cred");
}

// ── Unbound session (daemon token, agent peer session) ────────────────────────

#[test]
fn auth_gate_matrix_unbound_session_passes_any_peer_pid() {
    let s = make_session(vec![Scope::ReadWorkspace], None);
    assert!(s.pid_authorizes(Some(1)));
    assert!(s.pid_authorizes(Some(99999)));
    assert!(s.pid_authorizes(None));
}

// ── McpAuth ───────────────────────────────────────────────────────────────────

#[test]
fn auth_gate_matrix_mcp_auth_full_allows_all() {
    let auth = McpAuth {
        agent_id: "agent-x".to_string(),
        scopes: vec![Scope::Full],
        workspace_id: None,
    };
    assert!(auth.allows(Scope::ReadWorkspace));
    assert!(auth.allows(Scope::AdminDaemon));
    assert!(auth.allows(Scope::SuiAuthoring));
}

#[test]
fn auth_gate_matrix_mcp_auth_worker_scopes_deny_admin() {
    let auth = McpAuth {
        agent_id: "worker-1".to_string(),
        scopes: scopes_for_role(Some("worker")),
        workspace_id: None,
    };
    assert!(auth.allows(Scope::ReadWorkspace));
    assert!(!auth.allows(Scope::AdminDaemon));
    assert!(!auth.allows(Scope::CreateRuns));
}

// ── Token format ──────────────────────────────────────────────────────────────

#[test]
fn auth_gate_matrix_session_token_has_correct_prefix() {
    let token = generate_session_token().unwrap();
    assert!(is_session_token(&token));
    assert!(token.starts_with("ship_sess_"));
    // 16 bytes → 32 hex chars, plus "ship_sess_" (10) = 42 total minimum.
    assert!(token.len() >= 42);
}

#[test]
fn auth_gate_matrix_space_window_scopes_are_broad_but_capped() {
    let scopes = space_window_scopes();
    // Broad: includes orchestration and authoring.
    assert!(scopes.contains(&Scope::CreateRuns));
    assert!(scopes.contains(&Scope::ManageRuns));
    assert!(scopes.contains(&Scope::SuiAuthoring));
    // Capped: no Full, no AdminDaemon.
    assert!(!scopes.contains(&Scope::Full));
    assert!(!scopes.contains(&Scope::AdminDaemon));
}
