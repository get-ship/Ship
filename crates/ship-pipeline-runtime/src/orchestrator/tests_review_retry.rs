//! Tests for `review_retry` — `try_review_retry` and
//! `emit_pipeline_run_failed_for_review`.
//!
//! `try_review_retry` reads from the pipeline_run projection so every test
//! seeds a `pipeline.run.started` event first (which lands the job in the
//! projection). The kernel actor grants `pipeline.*` write access so the
//! emitters can persist events.

use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_substrate::events::pipeline::{
    event_types, Pipeline, PipelineRunFailedPayload, PipelineRunStartedPayload,
    PipelineStep, PipelineStepObservedPayload, StepTrigger, StepType, StepVerdict,
};
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};

use crate::orchestrator::review_retry::{
    emit_pipeline_run_failed_for_review, try_review_retry,
};
use crate::orchestrator::tests_fanout_harness::make_octx;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fresh_ship_dir() -> std::path::PathBuf {
    let dir = ship_substrate::project::get_global_dir().expect("test ship dir");
    for name in ["platform.db", "data.db"] {
        let p = dir.join(name);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }
    dir
}

fn actor_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.orchestrator".to_string(),
        write_namespaces: vec!["pipeline.".to_string(), "job.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec![],
        see_directed_events: false,
    }
}

fn make_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router.spawn_actor("service.orchestrator", actor_config()).unwrap();
    Arc::new(Mutex::new(router))
}

fn agent_step(id: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: StepType::Agent,
        agent: Some("implementer".into()),
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

fn review_step(id: &str, after_step: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(),
        step_type: StepType::Review,
        agent: Some("reviewer".into()),
        goal: None,
        after: vec![after_step.into()],
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

async fn seed_job(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    pipeline: Pipeline,
) {
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    let payload = PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.into(),
        slug: slug.into(),
        agent: "implementer".into(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".into(),
        source_spec_path: None,
        plan_id: None,
        model: None,
        harness: None,
        depends_on: None,
        pipeline: None,
        steps: Some(pipeline),
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
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, pipeline_run_id, &payload)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

fn observed_for(
    pipeline_run_id: &str,
    slug: &str,
    step_name: &str,
    reason: Option<&str>,
) -> PipelineStepObservedPayload {
    PipelineStepObservedPayload {
        pipeline_run_id: pipeline_run_id.into(),
        slug: slug.into(),
        step_index: 0,
        step_name: step_name.into(),
        trigger: StepTrigger::DoneDeclared,
        done_declared: true,
        done_result: Some(false),
        exit_code: None,
        commits_since_start: 0,
        uncommitted_files: 0,
        duration_secs: 0,
        reason: reason.map(str::to_string),
        succeeded: false,
        verdict: StepVerdict::Failed,
        start_sha: None,
        log_tail: None,
    }
}

fn find_run_failed(pipeline_run_id: &str) -> Option<PipelineRunFailedPayload> {
    let events = ship_substrate::db::events::list_all_events().ok()?;
    events.into_iter().find_map(|e| {
        if e.entity_id == pipeline_run_id && e.event_type == event_types::PIPELINE_RUN_FAILED {
            serde_json::from_str(&e.payload_json).ok()
        } else {
            None
        }
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// No job in projection → retry returns false immediately.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestrator_review_retry_no_job_returns_false() {
    let _ship_dir = fresh_ship_dir();
    let kernel = make_kernel();
    let octx = make_octx();

    let payload = observed_for("j-rr-no-job", "rr-no-job", "review", None);
    let result = try_review_retry(&kernel, &octx, &payload).await;
    assert!(!result, "missing job must return false");
}

/// Step is Agent type (not Review) → retry returns false.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestrator_review_retry_non_review_step_returns_false() {
    let _ship_dir = fresh_ship_dir();
    let kernel = make_kernel();
    let octx = make_octx();

    let pipeline = Pipeline {
        steps: vec![agent_step("impl"), agent_step("not-a-review")],
        auto_merge: false,
    };
    seed_job(&kernel, "j-rr-agent", "rr-agent", pipeline).await;

    let payload = observed_for("j-rr-agent", "rr-agent", "not-a-review", None);
    let result = try_review_retry(&kernel, &octx, &payload).await;
    assert!(!result, "non-Review step must not retry");
}

/// Review step present but gated step has max_retries = 0 → no retry.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestrator_review_retry_zero_max_retries_returns_false() {
    let _ship_dir = fresh_ship_dir();
    let kernel = make_kernel();
    let octx = make_octx();

    let mut impl_step = agent_step("impl");
    impl_step.max_retries = Some(0);
    let pipeline = Pipeline {
        steps: vec![impl_step, review_step("review", "impl")],
        auto_merge: false,
    };
    seed_job(&kernel, "j-rr-zero", "rr-zero", pipeline).await;

    let payload = observed_for("j-rr-zero", "rr-zero", "review", None);
    let result = try_review_retry(&kernel, &octx, &payload).await;
    assert!(!result, "max_retries = 0 must not retry");
}

/// Review step has no `after` entries → retry returns false (no gated step).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestrator_review_retry_review_without_after_returns_false() {
    let _ship_dir = fresh_ship_dir();
    let kernel = make_kernel();
    let octx = make_octx();

    let mut orphan_review = review_step("review", "impl");
    orphan_review.after = vec![]; // no gated step declared
    let pipeline = Pipeline {
        steps: vec![agent_step("impl"), orphan_review],
        auto_merge: false,
    };
    seed_job(&kernel, "j-rr-orphan", "rr-orphan", pipeline).await;

    let payload = observed_for("j-rr-orphan", "rr-orphan", "review", None);
    let result = try_review_retry(&kernel, &octx, &payload).await;
    assert!(!result, "review step without after entries must not retry");
}

/// `emit_pipeline_run_failed_for_review` emits `pipeline.run.failed` when
/// the failing step is a Review type, carrying the review reason.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestrator_review_retry_emit_run_failed_carries_reason() {
    let _ship_dir = fresh_ship_dir();
    let kernel = make_kernel();

    let pipeline = Pipeline {
        steps: vec![agent_step("impl"), review_step("review", "impl")],
        auto_merge: false,
    };
    seed_job(&kernel, "j-rr-emit", "rr-emit", pipeline).await;

    let mut payload = observed_for("j-rr-emit", "rr-emit", "review", Some("quality bar not met"));
    payload.log_tail = Some("last 5 lines…".into());

    emit_pipeline_run_failed_for_review(&kernel, &payload).await;

    let emitted = find_run_failed("j-rr-emit")
        .expect("pipeline.run.failed must be emitted for a failed review step");
    assert_eq!(emitted.pipeline_run_id, "j-rr-emit");
    assert!(
        emitted.error.contains("quality bar not met"),
        "error must contain the review reason: {}",
        emitted.error
    );
    assert!(
        emitted.error.contains("last 5 lines"),
        "error must include the log tail: {}",
        emitted.error
    );
}

/// `emit_pipeline_run_failed_for_review` is a no-op when the step is not a
/// Review type — it must not emit `pipeline.run.failed` for agent steps.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn orchestrator_review_retry_emit_run_failed_noop_for_agent_step() {
    let _ship_dir = fresh_ship_dir();
    let kernel = make_kernel();

    let pipeline = Pipeline {
        steps: vec![agent_step("impl"), agent_step("also-agent")],
        auto_merge: false,
    };
    seed_job(&kernel, "j-rr-noop", "rr-noop", pipeline).await;

    let payload = observed_for("j-rr-noop", "rr-noop", "also-agent", Some("some reason"));
    emit_pipeline_run_failed_for_review(&kernel, &payload).await;

    assert!(
        find_run_failed("j-rr-noop").is_none(),
        "pipeline.run.failed must NOT be emitted for a non-review step"
    );
}
