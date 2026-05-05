//! Scope enforcement tests — verify that handlers reject callers
//! who lack the required scope and accept those who have it.

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use tempfile::TempDir;

use crate::auth::{AuthContext, Scope};
use crate::rest_api::*;
use crate::test_harness::TestDaemon;

fn make_api_state(dir: &TempDir) -> ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
}

fn worker_auth() -> AuthContext {
    AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: Some("agent.worker".into()),
        workspace_id: None,
        session_id: None,
        scopes: crate::auth::scopes_for_role(Some("worker")),
    }
}

fn commander_auth() -> AuthContext {
    AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: Some("agent.commander".into()),
        workspace_id: None,
        session_id: None,
        scopes: crate::auth::scopes_for_role(Some("commander")),
    }
}

// ── Broadcast scope ───────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn worker_cannot_broadcast() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_broadcast(
        State(state),
        Extension(worker_auth()),
        Json(MeshBroadcastReq {
            from: "agent.worker".into(),
            body: serde_json::json!({}),
            capability_filter: None,
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread")]
async fn commander_can_broadcast() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_broadcast(
        State(state),
        Extension(commander_auth()),
        Json(MeshBroadcastReq {
            from: "agent.commander".into(),
            body: serde_json::json!({}),
            capability_filter: None,
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── MeshSend scope ────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn worker_can_send_mesh_message() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = mesh_send(
        State(state),
        Extension(worker_auth()),
        Json(MeshSendReq {
            from: "agent.worker".into(),
            to: "agent.lead".into(),
            body: serde_json::json!({"text": "done"}),
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── CreateJobs scope ──────────────────────────────────────────────────────
//
// Worker tokens must be rejected by the dispatch endpoint so sub-agents
// can't spawn top-level jobs on behalf of the user.

fn dispatch_req(pipeline: &str) -> crate::runtime_api::DispatchRequest {
    crate::runtime_api::DispatchRequest {
        pipeline: pipeline.into(),
        spec: None,
        provider: None,
        model: None,
        vars: None,
        headless: false,
        workspace: None,
        slug: None,
        branch: None,
        agent: None,
        source_spec_path: None,
        depends_on: None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn worker_cannot_create_jobs() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let resp = crate::runtime_api::dispatch_pipeline(
        axum::extract::State(state),
        Extension(worker_auth()),
        Json(dispatch_req("test-pipeline")),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── Full scope for daemon token and contributor ───────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn daemon_token_has_full_scope() {
    let auth = TestDaemon::cli_auth();
    assert!(auth.has_scope(Scope::Full));
    assert!(auth.has_scope(Scope::CreateJobs));
    assert!(auth.has_scope(Scope::AdminDaemon));
    assert!(auth.has_scope(Scope::MeshBroadcast));
}

#[tokio::test(flavor = "multi_thread")]
async fn contributor_has_full_scope() {
    let scopes = crate::auth::scopes_for_role(Some("contributor"));
    assert!(Scope::Full.is_granted_by(&scopes));
    assert!(Scope::CreateJobs.is_granted_by(&scopes));
    assert!(Scope::AdminDaemon.is_granted_by(&scopes));
}

// ── Scope-less caller gets no special access ──────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn unknown_role_gets_minimal_scopes() {
    let scopes = crate::auth::scopes_for_role(Some("unknown-role"));
    assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
    assert!(Scope::ReadJobs.is_granted_by(&scopes));
    assert!(Scope::LogProgress.is_granted_by(&scopes));
    assert!(!Scope::CreateJobs.is_granted_by(&scopes));
    assert!(!Scope::MeshSend.is_granted_by(&scopes));
    assert!(!Scope::WriteWorkspace.is_granted_by(&scopes));
}

// ── Event route requires EmitEvents ───────────────────────────────────────

#[tokio::test(flavor = "multi_thread")]
async fn minimal_scope_cannot_route_events() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let auth = AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: None,
        workspace_id: None,
        session_id: None,
        scopes: vec![Scope::ReadWorkspace],
    };

    let envelope = ship_substrate::events::EventEnvelope::new(
        "test.event",
        "entity-1",
        &serde_json::json!({}),
    )
    .unwrap();

    let resp = crate::actor_api::event_route(
        axum::extract::State(state),
        Extension(auth),
        Json(crate::actor_api::EventRouteReq {
            envelope,
            workspace_id: None,
            session_id: None,
        }),
    )
    .await
    .into_response();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
