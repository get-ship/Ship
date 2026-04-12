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
    let hook_token = Arc::new("ship_hook_test".to_string());
    let registry: SessionRegistry = Arc::new(tokio::sync::Mutex::new(
        std::collections::HashMap::new(),
    ));
    Router::new()
        .route("/test", axum::routing::get(ok_handler))
        .layer(middleware::from_fn(move |req, next| {
            auth::auth_middleware(token.clone(), hook_token.clone(), registry.clone(), req, next)
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

const TEST_TOKEN: &str = "a]1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6abc";

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

// ── Hook token tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn valid_hook_token_returns_200() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", "Bearer ship_hook_test")
        .body(Body::empty())
        .unwrap();

    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::OK, "valid hook token must be accepted");
}

#[tokio::test]
async fn wrong_hook_token_returns_401() {
    let router = test_router(TEST_TOKEN);
    let req = Request::builder()
        .uri("/test")
        .header("authorization", "Bearer ship_hook_wrong")
        .body(Body::empty())
        .unwrap();

    let (status, _) = send(router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "invalid hook token must be rejected");
}

#[test]
fn hook_token_roundtrips() {
    let dir = tempfile::TempDir::new().unwrap();
    let token = auth::generate_hook_token().unwrap();
    auth::write_hook_token(dir.path(), &token).unwrap();
    let read = auth::read_hook_token(dir.path()).unwrap();
    assert_eq!(read, token);
    assert!(auth::is_hook_token(&token));
}

#[test]
fn ensure_hook_token_generates_if_missing() {
    let dir = tempfile::TempDir::new().unwrap();
    let token1 = auth::ensure_hook_token(dir.path()).unwrap();
    let token2 = auth::ensure_hook_token(dir.path()).unwrap();
    assert_eq!(token1, token2, "ensure_hook_token must be idempotent");
    assert!(auth::is_hook_token(&token1));
}
