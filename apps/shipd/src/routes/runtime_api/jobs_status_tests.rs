//! Integration tests for the agent-facing job/status and step-done daemon
//! handlers used by the MCP `update_job` and `step_done` tools.
//!
//! Relocated from `project_api/jobs_status_tests.rs` when the cores moved to
//! `ship-pipeline-runtime`. They need a real ApiState + kernel (the projection
//! rebuild path the previous direct `SqliteEventStore` write was bypassing),
//! so they stay in shipd, attached to the relocated handlers.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use tempfile::TempDir;

use ship_pipeline_runtime::status_core::{RunStatusReq, StepDoneReq};

use super::{step_done, update_job_status};

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

async fn seed_job(state: &crate::routes::rest_api::ApiState, slug: &str) -> String {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::pipeline::{PipelineRunStartedPayload, event_types};

    let pipeline_run_id = ship_substrate::gen_ulid();
    let payload = PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.clone(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("test/{slug}"),
        spec_path: String::new(),
        source_spec_path: None,
        plan_id: None,
        model: None,
        harness: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: None,
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, &pipeline_run_id, &payload)
        .unwrap()
        .with_actor_id("test");
    state.persist_and_route(&env).await.unwrap();
    pipeline_run_id
}

#[tokio::test(flavor = "multi_thread")]
async fn update_job_status_dispatched_advances_projection() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let pipeline_run_id = seed_job(&state, "demo-dispatch").await;

    let (status, body) = update_job_status(
        State(state.clone()),
        None,
        Path(pipeline_run_id.clone()),
        axum::Json(RunStatusReq {
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
    let jobs = ship_pipeline_runtime::projections::pipeline_run::load_pipeline_runs_at(state.ship_dir()).unwrap();
    let rec = jobs.get(&pipeline_run_id).expect("job projection row missing");
    use ship_pipeline_runtime::projections::pipeline_run::PipelineRunStatus;
    assert!(
        matches!(rec.status, PipelineRunStatus::Dispatched),
        "expected Dispatched, got {:?}",
        rec.status
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn update_job_status_rejects_unknown_value() {
    let dir = TempDir::new().unwrap();
    let state = make_api_state(&dir);
    let pipeline_run_id = seed_job(&state, "demo-bad").await;

    let err = update_job_status(
        State(state),
        None,
        Path(pipeline_run_id),
        axum::Json(RunStatusReq {
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
    let pipeline_run_id = seed_job(&state, "demo-noworktree").await;

    let err = update_job_status(
        State(state),
        None,
        Path(pipeline_run_id),
        axum::Json(RunStatusReq {
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

    let err = step_done(
        State(state),
        None,
        Path("ghost-slug".to_string()),
        axum::Json(StepDoneReq { summary: None, outputs: Default::default() }),
    )
    .await
    .unwrap_err();
    assert_eq!(err.0, StatusCode::NOT_FOUND);
}
