//! Tests for the session surface — registry, minters, token formatters,
//! and pid-binding.

use std::sync::Arc;
use tokio::sync::Mutex;

use super::session::{
    drop_session, generate_session_token, is_session_token, mint_session, operator_session,
    space_window_session, DaemonSession, SessionRegistry,
};

fn session_bound_to(pid: Option<u32>) -> DaemonSession {
    DaemonSession {
        token: "ship_sess_x".to_string(),
        agent_id: "win".to_string(),
        grants: vec![],
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
    // Has the full C2 elevated set (operator's own profile), never C3.
    assert_eq!(s.grants, vec!["agents", "skills", "mcp", "create", "view"]);
    assert!(s.ephemeral, "window tokens are never persisted");
    // NEVER the operator cred — that is the separate `operator_session`.
    assert!(!s.is_operator_cred, "a space window is C2-capped, never the operator cred");
}

#[test]
fn unbound_session_passes_regardless_of_peer_pid() {
    // Agent peer session, legacy session — not pid-bound.
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
fn session_token_format() {
    let token = generate_session_token().unwrap();
    assert!(is_session_token(&token));
}

#[tokio::test]
async fn mint_session_registers_bound_token() {
    let registry: SessionRegistry =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    let token = mint_session(
        &registry,
        "rust-lane".into(),
        vec!["agents".to_string()],
        Some(3600),
    )
    .await
    .unwrap();

    assert!(is_session_token(&token));

    let reg = registry.lock().await;
    let session = reg.get(&token).expect("registered");
    assert_eq!(session.agent_id, "rust-lane");
    assert_eq!(session.grants, vec!["agents".to_string()]);
    assert!(session.ephemeral);
    assert!(session.expires_at_unix.is_some());
}

#[tokio::test]
async fn drop_session_invalidates_token() {
    let registry: SessionRegistry =
        Arc::new(Mutex::new(std::collections::HashMap::new()));
    let token = mint_session(&registry, "a".into(), vec![], None)
        .await
        .unwrap();
    assert!(registry.lock().await.contains_key(&token));
    drop_session(&registry, &token).await;
    assert!(!registry.lock().await.contains_key(&token));
}
