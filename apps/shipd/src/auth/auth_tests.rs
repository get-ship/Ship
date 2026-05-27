//! Auth hardening tests — enforce tightened auth policy.
//!
//! These tests define the target behavior for the auth middleware after
//! hardening. They are expected to FAIL on current code.

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
fn test_router(token: &str) -> Router {
    let token = Arc::new(token.to_string());
    let registry: SessionRegistry = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::new(),
    ));
    Router::new()
        .route("/test", axum::routing::get(ok_handler))
        .layer(middleware::from_fn(move |req, next| {
            auth::auth_middleware(token.clone(), registry.clone(), req, next)
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

// 64-char lowercase hex — a well-formed daemon token (`is_daemon_token`).
const TEST_TOKEN: &str = "a01b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abc";

// ── Pid-bound session token enforcement (space window) ───────────────────

use crate::auth::peer_socket::PeerPid;
use ship_platform::auth::scopes::space_window_session;

/// Router whose auth layer sees a session registry pre-seeded with `session`,
/// and whose outer layer injects `peer_pid` into request extensions (modelling
/// the UDS peer-cred layer). `peer_pid = None` models TCP (no attested pid).
async fn router_with_pid_bound_session(
    session: ship_platform::auth::scopes::DaemonSession,
    peer_pid: Option<u32>,
) -> Router {
    let token = Arc::new(TEST_TOKEN.to_string());
    let registry: SessionRegistry =
        Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    registry.lock().await.insert(session.token.clone(), session);
    Router::new()
        .route("/test", axum::routing::get(ok_handler))
        .layer(middleware::from_fn(move |req, next| {
            auth::auth_middleware(token.clone(), registry.clone(), req, next)
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

// ── Localhost exemption must be dead ─────────────────────────────────────

#[tokio::test]
async fn request_without_token_returns_401() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "localhost origin without token must be rejected");
}

#[tokio::test]
async fn request_without_token_no_origin_returns_401() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "no token and no origin must be rejected");
}

#[tokio::test]
async fn request_with_valid_daemon_token_returns_200() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", format!("Bearer {TEST_TOKEN}"))
        .body(Body::empty())
        .unwrap();

    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::OK);
}

// ── CallerKind derived from token, not header ────────────────────────────

#[tokio::test]
async fn caller_kind_ignores_header_when_daemon_token() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", format!("Bearer {TEST_TOKEN}"))
        .header("x-ship-caller", "mcp")
        .body(Body::empty())
        .unwrap();

    let (status, body) = send(router, req).await;
    assert_eq!(status, StatusCode::OK);
    // Daemon token = local CLI caller. The header claim of "mcp" must be ignored.
    assert_eq!(body, "Cli", "daemon token must always resolve to CallerKind::Cli");
}

#[tokio::test]
async fn runtime_caller_kind_never_accepted() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", format!("Bearer {TEST_TOKEN}"))
        .header("x-ship-caller", "runtime")
        .body(Body::empty())
        .unwrap();

    let (status, body) = send(router, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(body, "Runtime", "external callers must never get CallerKind::Runtime");
}

// ── Dual token support ──────────────────────────────────────────────────

#[test]
fn daemon_token_format_is_64_hex_chars() {
    assert!(is_daemon_token(
        "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
    ));
}

#[test]
fn session_token_format_is_not_daemon_token() {
    assert!(!is_daemon_token("ship_session_abc123"));
}

/// Helper that the implementation must provide: distinguish daemon tokens
/// (64-char hex) from session tokens (ship_ prefix).
fn is_daemon_token(token: &str) -> bool {
    token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit())
}

// ── ship_hook_ tokens must be rejected ──────────────────────────────────

#[tokio::test]
async fn ship_hook_prefixed_token_returns_401() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", "Bearer ship_hook_anything")
        .body(Body::empty())
        .unwrap();

    let (status, _) = send(router, req).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "hook-token class is removed; ship_hook_ prefix must be rejected"
    );
}

// ── Cookie-based session auth (phase 1) ─────────────────────────────────

use crate::auth::session;

/// Valid 64-hex daemon token (distinct from TEST_TOKEN above which intentionally
/// contains a non-hex `]`).
const VALID_DAEMON_TOKEN: &str =
    "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2";

fn router_with_session(token: &str) -> (Router, SessionRegistry) {
    let token = Arc::new(token.to_string());
    let registry: SessionRegistry = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::new(),
    ));
    let reg_for_endpoint = registry.clone();
    let router = Router::new()
        .route("/test", axum::routing::get(ok_handler))
        .route(
                "/api/v1/session",
            axum::routing::post(move |auth: axum::Extension<AuthContext>| {
                let reg = reg_for_endpoint.clone();
                async move { session::issue_session_handler(reg, auth.0).await }
            }),
        )
        .layer(middleware::from_fn({
            let registry = registry.clone();
            move |req, next| auth::auth_middleware(token.clone(), registry.clone(), req, next)
        }));
    (router, registry)
}

#[tokio::test]
async fn session_endpoint_issues_cookie_with_valid_daemon_token() {
    let (router, _reg) = router_with_session(VALID_DAEMON_TOKEN);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/session")
        .header("authorization", format!("Bearer {VALID_DAEMON_TOKEN}"))
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let cookie = resp
        .headers()
        .get("set-cookie")
        .expect("set-cookie header present")
        .to_str()
        .unwrap()
        .to_string();
    assert!(cookie.contains("ship_session="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));
    assert!(cookie.contains("Path=/api"));
    assert!(cookie.contains("Max-Age=86400"));
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["ok"], serde_json::json!(true));
    assert_eq!(parsed["expires_in"], serde_json::json!(86400));
}

#[tokio::test]
async fn session_endpoint_rejects_missing_auth() {
    let (router, _reg) = router_with_session(VALID_DAEMON_TOKEN);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/session")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_accepts_valid_session_cookie() {
    let (router, reg) = router_with_session(VALID_DAEMON_TOKEN);
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
    let (router, reg) = router_with_session(VALID_DAEMON_TOKEN);
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
async fn auth_middleware_falls_through_to_bearer_when_cookie_missing() {
    let (router, _reg) = router_with_session(VALID_DAEMON_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", format!("Bearer {VALID_DAEMON_TOKEN}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn query_token_rejected_on_non_raw_paths() {
    // `?token=` is only accepted for the iframe `/raw` flow. Everywhere
    // else it must be ignored so tokens don't leak into access logs.
    let (router, _reg) = router_with_session(VALID_DAEMON_TOKEN);
    let req = Request::builder()
        .uri(format!("/test?token={VALID_DAEMON_TOKEN}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn query_token_accepted_on_raw_view_path() {
    let token = Arc::new(VALID_DAEMON_TOKEN.to_string());
    let registry: SessionRegistry = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::new(),
    ));
    let router = Router::new()
        .route(
                "/api/v1/view/{name}/raw",
            axum::routing::get(ok_handler),
        )
        .layer(middleware::from_fn({
            let registry = registry.clone();
            move |req, next| auth::auth_middleware(token.clone(), registry.clone(), req, next)
        }));
    let req = Request::builder()
        .uri(format!("/api/v1/view/pipelines/raw?token={VALID_DAEMON_TOKEN}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn anomalous_token_format_fails_closed() {
    // A token that equals `expected_token` but is neither the 64-hex daemon
    // format nor an SDK prefix must be rejected, not granted Cli + Full. The
    // pre-hardening catch-all returned 200; fail-closed returns 401.
    let weird = "not-hex-not-sdk-prefix";
    let router = test_router(weird);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", format!("Bearer {weird}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "anomalous credential format must fail closed");
}
