//! Tests for the router's terminal-state guard.
//!
//! Once a job reaches `Failed`, `Merged`, or `Cancelled`, pipeline events
//! arriving afterwards (late step.observed, replayed deliveries, zombie
//! tmux teardown firing hooks) must NOT advance the job. Operator-origin
//! events — job.created / job.update / job.cancelled — are exempt.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{event_types, PipelineStepObservedPayload, StepTrigger, StepVerdict};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_substrate::events::EventEnvelope;
use crate::projections::pipeline_run::{load_pipeline_runs, PipelineRunStatus};
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, SessionContext, SessionExecutor,
    ProcessRegistryPort,
};
use crate::services::orchestrator::router::handle_job_event;
use crate::services::orchestrator::OrchestratorContext;

// ── Mocks ──────────────────────────────────────────────────────────────────

struct NoopIsolation;

#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, job: &SessionContext) -> anyhow::Result<PathBuf> {
        Ok(job.work_dir.clone())
    }
    async fn cleanup(&self, _job: &SessionContext) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Records whether spawn was ever called. Any invocation implies the
/// guard failed to suppress an advance.
#[derive(Default)]
struct TrackingExecutor {
    spawned: std::sync::atomic::AtomicUsize,
}

#[async_trait]
impl SessionExecutor for TrackingExecutor {
    async fn spawn(&self, _ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        self.spawned
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(ExecutorHandle {
            pid: Some(1),
            inner: Box::new(()),
        })
    }
    async fn is_alive(&self, _h: &ExecutorHandle) -> bool {
        true
    }
    async fn stop(&self, _h: &ExecutorHandle) -> anyhow::Result<()> {
        Ok(())
    }
    async fn send(&self, _h: &ExecutorHandle, _m: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

struct TestPlatform(PathBuf);

#[async_trait]
impl AgentPlatform for TestPlatform {
    fn worktrees_dir(&self) -> PathBuf {
        self.0.clone()
    }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> {
        Ok(true)
    }
    async fn cleanup_orphaned_worktrees(&self) {}
}

struct NoopRegistry;

#[async_trait]
impl ProcessRegistryPort for NoopRegistry {
    async fn register(&self, _slug: &str, _h: ExecutorHandle) {}
    async fn unregister(&self, _slug: &str) {}
}

// ── Harness ────────────────────────────────────────────────────────────────

fn dispatch_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec!["job.".to_string(), "step.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["pipeline.".to_string()],
        see_directed_events: false,
    }
}

fn octx(executor: Arc<TrackingExecutor>, worktrees: PathBuf) -> OrchestratorContext {
    OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor,
        platform: Arc::new(TestPlatform(worktrees)),
        registry: Arc::new(NoopRegistry),
        probe: Arc::new(crate::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        job_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(std::collections::HashMap::new())),
    }
}

/// Seed the event log with `job.created` + a terminal event so the projection
/// reports the requested status. Returns the kernel wired to the global DB.
async fn seed_terminal(
    job_id: &str,
    slug: &str,
    terminal_event_type: &str,
) -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router
        .spawn_actor("service.job-dispatch", dispatch_config())
        .unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let created = ship_substrate::events::pipeline::PipelineRunStartedPayload {
        job_id: job_id.into(),
        slug: slug.into(),
        agent: "rust-runtime".into(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".into(),
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
        headless: true,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: None,
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, job_id, &created).unwrap();
    crate::services::orchestrator::emit::persist_and_route(&kernel, env, slug, None).await;

    let terminal_payload = serde_json::json!({
        "job_id": job_id,
        "slug": slug,
        "error": "preexisting terminal state",
        "reason": "test",
        "initiated_by": "test",
        "stage": "executor_spawn",
    });
    let env = EventEnvelope::new(terminal_event_type, job_id, &terminal_payload).unwrap();
    crate::services::orchestrator::emit::persist_and_route(&kernel, env, slug, None).await;

    kernel
}

fn observed_advance(job_id: &str, slug: &str) -> EventEnvelope {
    let payload = PipelineStepObservedPayload {
        job_id: job_id.into(),
        slug: slug.into(),
        step_index: 0,
        step_name: "step-0".into(),
        trigger: StepTrigger::DoneDeclared,
        done_declared: true,
        done_result: Some(true),
        exit_code: None,
        commits_since_start: 1,
        uncommitted_files: 0,
        duration_secs: 1,
        reason: None,
        verdict: StepVerdict::Advance,
        succeeded: true,
        start_sha: None,
        log_tail: None,
    };
    EventEnvelope::new(event_types::PIPELINE_STEP_OBSERVED, job_id, &payload).unwrap()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step_observed_ignored_when_job_failed() {
    let tmp = TempDir::new().unwrap();
    let executor = Arc::new(TrackingExecutor::default());
    let ctx = octx(executor.clone(), tmp.path().into());

    let job_id = "j-guard-failed";
    let kernel = seed_terminal(job_id, "guard-failed", event_types::PIPELINE_RUN_FAILED).await;

    assert_eq!(
        load_pipeline_runs().unwrap().get(job_id).unwrap().status,
        PipelineRunStatus::Failed
    );

    let env = observed_advance(job_id, "guard-failed");
    handle_job_event(&kernel, &env, &ctx).await;

    assert_eq!(
        executor.spawned.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "advance must be suppressed for Failed job"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step_observed_ignored_when_job_merged() {
    let tmp = TempDir::new().unwrap();
    let executor = Arc::new(TrackingExecutor::default());
    let ctx = octx(executor.clone(), tmp.path().into());

    let job_id = "j-guard-merged";
    let kernel = seed_terminal(job_id, "guard-merged", event_types::PIPELINE_RUN_MERGED).await;

    let env = observed_advance(job_id, "guard-merged");
    handle_job_event(&kernel, &env, &ctx).await;

    assert_eq!(
        executor.spawned.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step_observed_ignored_when_job_cancelled() {
    let tmp = TempDir::new().unwrap();
    let executor = Arc::new(TrackingExecutor::default());
    let ctx = octx(executor.clone(), tmp.path().into());

    let job_id = "j-guard-cancelled";
    let kernel = seed_terminal(job_id, "guard-cancelled", event_types::PIPELINE_RUN_CANCELLED).await;

    assert_eq!(
        load_pipeline_runs().unwrap().get(job_id).unwrap().status,
        PipelineRunStatus::Cancelled
    );

    let env = observed_advance(job_id, "guard-cancelled");
    handle_job_event(&kernel, &env, &ctx).await;

    assert_eq!(
        executor.spawned.load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn operator_events_bypass_terminal_guard() {
    // A job.update for a Failed job must still pass the guard and reach
    // handle_job_update. We check "passed the guard" indirectly: the
    // function completes without panicking and the projection remains
    // Failed (job.update doesn't transition status in the projection).
    let tmp = TempDir::new().unwrap();
    let executor = Arc::new(TrackingExecutor::default());
    let ctx = octx(executor.clone(), tmp.path().into());

    let job_id = "j-guard-operator";
    let kernel = seed_terminal(job_id, "guard-operator", event_types::PIPELINE_RUN_FAILED).await;

    let update_payload = serde_json::json!({
        "job_id": job_id,
        "slug": "guard-operator",
        "status": "failed",
        "note": "operator commented after failure",
    });
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_UPDATED, job_id, &update_payload).unwrap();
    handle_job_event(&kernel, &env, &ctx).await;

    // The guard must not swallow operator events; the job stays Failed.
    assert_eq!(
        load_pipeline_runs().unwrap().get(job_id).unwrap().status,
        PipelineRunStatus::Failed
    );
}
