//! Tests for first-step harness plumbing in the pipeline-run-dispatch path.
//!
//! `dispatch_pipeline_run_internal` must build the `LaunchProfile` from the
//! first pipeline step's `harness` when present, not only from the
//! job-level `payload.harness`. This mirrors how `advance_pipeline`
//! resolves harness for steps 2+, so the two dispatch paths agree.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{
    event_types, PipelineRunStartedPayload, Pipeline, PipelineStep, StepType,
};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_code::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, SessionContext, SessionExecutor,
};
use durable_execution::{InMemoryJournalStore, JournalStore};
use crate::orchestrator::dispatch::dispatch_pipeline_run_internal;
use crate::orchestrator::OrchestratorContext;
use super::tests_fakes::NoopRegistry;

/// Executor that succeeds every spawn with pid=42.
struct SucceedingExecutor;

#[async_trait]
impl SessionExecutor for SucceedingExecutor {
    async fn spawn(&self, _ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Ok(ExecutorHandle { pid: Some(42), inner: Box::new(()) })
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
}

struct OkIsolation(PathBuf);

#[async_trait]
impl IsolationStrategy for OkIsolation {
    async fn prepare(&self, _: &SessionContext) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.0).ok();
        Ok(self.0.clone())
    }
    async fn cleanup(&self, _: &SessionContext) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Records the (harness, model) it was asked to spawn, then errors so
/// dispatch skips all post-spawn work (monitors, events, etc).
type CapturedSpawn = Option<(Option<String>, Option<String>)>;

struct RecordingExecutor {
    captured: Arc<StdMutex<CapturedSpawn>>,
}

#[async_trait]
impl SessionExecutor for RecordingExecutor {
    async fn spawn(&self, ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        *self.captured.lock().unwrap() =
            Some((ctx.launch.harness.clone(), ctx.launch.model.clone()));
        Err(anyhow::anyhow!("recording executor: refusing to spawn"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
}

struct TestPlatform(PathBuf);

#[async_trait]
impl AgentPlatform for TestPlatform {
    fn worktrees_dir(&self) -> PathBuf { self.0.clone() }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> { Ok(true) }
    async fn cleanup_orphaned_worktrees(&self) {}
}

fn dispatch_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.orchestrator".to_string(),
        write_namespaces: vec!["job.".to_string(), "step.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["pipeline.".to_string()],
        see_directed_events: false,
    }
}

fn octx_with(
    isolation: Arc<dyn IsolationStrategy>,
    executor: Arc<dyn SessionExecutor>,
    worktrees: PathBuf,
) -> OrchestratorContext {
    octx_with_journal_store(
        isolation,
        executor,
        worktrees,
        Arc::new(durable_execution::InMemoryJournalStore::new()),
    )
}

fn octx_with_journal_store(
    isolation: Arc<dyn IsolationStrategy>,
    executor: Arc<dyn SessionExecutor>,
    worktrees: PathBuf,
    journal_store: Arc<dyn JournalStore>,
) -> OrchestratorContext {
    OrchestratorContext {
        isolation,
        executor,
        platform: Arc::new(TestPlatform(worktrees)),
        registry: Arc::new(NoopRegistry),
        probe: Arc::new(ship_code::process_tree_probe::SysinfoProcessTreeProbe::new()),
        spawn_policy: Arc::new(crate::orchestrator::spawn_policy::NoopSpawnPolicy),
        projections: Arc::new(crate::orchestrator::projection_reader::ProjectionsReader),
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        run_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        journal_store,
    }
}

fn payload_with_first_step(
    pipeline_run_id: &str,
    slug: &str,
    step_harness: Option<&str>,
    step_model: Option<&str>,
    payload_harness: Option<&str>,
    payload_model: Option<&str>,
) -> PipelineRunStartedPayload {
    let first = PipelineStep {
        id: "implement".into(),
        step_type: StepType::Agent,
        agent: Some("rust-lane".into()),
        goal: None,
        after: vec![],
        model: step_model.map(str::to_string),
        harness: step_harness.map(str::to_string),
        command: None,
        max_retries: None,
        gate: None,
        produces: vec![],
        consumes: vec![],
        promotes: vec![],
        fanout: None,
            invocation: None,
    };
    PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.into(),
        slug: slug.into(),
        agent: "rust-lane".into(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".into(),
        source_spec_path: None,
        plan_id: None,
        model: payload_model.map(str::to_string),
        harness: payload_harness.map(str::to_string),
        depends_on: None,
        pipeline: None,
        steps: Some(Pipeline { steps: vec![first], auto_merge: false }),
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
    }
}

async fn run_dispatch(
    payload: &PipelineRunStartedPayload,
    tmp: &TempDir,
) -> Option<(Option<String>, Option<String>)> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router.spawn_actor("service.orchestrator", dispatch_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let worktree = tmp.path().join(format!("worktrees/{}", payload.slug));
    let captured = Arc::new(StdMutex::new(None));
    let ctx = octx_with(
        Arc::new(OkIsolation(worktree.clone())),
        Arc::new(RecordingExecutor { captured: captured.clone() }),
        tmp.path().join("worktrees"),
    );

    let created = ship_substrate::events::EventEnvelope::new(
        event_types::PIPELINE_RUN_STARTED, &payload.pipeline_run_id, payload,
    ).unwrap();
    ship_runner::emit::persist_and_route(
        &kernel, created, &payload.slug,
    ).await;

    dispatch_pipeline_run_internal(&kernel, payload, &ctx).await;
    captured.lock().unwrap().clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_step_harness_is_used_when_payload_harness_none() {
    let tmp = TempDir::new().unwrap();
    let payload = payload_with_first_step(
        "j-prov-step", "prov-step",
        Some("codex"), None,
        None, None,
    );
    let captured = run_dispatch(&payload, &tmp).await.expect("executor called");
    assert_eq!(
        captured.0.as_deref(),
        Some("codex"),
        "step.harness must flow into LaunchProfile.harness on first-step dispatch",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_step_model_is_used_when_payload_model_none() {
    let tmp = TempDir::new().unwrap();
    let payload = payload_with_first_step(
        "j-model-step", "model-step",
        None, Some("gpt-5-codex"),
        None, None,
    );
    let captured = run_dispatch(&payload, &tmp).await.expect("executor called");
    assert_eq!(
        captured.1.as_deref(),
        Some("gpt-5-codex"),
        "step.model must flow into LaunchProfile.model on first-step dispatch",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn step_harness_takes_precedence_over_payload_harness() {
    let tmp = TempDir::new().unwrap();
    let payload = payload_with_first_step(
        "j-prov-prec", "prov-prec",
        Some("codex"), None,
        Some("claude"), None,
    );
    let captured = run_dispatch(&payload, &tmp).await.expect("executor called");
    assert_eq!(
        captured.0.as_deref(),
        Some("codex"),
        "step.harness must win over payload.harness when both are set",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn payload_harness_is_used_when_step_harness_none() {
    let tmp = TempDir::new().unwrap();
    let payload = payload_with_first_step(
        "j-prov-fallback", "prov-fallback",
        None, None,
        Some("claude"), None,
    );
    let captured = run_dispatch(&payload, &tmp).await.expect("executor called");
    assert_eq!(
        captured.0.as_deref(),
        Some("claude"),
        "payload.harness must be used when step.harness is absent",
    );
}

// ── T1 / T2: shadow journal behavioral tests ─────────────────────────────────

/// T1: a successful agent-spawn produces a three-entry journal and persists it.
///
/// The shadow-journal block (C2) calls `durable_execution::execute` with
/// `ShadowDispatch` and `DispatchProgram`, then saves the resulting journal.
/// The journal MUST have three entries (one per op in `DispatchProgram::run`)
/// in deterministic order and MUST NOT be suspended.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_agent_spawn_creates_three_entry_journal() {
    let tmp = TempDir::new().unwrap();
    let store: Arc<dyn JournalStore> = Arc::new(InMemoryJournalStore::new());

    let pipeline_run_id = "j-journal-t1";
    let payload = payload_with_first_step(
        pipeline_run_id, "journal-t1",
        None, None, None, None,
    );

    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = ship_substrate::events::kernel_router::KernelRouter::new(base).unwrap();
    router.spawn_actor("service.orchestrator", dispatch_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let created = ship_substrate::events::EventEnvelope::new(
        event_types::PIPELINE_RUN_STARTED, &payload.pipeline_run_id, &payload,
    ).unwrap();
    ship_runner::emit::persist_and_route(&kernel, created, &payload.slug).await;

    let worktree = tmp.path().join(format!("worktrees/{}", payload.slug));
    let ctx = octx_with_journal_store(
        Arc::new(OkIsolation(worktree)),
        Arc::new(SucceedingExecutor),
        tmp.path().join("worktrees"),
        store.clone(),
    );

    dispatch_pipeline_run_internal(&kernel, &payload, &ctx).await;

    let journal = store.load(pipeline_run_id).await
        .expect("store load ok")
        .expect("journal must be present after successful agent spawn");

    let entries = journal.entries();
    assert_eq!(
        entries.len(), 3,
        "DispatchProgram produces exactly three ops; got {}: {:?}",
        entries.len(),
        entries.iter().map(|e| &e.op).collect::<Vec<_>>(),
    );
    assert_eq!(entries[0].op, "pipeline.prepare_isolation");
    assert_eq!(entries[1].op, "pipeline.spawn_agent");
    assert_eq!(entries[2].op, "pipeline.emit_dispatched");
    assert!(
        !journal.is_suspended(),
        "journal must not be suspended after a complete agent-spawn"
    );
}

/// T2: a failed spawn (executor returns Err) must not write anything to the store.
///
/// The shadow-journal block only runs after a successful `executor.spawn`. When
/// the executor fails, `dispatch_pipeline_run_internal` returns early after
/// emitting spawn_failed. The store must remain empty.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_failed_spawn_does_not_save_journal() {
    let tmp = TempDir::new().unwrap();
    let store: Arc<dyn JournalStore> = Arc::new(InMemoryJournalStore::new());

    let pipeline_run_id = "j-journal-t2";
    let payload = payload_with_first_step(
        pipeline_run_id, "journal-t2",
        None, None, None, None,
    );

    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = ship_substrate::events::kernel_router::KernelRouter::new(base).unwrap();
    router.spawn_actor("service.orchestrator", dispatch_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let created = ship_substrate::events::EventEnvelope::new(
        event_types::PIPELINE_RUN_STARTED, &payload.pipeline_run_id, &payload,
    ).unwrap();
    ship_runner::emit::persist_and_route(&kernel, created, &payload.slug).await;

    // RecordingExecutor always returns Err — simulates a failing spawn.
    let captured = Arc::new(StdMutex::new(None));
    let worktree = tmp.path().join(format!("worktrees/{}", payload.slug));
    let ctx = octx_with_journal_store(
        Arc::new(OkIsolation(worktree)),
        Arc::new(RecordingExecutor { captured }),
        tmp.path().join("worktrees"),
        store.clone(),
    );

    dispatch_pipeline_run_internal(&kernel, &payload, &ctx).await;

    let result = store.load(pipeline_run_id).await.expect("store load ok");
    assert!(
        result.is_none(),
        "no journal must be saved when executor.spawn() fails"
    );
}
