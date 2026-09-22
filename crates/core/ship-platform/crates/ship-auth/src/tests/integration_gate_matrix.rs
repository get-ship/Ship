/// Session/grant integration tests — the capped-session invariants.
///
/// Covers:
/// - Space-window session structural invariants (C2 grant set, never operator)
/// - `pid_authorizes` enforcement — bound token, unbound token, no-pid transport
/// - Session-token format
use crate::scopes::{
    DaemonSession, generate_session_token, is_session_token, space_window_session,
};

fn make_session(pid: Option<u32>) -> DaemonSession {
    DaemonSession {
        token: "ship_sess_test".to_string(),
        agent_id: "test-agent".to_string(),
        grants: vec![],
        space_id: None,
        pid,
        created_at: std::time::Instant::now(),
        expires_at_unix: None,
        ephemeral: true,
        is_operator_cred: false,
    }
}

// ── Space-window structural invariants ───────────────────────────────────────

#[test]
fn auth_gate_matrix_space_window_pid_bound_fail_closed_on_wrong_pid() {
    let s = space_window_session("space-abc", 1234, "ship_sess_win".to_string());
    assert!(s.pid_authorizes(Some(1234)), "correct pid authorized");
    assert!(!s.pid_authorizes(Some(9999)), "stolen token from different process rejected");
    // Fail closed when no pid available (TCP transport without SO_PEERCRED).
    assert!(!s.pid_authorizes(None), "no-pid transport must be rejected for bound token");
}

#[test]
fn auth_gate_matrix_space_window_c2_grants_are_authoring_set() {
    let s = space_window_session("space-abc", 1, "ship_sess_win".to_string());
    assert_eq!(
        s.grants,
        vec!["agents", "skills", "mcp", "create", "view"],
        "space window has the full C2 elevated set"
    );
}

#[test]
fn auth_gate_matrix_space_window_is_ephemeral_never_persisted() {
    let s = space_window_session("space-abc", 1, "ship_sess_win".to_string());
    assert!(s.ephemeral, "space window tokens are never persisted");
    // No expiry unix — ephemeral, lives for the window's run.
    assert!(s.expires_at_unix.is_none());
}

#[test]
fn auth_gate_matrix_space_window_agent_id_encodes_space_and_is_never_operator() {
    let s = space_window_session("acme", 1, "ship_sess_win".to_string());
    // agent_id is "space:<space_id>" — a stable space-surface identity. The
    // window is C2-capped because it carries no operator cred (S6 #124).
    assert_eq!(s.agent_id, "space:acme");
    assert!(!s.is_operator_cred, "a space window is never the operator cred");
}

// ── Unbound session (agent peer session) ──────────────────────────────────────

#[test]
fn auth_gate_matrix_unbound_session_passes_any_peer_pid() {
    let s = make_session(None);
    assert!(s.pid_authorizes(Some(1)));
    assert!(s.pid_authorizes(Some(99999)));
    assert!(s.pid_authorizes(None));
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
