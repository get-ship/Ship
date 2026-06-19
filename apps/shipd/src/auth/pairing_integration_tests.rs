//! Integration tests for the pairing flow.
//!
//! Exercises the full init → confirm → status chain through the handler
//! functions, asserting that:
//!   - Session tokens come back with Full scope
//!   - Codes are single-use
//!   - Claim tokens are single-use
//!   - Paired sessions persist across restarts via KV

use crate::auth;
use crate::auth::{AuthContext, Scope};
use crate::auth::pairing;
use crate::auth::scopes::{DaemonSession, SessionRegistry};
use ship_substrate::events::CallerKind;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

 fn setup_ship_dir() {
     // `get_global_dir()` auto-isolates per test binary (see runtime::project).
     // The kernel router ensures its own scope dbs on first use.
     let _ = ship_substrate::project::get_global_dir();
 }

fn session_registry() -> SessionRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

fn full_auth() -> AuthContext {
    AuthContext {
        caller_kind: CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
        grants: vec![],
        // pair_confirm gates on Scope::Full (the daemon-token bootstrap), not the
        // operator path — no operator cred required.
        operator_cred: false,
    }
}

fn no_auth() -> AuthContext {
    AuthContext {
        caller_kind: CallerKind::Mcp,
        agent_id: None,
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![Scope::ReadWorkspace],
        grants: vec![],
        operator_cred: false,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn pair_init_confirm_status_roundtrip() {
    setup_ship_dir();
    let reg = pairing::new_shared();
    let sessions = session_registry();

    // 1. Init
    let init = auth::pair_init(reg.clone(), sessions.clone(), None, None)
        .await
        .unwrap();
    let code = init.0.code.clone();
    let claim = init.0.claim_token.clone();
    assert!(!init.0.dev_mode);
    assert!(code.contains('-'), "code should be formatted XXX-XXX");

    // 2. Status before confirm → pending
    use axum::extract::Query;
    let pending = auth::pair_status(
        reg.clone(),
        Query(auth::PairStatusQuery { claim_token: claim.clone() }),
    )
    .await;
    assert!(matches!(pending.0, auth::PairStatusResponse::Pending));

    // 3. Confirm with daemon-token auth
    let (status, resp) = auth::pair_confirm(
        reg.clone(),
        sessions.clone(),
        axum::Extension(full_auth()),
        axum::Json(auth::PairConfirmRequest { code: code.clone() }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(resp.0.ok);

    // 4. Status after confirm → approved with session_token
    let approved = auth::pair_status(
        reg.clone(),
        Query(auth::PairStatusQuery { claim_token: claim.clone() }),
    )
    .await;
    let session_token = match approved.0 {
        auth::PairStatusResponse::Approved { session_token } => session_token,
        other => panic!("expected approved, got {other:?}"),
    };
    assert!(session_token.starts_with("ship_sess_"));

    // 5. Session registry contains the new session with Full scope
    let reg_lock = sessions.lock().await;
    let session = reg_lock
        .get(&session_token)
        .expect("session must be in registry");
    assert_eq!(session.agent_id, "studio");
    assert!(session.scopes.contains(&Scope::Full));
    assert!(!session.ephemeral, "paired sessions must be persistent");
    drop(reg_lock);

    // 6. Second claim returns Unknown (single-use)
    let second_claim = auth::pair_status(
        reg.clone(),
        Query(auth::PairStatusQuery { claim_token: claim }),
    )
    .await;
    assert!(matches!(second_claim.0, auth::PairStatusResponse::Unknown));
}

#[tokio::test(flavor = "multi_thread")]
async fn pair_confirm_rejects_non_full_scope() {
    setup_ship_dir();
    let reg = pairing::new_shared();
    let sessions = session_registry();

    let init = auth::pair_init(reg.clone(), sessions.clone(), None, None)
        .await
        .unwrap();

    let (status, _resp) = auth::pair_confirm(
        reg.clone(),
        sessions.clone(),
        axum::Extension(no_auth()),
        axum::Json(auth::PairConfirmRequest { code: init.0.code.clone() }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn pair_confirm_unknown_code_returns_404() {
    setup_ship_dir();
    let reg = pairing::new_shared();
    let sessions = session_registry();

    let (status, _resp) = auth::pair_confirm(
        reg.clone(),
        sessions.clone(),
        axum::Extension(full_auth()),
        axum::Json(auth::PairConfirmRequest {
            code: "NOT-REAL".to_string(),
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn pair_code_is_single_use() {
    setup_ship_dir();
    let reg = pairing::new_shared();
    let sessions = session_registry();

    let init = auth::pair_init(reg.clone(), sessions.clone(), None, None)
        .await
        .unwrap();
    let code = init.0.code.clone();

    // First confirm succeeds.
    let (s1, _) = auth::pair_confirm(
        reg.clone(),
        sessions.clone(),
        axum::Extension(full_auth()),
        axum::Json(auth::PairConfirmRequest { code: code.clone() }),
    )
    .await;
    assert_eq!(s1, axum::http::StatusCode::OK);

    // Second confirm with the same code fails.
    let (s2, _) = auth::pair_confirm(
        reg.clone(),
        sessions.clone(),
        axum::Extension(full_auth()),
        axum::Json(auth::PairConfirmRequest { code }),
    )
    .await;
    assert_eq!(s2, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test(flavor = "multi_thread")]
async fn persisted_session_survives_hydrate() {
    setup_ship_dir();

    // Insert a persisted session directly (simulating a prior daemon run).
    let session = DaemonSession {
        token: "ship_sess_abc123".to_string(),
        agent_id: "studio".to_string(),
        scopes: vec![Scope::Full],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: Some(chrono::Utc::now().timestamp() + 3600),
        ephemeral: false,
        is_operator_cred: false,
    };
    auth::persist_session(&session);

    // New registry, simulating daemon restart.
    let fresh = session_registry();
    auth::hydrate_sessions(&fresh).await;

    let loaded = fresh.lock().await;
    assert!(
        loaded.contains_key("ship_sess_abc123"),
        "session must hydrate from KV"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn expired_session_is_purged_on_hydrate() {
    setup_ship_dir();

    // Expired session.
    let session = DaemonSession {
        token: "ship_sess_expired".to_string(),
        agent_id: "studio".to_string(),
        scopes: vec![Scope::Full],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: Some(chrono::Utc::now().timestamp() - 3600),
        ephemeral: false,
        is_operator_cred: false,
    };
    auth::persist_session(&session);

    let fresh = session_registry();
    auth::hydrate_sessions(&fresh).await;

    let loaded = fresh.lock().await;
    assert!(
        !loaded.contains_key("ship_sess_expired"),
        "expired session must not be loaded"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn ephemeral_session_not_persisted() {
    setup_ship_dir();

    let session = DaemonSession {
        token: "ship_sess_ephemeral".to_string(),
        agent_id: "studio-dev".to_string(),
        scopes: vec![Scope::Full],
        grants: vec![],
        workspace_id: None,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: None,
        ephemeral: true,
        is_operator_cred: false,
    };
    auth::persist_session(&session);

    let fresh = session_registry();
    auth::hydrate_sessions(&fresh).await;

    let loaded = fresh.lock().await;
    assert!(
        !loaded.contains_key("ship_sess_ephemeral"),
        "ephemeral sessions must not be persisted"
    );
}

