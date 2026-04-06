//! Tests for the REST API mesh endpoints — register, send, broadcast,
//! discover, status, checkout hook, SSE events, and identity binding.

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use tempfile::TempDir;

use crate::auth::AuthContext;
use crate::rest_api::*;
use crate::test_harness::TestDaemon;

fn make_api_state(dir: &TempDir) -> ApiState {
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = runtime::events::init_kernel_router(ship_dir.clone()).unwrap();
    let mesh_registry = std::sync::Arc::new(tokio::sync::RwLock::new(
        std::collections::HashMap::new(),
    ));
    ApiState {
        kernel,
        mesh_registry,
        agent_mailboxes: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
        pty_connections: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
        ship_dir,
    }
}

// ── mesh_register ───────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn register_returns_ok() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_register(
        State(state),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "agent.test".into(),
            capabilities: vec!["rust".into()],
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn register_spawns_actor_in_kernel() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "agent.spawned".into(),
            capabilities: vec![],
        }),
    )
    .await;

    let kernel = state.kernel.lock().await;
    assert!(kernel.has_actor("agent.spawned"));
}

#[tokio::test(flavor = "multi_thread")]
async fn register_stashes_mailbox() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "agent.mailbox".into(),
            capabilities: vec![],
        }),
    )
    .await;

    let mailboxes = state.agent_mailboxes.lock().await;
    assert!(mailboxes.contains_key("agent.mailbox"));
}

#[tokio::test(flavor = "multi_thread")]
async fn register_twice_replaces_actor() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "agent.twice".into(),
            capabilities: vec!["v1".into()],
        }),
    )
    .await;

    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "agent.twice".into(),
            capabilities: vec!["v2".into()],
        }),
    )
    .await;

    let kernel = state.kernel.lock().await;
    assert!(kernel.has_actor("agent.twice"));
}

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
        let mut registry = state.mesh_registry.write().await;
        registry.insert(
            "agent.a".into(),
            runtime::services::mesh::MeshEntry {
                agent_id: "agent.a".into(),
                label: "Agent A".into(),
                capabilities: vec!["rust".into()],
                registered_at: chrono::Utc::now(),
                status: runtime::services::mesh::AgentStatus::Active,
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
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();
    // init_project creates the markers that get_project_dir looks for.
    let _ = runtime::project::init_project(dir.path().to_path_buf());

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

    let has = state.agent_mailboxes.lock().await.contains_key("agent.unregistered");
    assert!(!has, "unregistered agent should have no mailbox");
}

#[tokio::test(flavor = "multi_thread")]
async fn mesh_events_registered_agent_has_mailbox() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "agent.sse-test".into(),
            capabilities: vec![],
        }),
    )
    .await;

    let has = state.agent_mailboxes.lock().await.contains_key("agent.sse-test");
    assert!(has, "registered agent must have stashed mailbox");
}

#[tokio::test(flavor = "multi_thread")]
async fn mesh_events_takes_mailbox_from_stash() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    mesh_register(
        State(state.clone()),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "agent.take".into(),
            capabilities: vec![],
        }),
    )
    .await;

    // Remove mailbox as the SSE endpoint would.
    let removed = state.agent_mailboxes.lock().await.remove("agent.take");
    assert!(removed.is_some(), "mailbox should be removable once");

    // Second removal finds nothing.
    let gone = state.agent_mailboxes.lock().await.remove("agent.take");
    assert!(gone.is_none(), "mailbox should only be takeable once");
}

// ── identity binding (S4: agent cannot impersonate) ────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_cannot_register_as_different_id() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    // MCP caller authenticated as "claude-v020" tries to register as "codex-main".
    let auth = TestDaemon::mcp_auth("claude-v020");
    let resp = mesh_register(
        State(state),
        Extension(auth),
        Json(MeshRegisterReq {
            agent_id: "codex-main".into(),
            capabilities: vec![],
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_can_register_as_own_id() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let auth = TestDaemon::mcp_auth("claude-v020");
    let resp = mesh_register(
        State(state),
        Extension(auth),
        Json(MeshRegisterReq {
            agent_id: "claude-v020".into(),
            capabilities: vec!["rust".into()],
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn cli_caller_can_register_any_agent() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    // CLI callers are trusted — no identity binding restriction.
    let resp = mesh_register(
        State(state),
        Extension(TestDaemon::cli_auth()),
        Json(MeshRegisterReq {
            agent_id: "any-agent-id".into(),
            capabilities: vec![],
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_agent_without_agent_id_can_register() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    // MCP caller with no agent_id in auth (e.g. Studio) — allowed.
    let auth = AuthContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        agent_id: None,
        workspace_id: None,
        session_id: None,
    };
    let resp = mesh_register(
        State(state),
        Extension(auth),
        Json(MeshRegisterReq {
            agent_id: "studio-agent".into(),
            capabilities: vec![],
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}
