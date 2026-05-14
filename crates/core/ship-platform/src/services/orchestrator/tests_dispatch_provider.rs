//! Tests for first-step provider plumbing in the job-dispatch path.
//!
//! `dispatch_job_internal` must build the `LaunchProfile` from the
//! first pipeline step's `provider` when present, not only from the
//! job-level `payload.provider`. This mirrors how `advance_pipeline`
//! resolves provider for steps 2+, so the two dispatch paths agree.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_substrate::events::job::{
    event_types, JobCreatedPayload, Pipeline, PipelineStep, StepType,
};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, JobContext, JobExecutor,
    ProcessRegistryPort,
};
use crate::services::orchestrator::dispatch::dispatch_job_internal;
use crate::services::orchestrator::OrchestratorContext;

struct OkIsolation(PathBuf);

#[async_trait]
impl IsolationStrategy for OkIsolation {
    async fn prepare(&self, _: &JobContext) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.0).ok();
        Ok(self.0.clone())
    }
    async fn cleanup(&self, _: &JobContext) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Records the (provider, model) it was asked to spawn, then errors so
/// dispatch skips all post-spawn work (monitors, events, etc).
struct RecordingExecutor {
    captured: Arc<StdMutex<Option<(Option<String>, Option<String>)>>>,
}

#[async_trait]
impl JobExecutor for RecordingExecutor {
    async fn spawn(&self, ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
        *self.captured.lock().unwrap() =
            Some((ctx.launch.provider.clone(), ctx.launch.model.clone()));
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

struct NoopRegistry;

#[async_trait]
impl ProcessRegistryPort for NoopRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
}

fn dispatch_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec!["job.".to_string(), "step.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
    }
}

fn octx_with(
    isolation: Arc<dyn IsolationStrategy>,
    executor: Arc<dyn JobExecutor>,
    worktrees: PathBuf,
) -> OrchestratorContext {
    OrchestratorContext {
        isolation,
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

fn payload_with_first_step(
    job_id: &str,
    slug: &str,
    step_provider: Option<&str>,
    step_model: Option<&str>,
    payload_provider: Option<&str>,
    payload_model: Option<&str>,
) -> JobCreatedPayload {
    let first = PipelineStep {
        id: "implement".into(),
        step_type: StepType::Agent,
        agent: Some("rust-lane".into()),
        goal: None,
        after: vec![],
        model: step_model.map(str::to_string),
        provider: step_provider.map(str::to_string),
        command: None,
        max_retries: None,
        gate: None,
        produces: vec![],
        consumes: vec![],
        promotes: vec![],
        fanout: None,
    };
    JobCreatedPayload {
        job_id: job_id.into(),
        slug: slug.into(),
        agent: "rust-lane".into(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".into(),
        source_spec_path: None,
        plan_id: None,
        model: payload_model.map(str::to_string),
        provider: payload_provider.map(str::to_string),
        depends_on: None,
        pipeline: None,
        steps: Some(Pipeline { steps: vec![first], auto_merge: false }),
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: true,
        baseline_sha: None,
    }
}

async fn run_dispatch(
    payload: &JobCreatedPayload,
    tmp: &TempDir,
) -> Option<(Option<String>, Option<String>)> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router.spawn_actor("service.job-dispatch", dispatch_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let worktree = tmp.path().join(format!("worktrees/{}", payload.slug));
    let captured = Arc::new(StdMutex::new(None));
    let ctx = octx_with(
        Arc::new(OkIsolation(worktree.clone())),
        Arc::new(RecordingExecutor { captured: captured.clone() }),
        tmp.path().join("worktrees"),
    );

    let created = ship_substrate::events::EventEnvelope::new(
        event_types::JOB_CREATED, &payload.job_id, payload,
    ).unwrap();
    crate::services::orchestrator::emit::persist_and_route(
        &kernel, created, &payload.slug, None,
    ).await;

    dispatch_job_internal(&kernel, payload, &ctx).await;
    captured.lock().unwrap().clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_step_provider_is_used_when_payload_provider_none() {
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
        "step.provider must flow into LaunchProfile.provider on first-step dispatch",
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
async fn step_provider_takes_precedence_over_payload_provider() {
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
        "step.provider must win over payload.provider when both are set",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn payload_provider_is_used_when_step_provider_none() {
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
        "payload.provider must be used when step.provider is absent",
    );
}
