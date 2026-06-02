//! Integration tests for pipeline step I/O.
//!
//! Exercises the real flow: store artifacts through `ArtifactService`,
//! emit `job.created` + `step.done_declared` events through the real
//! event store, then call the orchestrator-side step I/O helpers to
//! verify end-to-end behavior.
//!
//! The runtime's test binary gives us a per-thread temp `SHIP_HOME`
//! automatically (`project::global::get_global_dir`'s test-isolation
//! path), so every helper that calls `get_global_dir()` lands in the
//! same directory without us having to touch env vars.
//!
//! Each test clears that directory's `platform.db` + `data.db` on entry
//! so prior tests sharing the same worker thread start from a clean state.

use std::sync::Arc;

use ship_artifacts::service::ArtifactService;
use ship_artifacts::types::ArtifactScope;
use ship_substrate::events::pipeline::{
    OutputScope, Pipeline, PipelineStep, PipelineStepDoneDeclaredPayload, StepOutput, StepPromote, StepType,
};
use ship_substrate::events::{EventEnvelope, EventSink};
use ship_substrate::persistence::event_store::SqliteEventStore;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};
use crate::services::orchestrator::step_io::{
    check_required_outputs, find_artifact_id_in_events, run_step_promotes,
};

/// Build an `ArtifactService` over a `SpaceContext` for the test's space.
fn art_service(
    kernel: &Arc<tokio::sync::Mutex<ship_substrate::events::KernelRouter>>,
    ship_dir: &std::path::Path,
    space_id: &str,
) -> ArtifactService {
    let data_svc = ship_substrate::data::DataService::open(ship_dir).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(data_svc, space_id));
    let kernel_log = Arc::new(KernelLog::new(kernel.clone()));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(space_id), store, kernel_log));
    ArtifactService::new(ctx, ship_dir.to_path_buf())
}

fn step(id: &str, agent: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: StepType::Agent,
        agent: Some(agent.into()),
        goal: None,
        after: vec![],
        model: None,
        harness: None,
        command: None,
        max_retries: None,
        gate: None,
        produces: vec![],
        consumes: vec![],
        promotes: vec![],
        fanout: None,
            invocation: None,
    }
}

/// Reset the test-isolated dir so tests sharing a worker thread don't
/// see stale events from a prior run.
fn fresh_ship_dir() -> std::path::PathBuf {
    let dir = ship_substrate::project::get_global_dir().expect("test ship dir");
    for name in ["platform.db", "data.db"] {
        let p = dir.join(name);
        if p.exists() { let _ = std::fs::remove_file(&p); }
    }
    dir
}

fn emit_event(ship_dir: &std::path::Path, event_type: &str, entity: &str, payload: serde_json::Value) {
    // pipeline.*/step.* are Runtime-scoped → seed runtime.db (the orchestrator
    // projection loaders read it).
    let store = SqliteEventStore::new_at_db_path(
        &ship_substrate::db::runtime_db_path(ship_dir),
    )
    .expect("event store");
    let env = EventEnvelope::new(event_type, entity, &payload).unwrap();
    store.persist(&env).expect("persist");
}

fn emit_pipeline_run_started(ship_dir: &std::path::Path, pipeline_run_id: &str, pipeline: &Pipeline) {
    emit_event(ship_dir, "pipeline.run.started", pipeline_run_id, serde_json::json!({
        "pipeline_run_id": pipeline_run_id,
        "slug": "test-slug",
        "agent": "orchestrator",
        "branch": "test",
        "spec_path": ".artifacts/job-spec.md",
        "steps": pipeline,
    }));
}

fn emit_step_done(
    ship_dir: &std::path::Path,
    pipeline_run_id: &str,
    step_id: &str,
    outputs: std::collections::BTreeMap<String, String>,
) {
    let payload = PipelineStepDoneDeclaredPayload {
        pipeline_run_id: pipeline_run_id.into(),
        slug: "test-slug".into(),
        step_id: step_id.into(),
        summary: Some("ok".into()),
        outputs,
    };
    emit_event(
        ship_dir,
        "pipeline.step.done_declared",
        pipeline_run_id,
        serde_json::to_value(&payload).unwrap(),
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn three_step_flow_resolves_consumes_and_promotes_patch() {
    let ship_dir = fresh_ship_dir();

    // The orchestrator's read-side `artifact_service` binds its `SpaceContext`
    // to the run's space, which it derives as `project_slug_from_ship_dir`.
    // The `SingleBackendDataStore` partitions rows by that space, so the store
    // side must use the SAME space for the row to be readable by id.
    let space_id = ship_substrate::project::project_slug_from_ship_dir(&ship_dir);
    let space_id = space_id.as_str();
    let pipeline_run_id = "job-three-step";
    let run_id = "run-001";
    let run_scope = ArtifactScope::Run {
        space_id: space_id.into(),
        run_id: run_id.into(),
    };

    let kernel = std::sync::Arc::new(tokio::sync::Mutex::new(
        ship_substrate::events::kernel_router::KernelRouter::new(ship_dir.clone()).expect("kernel"),
    ));
    let art_svc = art_service(&kernel, &ship_dir, space_id);

    // Step 1 — write-tests produces `failing_tests`.
    let tests_art = art_svc
        .store("failing_tests.js", "text/javascript",
               b"describe(); it(); expect();", &run_scope,
               None, None, Some("test-writer"), None)
        .await
        .unwrap();
    emit_step_done(&ship_dir, pipeline_run_id, "write-tests",
        [("failing_tests".to_string(), tests_art.id.clone())].into());

    // Resolver finds the upstream artifact by (pipeline_run_id, step_id, name).
    let resolved = find_artifact_id_in_events(pipeline_run_id, "write-tests", "failing_tests")
        .expect("resolver must find the tests artifact emitted by write-tests");
    assert_eq!(resolved, tests_art.id);

    // Step 2 — implement produces `patch`.
    let patch_art = art_svc
        .store("patch.diff", "text/x-diff",
               b"diff --git a/f b/f\n@@ -1 +1,2 @@\n hi\n+hello\n",
               &run_scope, None, None, Some("implementer"), None)
        .await
        .unwrap();
    emit_step_done(&ship_dir, pipeline_run_id, "implement",
        [("patch".to_string(), patch_art.id.clone())].into());

    // Step 3 — gate consumes patch, promotes it to project scope.
    let mut gate_step = step("gate", "reviewer");
    gate_step.after = vec!["implement".into()];
    gate_step.consumes = vec!["patch".into()];
    gate_step.promotes = vec![StepPromote {
        name: "patch".into(),
        target_scope: OutputScope::Project,
    }];

    let pipeline = Pipeline {
        steps: vec![
            {
                let mut s = step("write-tests", "test-writer");
                s.produces = vec![StepOutput {
                    name: "failing_tests".into(),
                    scope: OutputScope::Run,
                    required: true,
                }];
                s
            },
            {
                let mut s = step("implement", "implementer");
                s.after = vec!["write-tests".into()];
                s.consumes = vec!["failing_tests".into()];
                s.produces = vec![StepOutput {
                    name: "patch".into(),
                    scope: OutputScope::Run,
                    required: true,
                }];
                s
            },
            gate_step.clone(),
        ],
        auto_merge: false,
    };
    emit_pipeline_run_started(&ship_dir, pipeline_run_id, &pipeline);

    run_step_promotes(&kernel, pipeline_run_id, &gate_step).await;

    // Verify the patch now lives in project scope with the same bytes.
    let filter: std::collections::BTreeMap<String, serde_json::Value> = [
        ("scope".to_string(), serde_json::Value::String("project".into())),
        ("content_hash".to_string(), serde_json::Value::String(
            patch_art.content_hash.clone().unwrap_or_default())),
    ].into();
    let promoted = art_svc.list(filter, 10).await.unwrap();
    assert_eq!(promoted.len(), 1, "patch must have been promoted to project scope");
    assert_eq!(promoted[0].scope, "project");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_required_output_blocks_downstream_resolution() {
    let _ship_dir = fresh_ship_dir();

    // The producer step declared a required output but reported no outputs.
    let produces = vec![StepOutput {
        name: "patch".into(),
        scope: OutputScope::Run,
        required: true,
    }];
    let empty = std::collections::BTreeMap::new();
    let reason = check_required_outputs(&produces, &empty)
        .expect("required output missing must be flagged");
    assert!(reason.contains("patch"), "message must name the missing output: {reason}");

    // Downstream resolution returns None — the consumer step would fail
    // rather than silently dispatching without its input.
    let missing = find_artifact_id_in_events("job-missing", "step-that-never-ran", "patch");
    assert!(missing.is_none(),
        "resolver must return None when the producer never emitted outputs");
}
