//! Integration tests for the agent-facing job/status and step-done daemon
//! endpoints used by the MCP `update_job` and `step_done` tools.
//!
//! These exercise the projection-update path that the previous direct
//! `SqliteEventStore` write was bypassing — emitting via the endpoint must
//! both persist the event AND advance the jobs projection.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use tempfile::TempDir;

use super::{JobStatusReq, StepDoneReq, job_step_done, update_job_status};
use crate::project_api::EmitEventReq;

fn make_api_state(dir: &TempDir) -> crate::rest_api::ApiState {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    let kernel = ship_substrate::events::new_isolated_for_test(ship_dir.clone()).unwrap();
    std::sync::Arc::new(crate::ship_runtime::ShipRuntime::new_test(
        ship_dir,
        kernel,
    ))
}

async fn seed_job(state: &crate::rest_api::ApiState, slug: &str) -> String {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::job::{JobCreatedPayload, event_types};

    let job_id = ship_substrate::gen_ulid();
    let payload = JobCreatedPayload {
        job_id: job_id.clone(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("test/{slug}"),
        spec_path: String::new(),
        source_spec_path: None,
        plan_id: None,
        model: None,
        provider: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
        baseline_sha: None,
    };
    let env = EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
        .unwrap()
        .with_actor_id("test");
    state.persist_and_route(&env).await.unwrap();
    job_id
}

#[tokio::test(flavor = "multi_thread")]
async fn update_job_status_dispatched_advances_projection() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let job_id = seed_job(&state, "demo-dispatch").await;

    let (status, body) = update_job_status(
        State(state.clone()),
        None,
        Path(job_id.clone()),
        axum::Json(JobStatusReq {
            status: "dispatched".to_string(),
            worktree: Some("/tmp/ws-demo".to_string()),
            pid: Some(4321),
            gate_agent: None,
            blocker: None,
            error: None,
        }),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);
    assert!(body.0.ok);

    // Projection must reflect the dispatch — this is the assertion that
    // proves the daemon's kernel emit drove the projection update, which
    // is precisely what the old direct-write path skipped.
    let jobs = ship_platform::projections::job::load_jobs_at(state.ship_dir()).unwrap();
    let rec = jobs.get(&job_id).expect("job projection row missing");
    use ship_platform::projections::job::JobStatus;
    assert!(
        matches!(rec.status, JobStatus::Dispatched),
        "expected Dispatched, got {:?}",
        rec.status
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn update_job_status_rejects_unknown_value() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let job_id = seed_job(&state, "demo-bad").await;

    let err = update_job_status(
        State(state),
        None,
        Path(job_id),
        axum::Json(JobStatusReq {
            status: "invented".to_string(),
            worktree: None,
            pid: None,
            gate_agent: None,
            blocker: None,
            error: None,
        }),
    )
    .await
    .unwrap_err();
    assert_eq!(err.0, StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_job_status_requires_worktree_when_dispatched() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let job_id = seed_job(&state, "demo-noworktree").await;

    let err = update_job_status(
        State(state),
        None,
        Path(job_id),
        axum::Json(JobStatusReq {
            status: "dispatched".to_string(),
            worktree: None,
            pid: None,
            gate_agent: None,
            blocker: None,
            error: None,
        }),
    )
    .await
    .unwrap_err();
    assert_eq!(err.0, StatusCode::BAD_REQUEST);
}

#[tokio::test(flavor = "multi_thread")]
async fn step_done_404_when_job_missing() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let err = job_step_done(
        State(state),
        None,
        Path("ghost-slug".to_string()),
        axum::Json(StepDoneReq { summary: None, outputs: Default::default() }),
    )
    .await
    .unwrap_err();
    assert_eq!(err.0, StatusCode::NOT_FOUND);
}

// ---- Domain validation on /api/v1/events/emit ----

#[tokio::test(flavor = "multi_thread")]
async fn emit_event_validate_domain_rejects_reserved_namespace() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);

    let err = super::super::emit_event(
        State(state),
        None,
        axum::Json(EmitEventReq {
            event_type: "job.created".to_string(),
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

    let err = super::super::emit_event(
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

    let (status, body) = super::super::emit_event(
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
