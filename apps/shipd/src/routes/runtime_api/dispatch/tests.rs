//! Tests for dispatch handlers, request validation, and resume resolution.

use super::*;
use super::resume::{ResumeResolveError, load_and_validate_session_file};
use crate::auth::Scope;
use std::sync::Arc;
use tempfile::TempDir;
use ship_substrate::events::agent_session::InvocationMode;

fn make_state(dir: &TempDir) -> ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = ship_substrate::events::new_isolated_for_test(ship_dir.clone()).unwrap();
    Arc::new(crate::ship_runtime::ShipRuntime::new_test(ship_dir, kernel))
}

fn full_auth() -> Extension<AuthContext> {
    Extension(AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
        grants: vec![],
        // The dispatch REST handler gates on cross-project (mcp_workspace_violation),
        // not the operator path — no operator cred needed.
        operator_cred: false,
    })
}

fn mcp_auth(workspace: &str) -> Extension<AuthContext> {
    Extension(AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Mcp,
        agent_id: Some("rust-lane".into()),
        workspace_id: Some(workspace.into()),
        space_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
        grants: vec![],
        operator_cred: false,
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_cross_project_dispatch_rejected() {
    use axum::response::IntoResponse;
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    // Session bound to proj-a; request dispatches into proj-b.
    let req = bare_req("rust-lane", "proj-b");
    let resp = dispatch_agent_session(State(state), mcp_auth("proj-a"), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    use http_body_util::BodyExt;
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn bare_req(agent: &str, workspace: &str) -> DispatchRequest {
    DispatchRequest {
        agent: agent.into(),
        spec: None,
        prompt: None,
        resume: false,
        harness: None,
        model: None,
        open_tab: true,
        invocation: InvocationMode::Pty,
        keepalive: true,
        workspace: Some(workspace.into()),
        slug: None,
        branch: None,
        source_spec_path: None,
        depends_on: None,
        vars: None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn single_agent_dispatch_returns_session_id_slug_studio_url() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let req = bare_req("rust-lane", "solo");
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    let val = body_json(resp).await;
    assert_eq!(val["ok"], serde_json::json!(true));
    let data = &val["data"];
    assert!(data["slug"].as_str().unwrap().starts_with("solo-"));
    assert!(data["session_id"].as_str().is_some(), "session_id must be present");
    assert!(
        data["studio_url"]
            .as_str()
            .unwrap_or("")
            .contains("/sessions/"),
        "studio_url must point at the session route"
    );
    assert!(
        data["billing_warning"].is_null(),
        "PTY dispatch must not return a billing_warning"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_with_invocation_print_returns_billing_warning_for_claude() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("rust-lane", "billing-test");
    req.invocation = InvocationMode::Print;
    req.open_tab = false;
    req.keepalive = false;
    req.harness = Some("claude".to_string());
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    let val = body_json(resp).await;
    let warning = val["data"]["billing_warning"].as_str().unwrap_or("");
    assert!(
        warning.contains("API-billed"),
        "billing_warning must surface API-billed for claude -p, got: {warning}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_with_invocation_print_no_warning_for_non_claude() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("rust-lane", "billing-test");
    req.invocation = InvocationMode::Print;
    req.open_tab = false;
    req.keepalive = false;
    req.harness = Some("codex".to_string());
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    let val = body_json(resp).await;
    assert!(
        val["data"]["billing_warning"].is_null(),
        "non-claude print mode must not surface a billing warning"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_rejects_print_with_open_tab() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("rust-lane", "v");
    req.invocation = InvocationMode::Print;
    req.open_tab = true;
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let val = body_json(resp).await;
    let msg = val["data"].as_str().unwrap_or("");
    assert!(
        msg.contains("print") && msg.contains("attachable PTY"),
        "error must explain the conflict, got: {msg}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_rejects_print_with_keepalive() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("rust-lane", "v");
    req.invocation = InvocationMode::Print;
    req.open_tab = false;
    req.keepalive = true;
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let val = body_json(resp).await;
    let msg = val["data"].as_str().unwrap_or("");
    assert!(
        msg.contains("print") && msg.contains("keepalive"),
        "error must explain print exits on completion, got: {msg}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_rejects_stream_with_open_tab() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("codex", "v");
    req.invocation = InvocationMode::Stream;
    req.open_tab = true;
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let val = body_json(resp).await;
    let msg = val["data"].as_str().unwrap_or("");
    assert!(
        msg.contains("stream") && msg.contains("attach"),
        "error must explain server-mode attach unsupported, got: {msg}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_background_pty_keepalive_combination_valid() {
    // open_tab=false + invocation=pty + keepalive=true is the
    // canonical background subscription worker case. The old
    // `auto && keepalive → 400` mutex used to reject this; the
    // new model accepts it as a first-class combination.
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("rust-lane", "bg");
    req.open_tab = false;
    req.invocation = InvocationMode::Pty;
    req.keepalive = true;
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "background PTY keepalive worker must dispatch"
    );
}

#[test]
fn dispatch_request_rejects_pipeline_field() {
    // `deny_unknown_fields` makes the legacy `pipeline:` key a 400.
    let body = serde_json::json!({
        "agent": "rust-lane",
        "pipeline": "tdd",
    });
    let err = serde_json::from_value::<DispatchRequest>(body)
        .err()
        .expect("must reject pipeline field");
    let msg = err.to_string();
    assert!(
        msg.contains("pipeline") || msg.contains("unknown field"),
        "expected pipeline-field rejection, got: {msg}"
    );
}

use ship_harnesses::harnesses::session_file::write_session_file;

#[test]
fn load_session_file_missing_returns_400() {
    let tmp = TempDir::new().unwrap();
    let err = load_and_validate_session_file(tmp.path(), "some-slug", None)
        .expect_err("absent session.json must 400");
    match err {
        ResumeResolveError::Status { status, message } => {
            assert_eq!(status, 400u16);
            assert!(
                message.contains("no .ship/session.json"),
                "message was: {message}"
            );
        }
    }
}

#[test]
fn load_session_file_harness_mismatch_returns_400() {
    let tmp = TempDir::new().unwrap();
    write_session_file(tmp.path(), "claude", "sess-1", "rust-lane").unwrap();
    let err = load_and_validate_session_file(tmp.path(), "slug-x", Some("codex"))
        .expect_err("harness mismatch must 400");
    match err {
        ResumeResolveError::Status { status, message } => {
            assert_eq!(status, 400u16);
            assert!(
                message.contains("harness mismatch"),
                "message was: {message}"
            );
        }
    }
}

#[test]
fn load_session_file_unsupported_harness_returns_400() {
    // Cursor: session.json on disk but adapter refuses resume.
    // Dispatch must surface a 400 — never silently fresh-launch.
    let tmp = TempDir::new().unwrap();
    write_session_file(tmp.path(), "cursor", "anything", "rust-lane").unwrap();
    let err = load_and_validate_session_file(tmp.path(), "slug-x", None)
        .expect_err("cursor resume must 400");
    match err {
        ResumeResolveError::Status { status, message } => {
            assert_eq!(status, 400u16);
            assert!(
                message.contains("does not support session resume"),
                "message was: {message}"
            );
        }
    }
}

#[test]
fn load_session_file_happy_path_returns_record() {
    let tmp = TempDir::new().unwrap();
    write_session_file(tmp.path(), "claude", "sess-9", "rust-lane").unwrap();
    let s = load_and_validate_session_file(tmp.path(), "slug-x", Some("claude"))
        .expect("claude resume must validate");
    assert_eq!(s.harness, "claude");
    assert_eq!(s.session_id, "sess-9");
    assert_eq!(s.agent, "rust-lane");
}

#[tokio::test(flavor = "multi_thread")]
async fn resume_without_existing_session_returns_400() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("rust-lane", "empty-ws");
    req.resume = true;
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let val = body_json(resp).await;
    assert_eq!(val["ok"], serde_json::json!(false));
    let data = val["data"].as_str().unwrap_or("");
    assert!(
        data.contains("resume") && data.contains("no existing session"),
        "expected clear resume error, got: {data}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dispatch_accepts_prompt_and_returns_session() {
    let dir = TempDir::new().unwrap();
    let state = make_state(&dir);
    let mut req = bare_req("rust-lane", "p");
    req.prompt = Some("hello".into());
    let resp = dispatch_agent_session(State(state), full_auth(), Json(req))
        .await
        .into_response();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── DispatchRequest deserialization defaults ──────────────────

#[test]
fn minimal_request_deserializes_with_defaults() {
    // POST with just {"agent": "rust-lane"} must work — defaults
    // cover the three orthogonal axes. This is the shape every
    // shell, CLI, and MCP call relies on for the common case.
    let body = serde_json::json!({ "agent": "rust-lane" });
    let req: DispatchRequest = serde_json::from_value(body).expect("must accept minimal body");
    assert!(req.open_tab, "default open_tab=true (interactive)");
    assert!(req.keepalive, "default keepalive=true (chat shape)");
    assert_eq!(req.invocation, InvocationMode::Pty);
}

#[test]
fn invocation_field_parses_print_and_stream() {
    for (raw, expected) in [
        ("\"pty\"", InvocationMode::Pty),
        ("\"print\"", InvocationMode::Print),
        ("\"stream\"", InvocationMode::Stream),
    ] {
        let body = format!(
            "{{\"agent\":\"rust-lane\",\"invocation\":{raw},\"open_tab\":false,\"keepalive\":false}}"
        );
        let req: DispatchRequest = serde_json::from_str(&body).expect("must parse");
        assert_eq!(req.invocation, expected);
    }
}

#[test]
fn rejects_old_auto_field() {
    // `auto` is gone. A request that includes it should fail
    // deserialization (deny_unknown_fields), surfacing the rename
    // to operators rather than silently dropping the field.
    let body = serde_json::json!({
        "agent": "rust-lane",
        "auto": true,
    });
    let err = serde_json::from_value::<DispatchRequest>(body)
        .err()
        .expect("must reject the removed auto field");
    let msg = err.to_string();
    assert!(
        msg.contains("auto") || msg.contains("unknown field"),
        "expected auto-field rejection (the new model uses open_tab + invocation + keepalive), got: {msg}"
    );
}
