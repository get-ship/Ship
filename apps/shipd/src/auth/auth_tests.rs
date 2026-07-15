//! Auth hardening tests — enforce tightened auth policy.
//!
//! These tests pin the post-cut auth middleware behavior: the host
//! unix socket's peer-cred layer populates `AuthContext` ahead of the
//! middleware, session tokens authenticate against the session
//! registry with optional pid binding, browser cookies authenticate
//! without elevation, the loopback bypass is gated on
//! `--unsafe-skip-auth`, and requests without any of these
//! credentials are rejected with 401.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware;
use axum::response::IntoResponse;
use axum::Router;
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

use crate::auth::{self, AuthContext, SessionRegistry};

/// Build a minimal router with the auth middleware and a 200-OK handler.
fn test_router() -> Router {
    let registry: SessionRegistry = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::new(),
    ));
    Router::new()
        .route("/test", axum::routing::get(ok_handler))
        .layer(middleware::from_fn(move |req, next| {
            auth::auth_middleware(registry.clone(), req, next)
        }))
}

async fn ok_handler(axum::Extension(auth): axum::Extension<AuthContext>) -> impl IntoResponse {
    // Return the caller_kind as the body so tests can inspect it.
    let kind = format!("{:?}", auth.caller_kind);
    (StatusCode::OK, kind)
}

async fn send(router: Router, req: Request<Body>) -> (StatusCode, String) {
    let resp = router.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8_lossy(&bytes).to_string();
    (status, body)
}

// ── Pid-bound session token enforcement (space window) ───────────────────

use crate::auth::peer_socket::PeerPid;
use ship_auth::scopes::{operator_session, space_window_session};

/// Handler that reports whether the resolved caller is operator-class — the S6
/// end-to-end probe (the operator cred is the SOLE elevation source).
async fn operator_probe(axum::Extension(auth): axum::Extension<AuthContext>) -> impl IntoResponse {
    (StatusCode::OK, if auth.is_operator() { "operator" } else { "not-operator" })
}

/// Router serving `/whoami` (operator probe) behind the real bearer middleware,
/// pre-seeded with `session` and injecting `peer_pid` like the UDS peer-cred layer.
async fn operator_router(
    session: ship_auth::scopes::DaemonSession,
    peer_pid: Option<u32>,
) -> Router {
    let registry: SessionRegistry =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    registry.lock().await.insert(session.token.clone(), session);
    Router::new()
        .route("/whoami", axum::routing::get(operator_probe))
        .layer(middleware::from_fn(move |req, next| {
            auth::auth_middleware(registry.clone(), req, next)
        }))
        .layer(middleware::from_fn(move |mut req: Request<Body>, next: middleware::Next| async move {
            req.extensions_mut().insert(PeerPid(peer_pid));
            next.run(req).await
        }))
}

#[tokio::test]
async fn operator_cred_over_attested_transport_is_operator() {
    // The pid-attested operator cred presented from its bound process is the
    // operator — the SOLE elevation source (S6).
    let session = operator_session(4242, "ship_sess_op".to_string());
    let router = operator_router(session, Some(4242)).await;
    let req = Request::builder()
        .uri("/whoami")
        .header("authorization", "Bearer ship_sess_op")
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(router, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "operator", "a pid-attested operator cred is operator-class");
}

#[tokio::test]
async fn operator_cred_over_unattested_transport_is_rejected() {
    // The operator cred is pid-bound: presented over a transport that can't
    // attest a pid (TCP, no PeerPid) it fails closed — the operator binding
    // can't be stripped by switching transports.
    let session = operator_session(4242, "ship_sess_op".to_string());
    let router = operator_router(session, None).await;
    let req = Request::builder()
        .uri("/whoami")
        .header("authorization", "Bearer ship_sess_op")
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "an unattested transport must not present an operator cred"
    );
}

/// Router whose auth layer sees a session registry pre-seeded with `session`,
/// and whose outer layer injects `peer_pid` into request extensions (modelling
/// the UDS peer-cred layer). `peer_pid = None` models TCP (no attested pid).
async fn router_with_pid_bound_session(
    session: ship_auth::scopes::DaemonSession,
    peer_pid: Option<u32>,
) -> Router {
    let registry: SessionRegistry =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    registry.lock().await.insert(session.token.clone(), session);
    Router::new()
        .route("/test", axum::routing::get(ok_handler))
        .layer(middleware::from_fn(move |req, next| {
            auth::auth_middleware(registry.clone(), req, next)
        }))
        // Outermost: runs first, inserts the kernel pid like the peer-cred layer.
        .layer(middleware::from_fn(move |mut req: Request<Body>, next: middleware::Next| async move {
            req.extensions_mut().insert(PeerPid(peer_pid));
            next.run(req).await
        }))
}

#[tokio::test]
async fn pid_bound_session_allows_matching_process() {
    let session = space_window_session("sp-a", 4242, "ship_sess_winA".to_string());
    let router = router_with_pid_bound_session(session, Some(4242)).await;
    let req = Request::builder()
        .uri("/test")
        .header("authorization", "Bearer ship_sess_winA")
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::OK, "matching pid is authorized");
}

#[tokio::test]
async fn pid_bound_session_rejects_mismatched_process() {
    let session = space_window_session("sp-a", 4242, "ship_sess_winA".to_string());
    let router = router_with_pid_bound_session(session, Some(9999)).await;
    let req = Request::builder()
        .uri("/test")
        .header("authorization", "Bearer ship_sess_winA")
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "a different process is rejected (stolen token)");
}

#[tokio::test]
async fn pid_bound_session_fails_closed_without_peer_pid() {
    // Models TCP: no peer-cred layer ran, so no PeerPid was injected.
    let session = space_window_session("sp-a", 4242, "ship_sess_winA".to_string());
    let router = router_with_pid_bound_session(session, None).await;
    let req = Request::builder()
        .uri("/test")
        .header("authorization", "Bearer ship_sess_winA")
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "a bound token over an unattested transport fails closed");
}

// ── Unauthenticated requests are rejected ────────────────────────────────

#[tokio::test]
async fn request_without_token_returns_401() {
    let router = test_router();
    let req = Request::builder()
        .uri("/test")
        .header("origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "no token must be rejected");
}

// ── Cookie-based session auth ────────────────────────────────────────────

use crate::auth::session;

fn router_with_session() -> (Router, SessionRegistry) {
    let registry: SessionRegistry = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::new(),
    ));
    let router = Router::new()
        .route("/test", axum::routing::get(ok_handler))
        .layer(middleware::from_fn({
            let registry = registry.clone();
            move |req, next| auth::auth_middleware(registry.clone(), req, next)
        }));
    (router, registry)
}

#[tokio::test]
async fn auth_middleware_accepts_valid_session_cookie() {
    let (router, reg) = router_with_session();
    let id = session::issue_browser_session(&reg).await;
    let req = Request::builder()
        .uri("/test")
        .header("cookie", format!("ship_session={id}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(router, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "Cli");
}

#[tokio::test]
async fn auth_middleware_rejects_expired_session_cookie() {
    let (router, reg) = router_with_session();
    let id = session::issue_browser_session_with_ttl(
        &reg,
        std::time::Duration::from_millis(1),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    let req = Request::builder()
        .uri("/test")
        .header("cookie", format!("ship_session={id}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn query_token_rejected_on_non_raw_paths() {
    // `?token=` is no longer accepted anywhere. Every endpoint
    // authenticates via the `Authorization: Bearer` header, the
    // `ship_session` cookie, or peer-cred. A `?token=` is ignored.
    let (router, _reg) = router_with_session();
    let req = Request::builder()
        .uri("/test?token=ignored")
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Session is the sole binding minter (Phase 2 / #196) ───────────────────

use ship_substrate::events::CallerKind;
use ship_types::{Authority, SpaceId, event_registry::EventScope, local_device_id};

/// T1 — a space-window session mints the correct binding: its OWN space (not a
/// process-global), `Authority::Space`, the placeholder device id, no account.
#[test]
fn space_window_session_mints_space_binding() {
    let session = space_window_session("space-A", 4242, "ship_sess_x".to_string());
    let auth = AuthContext {
        caller_kind: CallerKind::Mcp,
        agent_id: Some(session.agent_id.clone()),
        space_id: session.space_id.clone(),
        session_id: None,
        grants: session.grants.clone(),
        // A space window is C2-capped — never the operator cred.
        operator_cred: session.is_operator_cred,
    };
    let ctx = auth.to_emit_context(CallerKind::Mcp);
    assert_eq!(ctx.binding.space_id(), Some(&SpaceId::new("space-A")));
    assert_eq!(
        *ctx.binding.authority(),
        Authority::Space { space_id: SpaceId::new("space-A") }
    );
    assert_eq!(ctx.binding.device_id(), &local_device_id());
    assert_eq!(ctx.binding.account_id(), None);
    assert_eq!(ctx.binding.key_for(EventScope::Space).as_deref(), Some("space-A"));
}

/// T2 — operator authority derives SOLELY from a verified operator cred (S6,
/// #124), NOT from the CLI transport. A verified operator cred → Operator; a
/// bare CLI connection (no cred) → NOT operator; an SDK token → Sdk, never
/// Operator (#207).
#[test]
fn operator_authority_requires_a_verified_cred_not_the_cli_transport() {
    // The verified operator cred — the SOLE elevation source.
    let operator = AuthContext {
        caller_kind: CallerKind::Cli,
        agent_id: None,
        space_id: None,
        session_id: None,
        grants: vec![],
        operator_cred: true,
    };
    assert!(operator.is_operator(), "a verified operator cred is operator-class");

    // The CLI transport WITHOUT a verified operator cred grants NO elevation —
    // this is the S6 kill: CLI != operator.
    let bare_cli = AuthContext {
        caller_kind: CallerKind::Cli,
        agent_id: None,
        space_id: None,
        session_id: None,
        grants: vec![],
        operator_cred: false,
    };
    assert!(
        !bare_cli.is_operator(),
        "the CLI transport must NOT confer operator (S6)"
    );

    let sdk = AuthContext {
        caller_kind: CallerKind::Sdk,
        agent_id: None,
        space_id: None,
        session_id: None,
        grants: vec![],
        operator_cred: false,
    };
    assert!(!sdk.is_operator(), "SDK token must NEVER be operator-class (#207)");
    assert_eq!(
        *sdk.to_emit_context(CallerKind::Sdk).binding.authority(),
        Authority::Sdk
    );
    // Intended P2 fail-soft: operator with no session space keys None (no
    // home-space fallback on the emit path).
    assert_eq!(operator.to_emit_context(CallerKind::Cli).binding.key_for(EventScope::Space), None);
}
