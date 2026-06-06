//! Tests for the REST API non-mesh endpoints — capabilities, CORS, hooks.
//!
//! The mesh.* op surface tests were removed with the mesh.*/actor.* op cut.

use crate::test_harness::TestDaemon;

// ── capabilities endpoint ──────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn capabilities_endpoint_exists() {
    let daemon = TestDaemon::new().await;
    let resp = daemon.get("/api/v1/capabilities").await;
    assert_eq!(resp.status, 200, "GET /api/v1/capabilities must return 200");
    assert!(resp.body.get("authenticated").is_some(), "response must contain 'authenticated'");
    assert!(resp.body.get("mode").is_some(), "response must contain 'mode'");
    assert!(resp.body.get("tier").is_some(), "response must contain 'tier'");
}

#[tokio::test(flavor = "multi_thread")]
async fn capabilities_returns_local_mode_with_daemon_token() {
    let daemon = TestDaemon::new().await;
    let resp = daemon.get("/api/v1/capabilities").await;
    assert_eq!(resp.status, 200);
    assert_eq!(resp.body["authenticated"], true);
    assert_eq!(resp.body["mode"], "local");
}

#[tokio::test(flavor = "multi_thread")]
async fn capabilities_returns_unauthenticated_without_token() {
    // A browser request from localhost with no token must be rejected.
    use tempfile::TempDir;
    let temp_dir = TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(temp_dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();

    let token = crate::auth::generate_token().unwrap();
    let token_arc = std::sync::Arc::new(token);

    let registry: crate::auth::SessionRegistry = std::sync::Arc::new(
        tokio::sync::Mutex::new(std::collections::HashMap::new()),
    );
    let router = axum::Router::new()
        .route("/api/v1/capabilities", axum::routing::get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(move |req, next| {
            crate::auth::auth_middleware(token_arc.clone(), registry.clone(), req, next)
        }));

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/capabilities")
        .header("origin", "http://localhost:3000")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(
        resp.status().as_u16(),
        401,
        "localhost browser request without token must return 401"
    );
}

// ── CORS enforcement ──────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn cors_allows_configured_origins() {
    let router = crate::test_cors_router();
    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/test")
        .header("origin", "https://getship.dev")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    let acao = resp
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok());
    assert_eq!(acao, Some("https://getship.dev"));
}

#[tokio::test(flavor = "multi_thread")]
async fn cors_rejects_unknown_origin() {
    let router = crate::test_cors_router();
    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/test")
        .header("origin", "https://evil.com")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    let acao = resp
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        acao.is_empty() || acao != "https://evil.com",
        "CORS must not allow evil.com"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn cors_allows_localhost_dev() {
    let router = crate::test_cors_router();
    let req = axum::http::Request::builder()
        .method("OPTIONS")
        .uri("/test")
        .header("origin", "http://localhost:3000")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = tower::ServiceExt::oneshot(router, req).await.unwrap();
    let acao = resp
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok());
    assert_eq!(acao, Some("http://localhost:3000"));
}
