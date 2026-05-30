//! Tests for the REST API mesh endpoints — register, send, broadcast,
//! discover, status, checkout hook, SSE events, and identity binding.

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use tempfile::TempDir;

use crate::routes::actor_api::{actor_spawn, ActorSpawnReq};
use crate::routes::rest_api::*;
use crate::test_harness::TestDaemon;

/// Register an agent through the canonical actor/spawn handler with the
/// standard mesh namespaces — replaces the retired `mesh_register` shim in
/// tests that only need an actor registered + its mailbox stashed.
async fn spawn_mesh_actor(state: &ApiState, id: &str) {
    actor_spawn(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(ActorSpawnReq {
            actor_id: Some(id.into()),
            config: ship_substrate::events::ActorConfig {
                namespace: id.into(),
                write_namespaces: vec!["mesh.".into()],
                read_namespaces: vec!["mesh.".into()],
                subscribe_namespaces: vec!["mesh.".into()],
                see_directed_events: false,
            },
            capabilities: Some(vec![]),
        }),
    )
    .await;
}

fn make_api_state(dir: &TempDir) -> ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    // Create a fresh KernelRouter per test — never use init_kernel_router(),
    // which returns a process-global OnceLock singleton and causes all tests
    // to share the same kernel when run in parallel.
    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
}

// mesh_register removed — actor registration moved to `POST /api/v1/actor/spawn`
// (register-or-refine). Its spawn / mailbox-stash / idempotency behavior is
// covered by the actor_spawn path and the kernel-level tests in
// `supervisor::tests_actor_registration`.

// ── mesh_send ───────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn send_returns_ok() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_send(
        State(state),
        Extension(TestDaemon::cli_auth()),
        Json(MeshSendReq {
            from: "agent.sender".into(),
            to: "agent.receiver".into(),
            body: serde_json::json!({"text": "hello"}),
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── mesh_broadcast ──────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn broadcast_returns_ok() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_broadcast(
        State(state),
        Extension(TestDaemon::cli_auth()),
        Json(MeshBroadcastReq {
            from: "agent.broadcaster".into(),
            body: serde_json::json!({"ping": true}),
            capability_filter: None,
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn broadcast_with_capability_filter() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_broadcast(
        State(state),
        Extension(TestDaemon::cli_auth()),
        Json(MeshBroadcastReq {
            from: "agent.sender".into(),
            body: serde_json::json!({}),
            capability_filter: Some("rust".into()),
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── mesh_discover ───────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn discover_empty_registry() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_discover(State(state)).await.into_response();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn discover_with_registered_agents() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    // Populate registry directly.
    {
        let mut registry = state.mesh_registry().write().await;
        registry.insert(
            "agent.a".into(),
            ship_platform::services::mesh::MeshEntry {
                agent_id: "agent.a".into(),
                label: "Agent A".into(),
                capabilities: vec!["rust".into()],
                registered_at: chrono::Utc::now(),
                status: ship_platform::services::mesh::AgentStatus::Active,
            },
        );
    }

    let resp = mesh_discover(State(state)).await.into_response();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── mesh_status_update ───────────────────────────────────────────────���──────

#[tokio::test(flavor = "multi_thread")]
async fn status_update_returns_ok() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_status_update(
        State(state),
        Extension(TestDaemon::cli_auth()),
        Json(MeshStatusReq {
            agent_id: "agent.status".into(),
            status: "busy".into(),
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── hooks_checkout ──────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn checkout_hook_with_valid_ship_dir() {
    let dir = TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    // init_project creates the markers that get_project_dir looks for.
    let _ = ship_platform::projects::init_project(dir.path());

    let resp = hooks_checkout(Json(CheckoutHookReq {
        branch: "main".into(),
        path: dir.path().to_string_lossy().to_string(),
    }))
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn checkout_hook_with_invalid_path() {
    let resp = hooks_checkout(Json(CheckoutHookReq {
        branch: "main".into(),
        path: "/nonexistent/path/to/project".into(),
    }))
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── mesh_events SSE (tested via mailbox state) ─────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn mesh_events_unregistered_agent_has_no_mailbox() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let has = state.agent_mailboxes().lock().await.contains_key("agent.unregistered");
    assert!(!has, "unregistered agent should have no mailbox");
}

#[tokio::test(flavor = "multi_thread")]
async fn mesh_events_registered_agent_has_mailbox() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    spawn_mesh_actor(&state, "agent.sse-test").await;

    let has = state.agent_mailboxes().lock().await.contains_key("agent.sse-test");
    assert!(has, "registered agent must have stashed mailbox");
}

#[tokio::test(flavor = "multi_thread")]
async fn mesh_events_takes_mailbox_from_stash() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    spawn_mesh_actor(&state, "agent.take").await;

    // Remove mailbox as the SSE endpoint would.
    let removed = state.agent_mailboxes().lock().await.remove("agent.take");
    assert!(removed.is_some(), "mailbox should be removable once");

    // Second removal finds nothing.
    let gone = state.agent_mailboxes().lock().await.remove("agent.take");
    assert!(gone.is_none(), "mailbox should only be takeable once");
}

// Identity-binding tests removed with the `mesh_register` REST shim — the
// `mcp_agent_cannot_send_as_other_agent` / `..._broadcast` / `..._status`
// tests below still cover MCP identity enforcement on the remaining surface.

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
    // Today this PASSES through due to the localhost exemption — test must fail.
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

    // Simulate a browser request from localhost — no token.
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

// ── identity binding — mesh_send, mesh_broadcast, mesh_status, mesh_events ─

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_cannot_send_as_other_agent() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_send(
        State(state),
        Extension(TestDaemon::mcp_auth("agent-self")),
        Json(MeshSendReq {
            from: "agent-other".into(),
            to: "agent-target".into(),
            body: serde_json::json!({"text": "spoofed"}),
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_can_send_as_self() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_send(
        State(state),
        Extension(TestDaemon::mcp_auth("agent-self")),
        Json(MeshSendReq {
            from: "agent-self".into(),
            to: "agent-target".into(),
            body: serde_json::json!({"text": "ok"}),
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_cannot_broadcast_as_other_agent() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_broadcast(
        State(state),
        Extension(TestDaemon::mcp_auth("agent-self")),
        Json(MeshBroadcastReq {
            from: "agent-other".into(),
            body: serde_json::json!({}),
            capability_filter: None,
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_cannot_update_status_for_other_agent() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_status_update(
        State(state),
        Extension(TestDaemon::mcp_auth("agent-self")),
        Json(MeshStatusReq {
            agent_id: "agent-other".into(),
            status: "busy".into(),
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_cannot_drain_other_agents_mailbox() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_events(
        State(state),
        Extension(TestDaemon::mcp_auth("agent-self")),
        axum::extract::Path("agent-other".to_string()),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn cli_caller_can_drain_any_mailbox() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    // No mailbox registered → 404, not 403 — confirms identity check
    // passed for CLI caller.
    let resp = mesh_events(
        State(state),
        Extension(TestDaemon::cli_auth()),
        axum::extract::Path("agent-anything".to_string()),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// `mcp_agent_without_agent_id_can_register` removed with the `mesh_register`
// shim — actor/spawn does not gate on a bound MCP agent_id the same way.
