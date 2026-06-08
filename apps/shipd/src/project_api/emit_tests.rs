//! Domain-validation tests for `POST /api/v1/events/emit`.
//!
//! Relocated from the deleted `project_api/jobs_status_tests.rs` — these are
//! unrelated to jobs-status. `emit_event` + `EmitEventReq` still live in
//! `project_api/mod.rs` until the `EventsService.Emit` cutover, so the tests
//! stay attached here.

use axum::extract::State;
use axum::http::StatusCode;
use tempfile::TempDir;

use super::{EmitEventReq, emit_event};

fn make_api_state(dir: &TempDir) -> crate::routes::rest_api::ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = ship_substrate::events::new_isolated_for_test(ship_dir.clone()).unwrap();
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(
        ship_dir,
        kernel,
    ))
}

#[tokio::test(flavor = "multi_thread")]
async fn emit_event_validate_domain_rejects_reserved_namespace() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let err = emit_event(
        State(state),
        None,
        axum::Json(EmitEventReq {
            event_type: "pipeline.run.started".to_string(),
            entity_id: "x".to_string(),
            workspace_id: None,
            payload: serde_json::json!({}),
            actor_id: Some("agent-a".to_string()),
            elevated: None,
            validate_domain: Some(true),
            actor: None,
        }),
    )
    .await
    .unwrap_err();
    assert_eq!(err.0, StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn emit_event_validate_domain_rejects_undotted() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let err = emit_event(
        State(state),
        None,
        axum::Json(EmitEventReq {
            event_type: "nodot".to_string(),
            entity_id: "x".to_string(),
            workspace_id: None,
            payload: serde_json::json!({}),
            actor_id: Some("agent-a".to_string()),
            elevated: None,
            validate_domain: Some(true),
            actor: None,
        }),
    )
    .await
    .unwrap_err();
    assert_eq!(err.0, StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn emit_event_accepts_valid_domain_event_with_actor_id() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let (status, body) = emit_event(
        State(state),
        None,
        axum::Json(EmitEventReq {
            event_type: "deployment.completed".to_string(),
            entity_id: "demo".to_string(),
            workspace_id: Some("ws-1".to_string()),
            payload: serde_json::json!({"env": "prod"}),
            actor_id: Some("agent-writer".to_string()),
            elevated: Some(false),
            validate_domain: Some(true),
            actor: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);
    assert!(body.0.ok);
}
