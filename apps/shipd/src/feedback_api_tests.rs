//! Integration tests for `POST /api/runtime/feedback`.

use axum::{Router, body::Body, http::Request, response::IntoResponse};
use http_body_util::BodyExt;
use std::sync::Arc;
use tempfile::TempDir;
use tower::ServiceExt;
use ulid::Ulid;

use super::FeedbackGuard;
use crate::auth::AuthContext;
use crate::rest_api::ApiState;
use crate::ship_runtime::ShipRuntime;

fn router_for(state: ApiState, auth: AuthContext) -> Router {
    Router::new()
        .route(
            "/api/runtime/feedback",
            axum::routing::post(super::post_feedback),
        )
        // A view-session route is included so tests can open a session to
        // exercise the view-scoped routing branch through the same harness.
        .route(
            "/api/runtime/view-sessions",
            axum::routing::post(crate::view_sessions_api::open_session),
        )
        .with_state(state)
        .layer(axum::Extension(auth))
        .layer(axum::Extension(FeedbackGuard::new()))
}

fn fresh_state() -> (ApiState, TempDir) {
    let dir = TempDir::new().unwrap();
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    runtime::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = Arc::new(tokio::sync::Mutex::new(
        runtime::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let state: ApiState = Arc::new(ShipRuntime::new_test(ship_dir, kernel));
    (state, dir)
}

fn mcp_auth(agent: &str) -> AuthContext {
    AuthContext {
        caller_kind: runtime::events::CallerKind::Mcp,
        agent_id: Some(agent.into()),
        workspace_id: None,
        session_id: None,
        scopes: vec![crate::auth::Scope::Full],
    }
}

async fn send_json(router: &Router, path: &str, body: serde_json::Value) -> (u16, serde_json::Value) {
    let req = Request::builder()
        .method("POST")
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

fn base_payload() -> serde_json::Value {
    serde_json::json!({
        "id": Ulid::new().to_string(),
        "kind": "point",
        "target": { "selector": "#root" },
        "created_at": "2026-04-13T00:00:00Z",
        "author_kind": "human",
    })
}

async fn open_view(router: &Router) -> String {
    let (s, body) = send_json(
        router,
        "/api/runtime/view-sessions",
        serde_json::json!({"template": "canvas/blank"}),
    )
    .await;
    assert_eq!(s, 200, "open: {body}");
    body["data"]["view_id"].as_str().unwrap().to_string()
}

// ─────────────────── Happy paths ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn view_scoped_feedback_fans_out_to_concurrent_sse_subscribers() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state.clone(), mcp_auth("agent.owner"));
    let view_id = open_view(&router).await;

    // Two concurrent subscribers on the view-session SSE stream.
    let sse_a = state.view_sessions().subscribe(&view_id).await.unwrap();
    let sse_b = state.view_sessions().subscribe(&view_id).await.unwrap();
    let ha = tokio::spawn(async move {
        let bytes = sse_a.into_response().into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    });
    let hb = tokio::spawn(async move {
        let bytes = sse_b.into_response().into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    });
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;

    let mut payload = base_payload();
    payload["target"]["view_id"] = serde_json::Value::String(view_id.clone());
    let id = payload["id"].as_str().unwrap().to_string();
    let (status, body) = send_json(&router, "/api/runtime/feedback", payload).await;
    assert_eq!(status, 201, "unexpected body: {body}");
    assert_eq!(body["id"], serde_json::Value::String(id.clone()));

    // Close the session — drops both producers so the two SSE bodies
    // terminate and we can collect them.
    state.view_sessions().close(&view_id, "agent.owner").await.unwrap();

    let body_a = tokio::time::timeout(std::time::Duration::from_secs(1), ha)
        .await
        .expect("sse A timed out")
        .unwrap();
    let body_b = tokio::time::timeout(std::time::Duration::from_secs(1), hb)
        .await
        .expect("sse B timed out")
        .unwrap();
    assert!(body_a.contains(&id), "sse A missing id {id}: {body_a}");
    assert!(body_b.contains(&id), "sse B missing id {id}: {body_b}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn view_scoped_feedback_reaches_owner_actor_mailbox() {
    let (state, _tmp) = fresh_state();

    // Stand up an actor subscribed to `studio.` — the owner's session would
    // do this; here we verify kernel delivery directly.
    let mut mailbox = spawn_studio_subscriber(&state, "agent.owner").await;
    let router = router_for(state.clone(), mcp_auth("agent.owner"));
    let view_id = open_view(&router).await;
    let mut payload = base_payload();
    payload["target"]["view_id"] = serde_json::Value::String(view_id.clone());
    let id = payload["id"].as_str().unwrap().to_string();
    let (status, _) = send_json(&router, "/api/runtime/feedback", payload).await;
    assert_eq!(status, 201);

    let ev = tokio::time::timeout(std::time::Duration::from_secs(1), mailbox.recv())
        .await
        .expect("kernel did not deliver within 1s")
        .expect("mailbox closed");
    assert_eq!(ev.event_type, "studio.feedback.created");
    assert!(ev.payload_json.contains(&id));
}

async fn spawn_studio_subscriber(
    state: &ApiState,
    agent: &str,
) -> runtime::events::Mailbox {
    let mut kernel = state.kernel().lock().await;
    let (_store, mailbox) = kernel
        .spawn_actor(
            agent,
            runtime::events::ActorConfig {
                namespace: agent.into(),
                write_namespaces: vec!["".into()],
                read_namespaces: vec!["agent.".into()],
                subscribe_namespaces: vec!["studio.".into()],
            },
        )
        .unwrap();
    mailbox
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn broadcast_feedback_without_view_id_reaches_subscribers() {
    let (state, _tmp) = fresh_state();
    let mut mailbox = spawn_studio_subscriber(&state, "agent.observer").await;
    let router = router_for(state, mcp_auth("agent.caller"));
    let payload = base_payload(); // no view_id
    let id = payload["id"].as_str().unwrap().to_string();
    let (status, _) = send_json(&router, "/api/runtime/feedback", payload).await;
    assert_eq!(status, 201);

    let ev = tokio::time::timeout(std::time::Duration::from_secs(1), mailbox.recv())
        .await
        .expect("workspace broadcast not delivered")
        .expect("mailbox closed");
    assert_eq!(ev.event_type, "studio.feedback.created");
    assert_eq!(ev.entity_id, id);
}

// ─────────────────── 404 / routing guards ───────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_view_id_returns_404_without_fallback_broadcast() {
    let (state, _tmp) = fresh_state();
    let mut mailbox = spawn_studio_subscriber(&state, "agent.obs2").await;
    let router = router_for(state, mcp_auth("agent.caller"));
    let mut payload = base_payload();
    payload["target"]["view_id"] = serde_json::Value::String("view-does-not-exist".into());
    let (status, body) = send_json(&router, "/api/runtime/feedback", payload).await;
    assert_eq!(status, 404);
    assert_eq!(body["error"]["code"], "view_not_found");
    // Give the kernel a chance to (wrongly) deliver; assert it did not.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        mailbox.try_recv().is_err(),
        "404 on unknown view_id must not fall back to broadcast"
    );
}

// ─────────────────── Validation (≥6 cases) ───────────────────

/// Table-driven validation: each row asserts that the given mutation on the
/// base payload yields a 422 with the named error code.
#[tokio::test(flavor = "multi_thread")]
async fn validation_rules_return_422() {
    type Mutate = fn(&mut serde_json::Value);
    let cases: &[(&str, Mutate)] = &[
        ("invalid_id", |p| p["id"] = serde_json::json!("not-a-ulid")),
        ("selector_required", |p| p["target"] = serde_json::json!({})),
        ("rect_forbidden", |p| p["target"]["rect"] = serde_json::json!([0.0, 0.0, 1.0, 1.0])),
        ("rect_required", |p| {
            p["kind"] = serde_json::json!("region");
            p["target"] = serde_json::json!({});
        }),
        ("rect_invalid", |p| {
            p["kind"] = serde_json::json!("region");
            p["target"] = serde_json::json!({ "rect": [-1.0, 0.0, 1.0, 1.0] });
        }),
        ("note_too_large", |p| p["note"] = serde_json::json!("x".repeat(10 * 1024 + 1))),
        ("selection_too_large", |p| {
            p["selection"] = serde_json::json!({ "kind": "text", "text": "x".repeat(20 * 1024 + 1) });
        }),
    ];
    for (expected_code, mutate) in cases {
        let (state, _tmp) = fresh_state();
        let router = router_for(state, mcp_auth("agent.caller"));
        let mut payload = base_payload();
        mutate(&mut payload);
        let (status, body) = send_json(&router, "/api/runtime/feedback", payload).await;
        assert_eq!(status, 422, "code `{expected_code}` expected 422: {body}");
        assert_eq!(body["error"]["code"], *expected_code, "case `{expected_code}`: {body}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn duplicate_id_within_window_returns_422() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, mcp_auth("agent.caller"));
    let payload = base_payload();
    let (status, _) = send_json(&router, "/api/runtime/feedback", payload.clone()).await;
    assert_eq!(status, 201);
    let (status, body) = send_json(&router, "/api/runtime/feedback", payload).await;
    assert_eq!(status, 422);
    assert_eq!(body["error"]["code"], "duplicate_id");
}

// ─────────────────── Rate-limit ───────────────────

#[tokio::test(flavor = "multi_thread")]
async fn agent_author_burst_triggers_429() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, mcp_auth("agent.burst"));
    let mut saw_429 = false;
    // Capacity is 10; 12 back-to-back calls must trip the limiter.
    for _ in 0..12 {
        let mut p = base_payload();
        p["author_kind"] = serde_json::Value::String("agent".into());
        let (s, body) = send_json(&router, "/api/runtime/feedback", p).await;
        if s == 429 {
            assert_eq!(body["error"]["code"], "rate_limited");
            saw_429 = true;
            break;
        } else {
            assert_eq!(s, 201);
        }
    }
    assert!(saw_429, "agent burst did not hit the rate limit");
}

#[tokio::test(flavor = "multi_thread")]
async fn human_author_is_not_rate_limited() {
    let (state, _tmp) = fresh_state();
    let router = router_for(state, mcp_auth("agent.host"));
    // Humans must be allowed to send well past the agent cap.
    for _ in 0..30 {
        let (s, body) = send_json(&router, "/api/runtime/feedback", base_payload()).await;
        assert_eq!(s, 201, "human call unexpectedly failed: {body}");
    }
}
