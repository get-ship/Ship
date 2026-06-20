/// Integration tests — browser session and DaemonSession lifecycle.
///
/// Covers:
/// - Browser session issue/lookup/expiry (in-memory registry)
/// - `parse_session_cookie` happy path and negative cases
/// - `is_browser_session_id` prefix check
/// - `DaemonSession::is_expired` — no-expiry, future expiry, past expiry
/// - `persist_session` emits `auth.session.created` (event-observe test)
/// - `remove_persisted_session` emits `auth.session.revoked`
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::scopes::{
    DaemonSession, Scope, SessionRegistry, generate_session_token, persist_session,
    remove_persisted_session,
};
use crate::session::{
    BROWSER_SESSION_TTL_SECS, COOKIE_NAME, generate_browser_session_id, is_browser_session_id,
    issue_browser_session, issue_browser_session_with_ttl, lookup_browser_session,
    parse_session_cookie,
};
use crate::testutil::{init_auth, query_events};
use ship_types::event_types::{AUTH_SESSION_CREATED, AUTH_SESSION_REVOKED};
use tempfile::tempdir;

fn empty_registry() -> SessionRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

fn persisted_session(token: &str) -> DaemonSession {
    DaemonSession {
        token: token.to_string(),
        agent_id: "test-agent".to_string(),
        scopes: vec![Scope::ReadWorkspace],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: Some(chrono::Utc::now().timestamp() + 86400),
        ephemeral: false,
        is_operator_cred: false,
    }
}

// ── Cookie parsing ────────────────────────────────────────────────────────────

#[test]
fn auth_session_cookie_parser_extracts_id() {
    let hdr = format!("{COOKIE_NAME}=abc123");
    assert_eq!(parse_session_cookie(&hdr), Some("abc123".to_string()));
}

#[test]
fn auth_session_cookie_parser_multi_cookie_header() {
    let hdr = format!("foo=bar; {COOKIE_NAME}=myid; other=1");
    assert_eq!(parse_session_cookie(&hdr), Some("myid".to_string()));
}

#[test]
fn auth_session_cookie_parser_absent_returns_none() {
    assert!(parse_session_cookie("foo=bar; baz=qux").is_none());
}

#[test]
fn auth_session_cookie_parser_empty_value_returns_none() {
    let hdr = format!("{COOKIE_NAME}=");
    assert!(parse_session_cookie(&hdr).is_none());
}

// ── Browser session id ────────────────────────────────────────────────────────

#[test]
fn auth_session_browser_id_has_prefix_and_length() {
    let id = generate_browser_session_id();
    assert!(is_browser_session_id(&id));
    // 32 bytes → 64 hex chars + "ship_browser_" (13) = 77 total.
    assert_eq!(id.len(), 13 + 64);
}

#[test]
fn auth_session_session_token_is_not_browser_id() {
    let token = generate_session_token().unwrap();
    assert!(!is_browser_session_id(&token));
}

// ── Browser session registry ──────────────────────────────────────────────────

#[tokio::test]
async fn auth_session_browser_issue_then_lookup_succeeds() {
    let reg = empty_registry();
    let id = issue_browser_session(&reg).await;
    let session = lookup_browser_session(&reg, &id).await.expect("session present");
    assert!(session.scopes.contains(&Scope::Full));
    assert!(session.expires_at_unix.is_some());
    let ttl = session.expires_at_unix.unwrap() - chrono::Utc::now().timestamp();
    // Allow ±1 second tolerance.
    assert!(ttl > 0 && ttl <= BROWSER_SESSION_TTL_SECS as i64 + 1);
}

#[tokio::test]
async fn auth_session_expired_browser_session_evicted_on_lookup() {
    let reg = empty_registry();
    let id = issue_browser_session_with_ttl(&reg, Duration::from_millis(1)).await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(lookup_browser_session(&reg, &id).await.is_none(), "expired session not returned");
    // Evicted from registry as a side-effect.
    assert!(reg.lock().await.get(&id).is_none(), "evicted from registry");
}

#[tokio::test]
async fn auth_session_missing_browser_session_returns_none() {
    let reg = empty_registry();
    assert!(lookup_browser_session(&reg, "ship_browser_nosuchid").await.is_none());
}

// ── DaemonSession::is_expired ─────────────────────────────────────────────────

#[test]
fn auth_session_is_expired_no_expiry_never_expires() {
    let s = DaemonSession {
        token: "t".to_string(),
        agent_id: "a".to_string(),
        scopes: vec![],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: None,
        ephemeral: true,
        is_operator_cred: false,
    };
    assert!(!s.is_expired());
}

#[test]
fn auth_session_is_expired_future_expiry_not_expired() {
    let s = DaemonSession {
        expires_at_unix: Some(chrono::Utc::now().timestamp() + 3600),
        token: "t".to_string(),
        agent_id: "a".to_string(),
        scopes: vec![],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        ephemeral: false,
        is_operator_cred: false,
    };
    assert!(!s.is_expired());
}

#[test]
fn auth_session_is_expired_past_expiry_is_expired() {
    let s = DaemonSession {
        expires_at_unix: Some(chrono::Utc::now().timestamp() - 1),
        token: "t".to_string(),
        agent_id: "a".to_string(),
        scopes: vec![],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        ephemeral: false,
        is_operator_cred: false,
    };
    assert!(s.is_expired());
}

// ── Event-emit: persist_session ───────────────────────────────────────────────

#[test]
fn auth_session_persist_emits_auth_session_created() {
    let tmp = tempdir().unwrap();
    init_auth(tmp.path()).unwrap();

    let token = generate_session_token().unwrap();
    let session = persisted_session(&token);
    persist_session(&session);

    let events = query_events(AUTH_SESSION_CREATED);
    assert!(
        events.iter().any(|e| e.entity_id == token),
        "expected auth.session.created with entity_id={token}; got {events:?}"
    );
}

#[test]
fn auth_session_persist_skips_ephemeral_session() {
    let tmp = tempdir().unwrap();
    init_auth(tmp.path()).unwrap();

    let pre = query_events(AUTH_SESSION_CREATED).len();

    let mut s = persisted_session("ship_sess_ephemeral");
    s.ephemeral = true;
    persist_session(&s);

    let post = query_events(AUTH_SESSION_CREATED).len();
    assert_eq!(pre, post, "ephemeral session must not emit an event");
}

// ── Event-emit: remove_persisted_session ──────────────────────────────────────

#[test]
fn auth_session_revoke_emits_auth_session_revoked() {
    let tmp = tempdir().unwrap();
    init_auth(tmp.path()).unwrap();

    let token = generate_session_token().unwrap();
    // Persist first so the revoke has a meaningful entity.
    persist_session(&persisted_session(&token));
    remove_persisted_session(&token);

    let events = query_events(AUTH_SESSION_REVOKED);
    assert!(
        events.iter().any(|e| e.entity_id == token),
        "expected auth.session.revoked with entity_id={token}; got {events:?}"
    );
}
