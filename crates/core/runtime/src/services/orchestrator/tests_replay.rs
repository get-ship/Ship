//! Reconcile is observation-only.
//!
//! The historic bug: `reconcile_dispatched_inner` emitted `job.step_failed`
//! the first time `resume_handle` returned `None`. A single missed probe
//! killed jobs whose agents were slow to surface after a daemon restart.
//! New contract verified here: a missing process yields one
//! `executor.session_destroyed` observation, no terminal job event, and
//! the projection stays `Dispatched` — the liveness sweep owns the
//! eventual transition.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, ExitSignal, IsolationStrategy, JobContext, JobExecutor,
    ProcessRegistryPort,
};
use crate::services::orchestrator::replay::reconcile_dispatched_jobs;
use crate::services::orchestrator::OrchestratorContext;

struct NoopIsolation;
#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, _: &JobContext) -> anyhow::Result<PathBuf> { Ok(PathBuf::from("/tmp")) }
    async fn cleanup(&self, _: &JobContext) -> anyhow::Result<()> { Ok(()) }
}

struct NoopPlatform;
#[async_trait]
impl AgentPlatform for NoopPlatform {
    fn worktrees_dir(&self) -> PathBuf { PathBuf::from("/tmp/test-worktrees-replay") }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> { Ok(true) }
    fn start_exit_handler(&self, _: Arc<Mutex<KernelRouter>>, _: tokio::sync::mpsc::UnboundedReceiver<ExitSignal>) {}
    fn start_liveness_sweep(&self, _: Arc<Mutex<KernelRouter>>) {}
    async fn cleanup_orphaned_worktrees(&self) {}
}

/// `resume_handle` always returns `None` — simulates a missing process.
struct MissingExecutor;
#[async_trait]
impl JobExecutor for MissingExecutor {
    async fn spawn(&self, _: &JobContext) -> anyhow::Result<ExecutorHandle> { Err(anyhow::anyhow!("unused")) }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn resume_handle(&self, _: &str, _: Option<&str>, _: Option<&str>) -> Option<ExecutorHandle> { None }
}

struct EmptyRegistry;
#[async_trait]
impl ProcessRegistryPort for EmptyRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
}

fn make_octx() -> Arc<OrchestratorContext> {
    Arc::new(OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor: Arc::new(MissingExecutor),
        platform: Arc::new(NoopPlatform),
        registry: Arc::new(EmptyRegistry),
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        job_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
    })
}

fn make_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = crate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router
        .spawn_actor(
            "service.job-dispatch",
            ActorConfig {
                namespace: "service.job-dispatch".to_string(),
                write_namespaces: vec![
                    "job.".to_string(),
                    "step.".to_string(),
                    "process.".to_string(),
                    "supervisor.".to_string(),
                    "executor.".to_string(),
                ],
                read_namespaces: vec![],
                subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
            },
        )
        .unwrap();
    Arc::new(Mutex::new(router))
}

async fn seed_dispatched_job(kernel: &Arc<Mutex<KernelRouter>>, job_id: &str, slug: &str) {
    use crate::events::job::{event_types, JobCreatedPayload, JobDispatchedPayload};
    use crate::events::{CallerKind, EmitContext, EventEnvelope};
    let ctx = EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, workspace_id: None, session_id: None };
    let created = JobCreatedPayload {
        job_id: job_id.to_string(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".to_string(),
        plan_id: None, depends_on: None, model: None, provider: None,
        pipeline: None, steps: None, pipeline_name: None, vars: None,
        spec_content: None, step_specs: None, headless: false,
    };
    let env = EventEnvelope::new(event_types::JOB_CREATED, job_id, &created).unwrap().with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
    let dispatched = JobDispatchedPayload {
        job_id: job_id.to_string(),
        worktree: format!("/tmp/test-worktrees-replay/{slug}"),
        pid: None,
        provider_meta: None,
    };
    let env = EventEnvelope::new(event_types::JOB_DISPATCHED, job_id, &dispatched).unwrap().with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

/// Missing process → observational event only. Reconcile never terminates.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconcile_dispatched_emits_session_destroyed_not_step_failed() {
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx();

    let slug = "replay-missing";
    seed_dispatched_job(&kernel, "j-replay-1", slug).await;

    reconcile_dispatched_jobs(&kernel, &octx).await;

    let all = crate::db::events::list_all_events().unwrap_or_default();

    let destroyed: Vec<_> = all
        .iter()
        .filter(|e| e.event_type == crate::events::executor::EXECUTOR_SESSION_DESTROYED && e.entity_id == slug)
        .collect();
    assert_eq!(destroyed.len(), 1, "one observation event per reconcile pass");
    let payload: serde_json::Value = serde_json::from_str(&destroyed[0].payload_json).unwrap();
    assert_eq!(payload["cause"].as_str(), Some("external"));
    assert_eq!(payload["slug"].as_str(), Some(slug));

    let terminal = all
        .iter()
        .filter(|e| e.entity_id == "j-replay-1")
        .any(|e| e.event_type == "job.step_failed" || e.event_type == "job.failed");
    assert!(!terminal, "reconcile must not emit any terminal job event");

    let jobs = crate::projections::job::load_jobs().unwrap();
    let job = jobs.get("j-replay-1").expect("seeded job present");
    assert_eq!(
        job.status,
        crate::projections::job::JobStatus::Dispatched,
        "orphaned job stays Dispatched; liveness sweep owns the terminal call"
    );
    assert!(job.error.is_none());
}
