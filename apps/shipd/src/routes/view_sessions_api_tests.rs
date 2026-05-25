//! Integration tests for the view-session REST routes.

use axum::{Router, body::Body, http::Request, response::IntoResponse};
use http_body_util::BodyExt;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;

use crate::auth::AuthContext;
use crate::routes::rest_api::ApiState;
use crate::ship_runtime::ShipRuntime;

fn router_for(state: ApiState, auth: AuthContext) -> Router {
    Router::new()
        .route(
                "/api/v1/view-sessions",
            axum::routing::post(super::open_session),
        )
        .route(
                "/api/v1/view-sessions/{id}",
            axum::routing::get(super::subscribe_session).delete(super::close_session),
        )
        .route(
                "/api/v1/view-sessions/{id}/patch",
            axum::routing::post(super::patch_session),
        )
        .route(
                "/api/v1/view-sessions/{id}/append",
            axum::routing::post(super::append_session),
        )
        .route(
                "/api/v1/view-sessions/{id}/signal",
            axum::routing::post(super::signal_session),
        )
        .with_state(state)
        .layer(axum::Extension(auth))
}

fn fresh_state() -> (ApiState, TempDir) {
    let dir = TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let state: ApiState = Arc::new(ShipRuntime::new_test(ship_dir, kernel));
    (state, dir)
}

fn mcp_auth(agent: &str) -> AuthContext {
    AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: Some(agent.into()),
        workspace_id: None,
        session_id: None,
        scopes: vec![crate::auth::Scope::Full],
        grants: vec![],
    }
}

async fn send_json(router: &Router, method: &str, path: &str, body: serde_json::Value) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
    (status, json)
}

async fn send_empty(router: &Router, method: &str, path: &str) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!(null));
    (status, json)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_surfaces_mcp_apps_meta() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, mcp_auth("agent.owner"));
    let (status, body) = send_json(
        &router,
        "POST",
        "/api/v1/view-sessions",
        serde_json::json!({"template": "canvas/blank"}),
    )
    .await;
    assert_eq!(status, 200);
    let view_id = body["data"]["view_id"].as_str().unwrap();
    assert_eq!(
        body["data"]["_meta"]["ui"]["resourceUri"],
        serde_json::json!(format!("ui://ship/view-session/{view_id}"))
    );
    assert_eq!(body["data"]["_meta"]["ui"]["csp"]["script-src"][0], "'self'");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_then_patch_then_close_succeeds() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, mcp_auth("agent.owner"));
    let (status, body) = send_json(
        &router,
        "POST",
        "/api/v1/view-sessions",
        serde_json::json!({ "template": "canvas/blank" }),
    )
    .await;
    assert_eq!(status, 200, "open returned {body}");
    let view_id = body["data"]["view_id"].as_str().unwrap().to_string();
    assert!(body["data"]["url"].as_str().unwrap().contains(&view_id));

    let (status, _) = send_json(
        &router,
        "POST",
        &format!("/api/v1/view-sessions/{view_id}/patch"),
        serde_json::json!({"selector": "#root", "html": "<p>hi</p>"}),
    )
    .await;
    assert_eq!(status, 200);

    let (status, _) = send_empty(
        &router,
        "DELETE",
        &format!("/api/v1/view-sessions/{view_id}"),
    )
    .await;
    assert_eq!(status, 200);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_owner_patch_returns_403() {
    let (state, _tmp) = fresh_state();
    // Open as owner, then swap auth to a different agent for patch.
    let router_owner = router_for(state.clone(), mcp_auth("agent.owner"));
    let (_, body) = send_json(
        &router_owner,
        "POST",
        "/api/v1/view-sessions",
        serde_json::json!({"template": "canvas/blank"}),
    )
    .await;
    let view_id = body["data"]["view_id"].as_str().unwrap().to_string();

    let router_intruder = router_for(state, mcp_auth("agent.other"));
    let (status, body) = send_json(
        &router_intruder,
        "POST",
        &format!("/api/v1/view-sessions/{view_id}/patch"),
        serde_json::json!({"selector": "#root", "html": "<p/>"}),
    )
    .await;
    assert_eq!(status, 403, "body: {body}");
    assert_eq!(body["error"]["code"], "not_owner");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_without_agent_is_unauthenticated() {
    let (state, _tmp) = fresh_state();
    let anon = AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: None,
        workspace_id: None,
        session_id: None,
        scopes: vec![crate::auth::Scope::Full],
        grants: vec![],
    };
    let router = router_for(state, anon);
    let (status, body) = send_json(
        &router,
        "POST",
        "/api/v1/view-sessions",
        serde_json::json!({"template": "canvas/blank"}),
    )
    .await;
    assert_eq!(status, 401);
    assert_eq!(body["error"]["code"], "unauthenticated");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn quota_returns_structured_conflict() {
    let (state, _tmp) = fresh_state();
    // Shrink the quota for the test by reaching into the underlying registry.
    // The test state uses default (3) — open 3 then expect 4th to fail.
    let router = router_for(state.clone(), mcp_auth("agent.owner"));
    for _ in 0..3 {
        let (s, _) = send_json(
            &router,
            "POST",
            "/api/v1/view-sessions",
            serde_json::json!({"template": "canvas/blank"}),
        )
        .await;
        assert_eq!(s, 200);
    }
    let (status, body) = send_json(
        &router,
        "POST",
        "/api/v1/view-sessions",
        serde_json::json!({"template": "canvas/blank"}),
    )
    .await;
    assert_eq!(status, 409);
    assert_eq!(body["error"]["code"], "quota_exceeded");
    assert_eq!(body["error"]["limit"], 3);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscribe_receives_broadcast_patches() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state.clone(), mcp_auth("agent.owner"));
    let (_, body) = send_json(
        &router,
        "POST",
        "/api/v1/view-sessions",
        serde_json::json!({"template": "canvas/blank"}),
    )
    .await;
    let view_id = body["data"]["view_id"].as_str().unwrap().to_string();

    // Subscribe by calling the registry directly (same Arc as the router's
    // state) so we don't have to fight axum 0.8's lack of state-extraction
    // from a built Router.
    let sse = state.view_sessions().subscribe(&view_id).await.unwrap();
    let handle = tokio::spawn(async move {
        let bytes = sse.into_response().into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    });
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let (s, _) = send_json(
        &router,
        "POST",
        &format!("/api/v1/view-sessions/{view_id}/patch"),
        serde_json::json!({"selector": "#root", "html": "<p>beacon-xyz</p>"}),
    )
    .await;
    assert_eq!(s, 200);

    let (s, _) = send_empty(
        &router,
        "DELETE",
        &format!("/api/v1/view-sessions/{view_id}"),
    )
    .await;
    assert_eq!(s, 200);

    let body = handle.await.unwrap();
    assert!(body.contains("beacon-xyz"), "SSE body did not contain patch: {body}");
}
