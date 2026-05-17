//! Tests for `job.spawn_failed` emission from the dispatch path.
//!
//! Each of the dispatch failure points — `isolation.prepare` and
//! `executor.spawn` — must emit a structured `job.spawn_failed` event
//! carrying the correct `stage` field, so the projection transitions to
//! Failed (instead of silently hanging the job in Pending forever).

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{event_types, PipelineRunStartedPayload, PipelineStepSpawnFailedPayload};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::projections::job::{load_jobs, JobStatus};
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, JobContext, JobExecutor,
    ProcessRegistryPort,
};
use crate::services::orchestrator::dispatch::dispatch_job_internal;
use crate::services::orchestrator::OrchestratorContext;

// ── Mocks ──────────────────────────────────────────────────────────────────

struct FailingIsolation;

#[async_trait]
impl IsolationStrategy for FailingIsolation {
    async fn prepare(&self, _job: &JobContext) -> anyhow::Result<PathBuf> {
        Err(anyhow::anyhow!("isolation boom: worktree conflict"))
    }
    async fn cleanup(&self, _job: &JobContext) -> anyhow::Result<()> {
        Ok(())
    }
}

struct OkIsolation(PathBuf);

#[async_trait]
impl IsolationStrategy for OkIsolation {
    async fn prepare(&self, _job: &JobContext) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.0).ok();
        Ok(self.0.clone())
    }
    async fn cleanup(&self, _job: &JobContext) -> anyhow::Result<()> {
        Ok(())
    }
}

struct FailingExecutor;

#[async_trait]
impl JobExecutor for FailingExecutor {
    async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("spawn boom: tmux missing"))
    }
    async fn is_alive(&self, _h: &ExecutorHandle) -> bool {
        false
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

fn job_payload(job_id: &str, slug: &str) -> PipelineRunStartedPayload {
    PipelineRunStartedPayload {
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
        baseline_sha: None,
    }
}

fn dispatch_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec!["job.".to_string(), "step.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
    }
}

fn octx(
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

fn find_spawn_failed(job_id: &str) -> Option<PipelineStepSpawnFailedPayload> {
    let events = ship_substrate::db::events::list_all_events().ok()?;
    events.into_iter().find_map(|e| {
        if e.entity_id == job_id && e.event_type == event_types::PIPELINE_STEP_SPAWN_FAILED {
            serde_json::from_str(&e.payload_json).ok()
        } else {
            None
        }
    })
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn isolation_prepare_failure_emits_spawn_failed() {
    let tmp = TempDir::new().unwrap();
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base.clone()).unwrap();
    router
        .spawn_actor("service.job-dispatch", dispatch_config())
        .unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let ctx = octx(
        Arc::new(FailingIsolation),
        Arc::new(FailingExecutor),
        tmp.path().join("worktrees"),
    );

    let payload = job_payload("j-spawn-iso", "spawn-iso");
    // Seed job.created so the projection reaches Pending first.
    let created = ship_substrate::events::EventEnvelope::new(
        event_types::PIPELINE_RUN_STARTED,
        &payload.job_id,
        &payload,
    )
    .unwrap();
    crate::services::orchestrator::emit::persist_and_route(
        &kernel,
        created,
        &payload.slug,
        None,
    )
    .await;

    dispatch_job_internal(&kernel, &payload, &ctx).await;

    let emitted = find_spawn_failed(&payload.job_id).expect("job.spawn_failed emitted");
    assert_eq!(emitted.stage, "isolation_prepare");
    assert_eq!(emitted.slug, payload.slug);
    assert!(
        emitted.error.contains("isolation boom"),
        "error preserved: {}",
        emitted.error
    );

    let jobs = load_jobs().unwrap();
    assert_eq!(
        jobs.get(&payload.job_id).unwrap().status,
        JobStatus::Failed,
        "projection must transition to Failed after spawn_failed"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn executor_spawn_failure_emits_spawn_failed() {
    let tmp = TempDir::new().unwrap();
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base.clone()).unwrap();
    router
        .spawn_actor("service.job-dispatch", dispatch_config())
        .unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let worktree = tmp.path().join("worktrees/spawn-exec");
    let ctx = octx(
        Arc::new(OkIsolation(worktree.clone())),
        Arc::new(FailingExecutor),
        tmp.path().join("worktrees"),
    );

    let payload = job_payload("j-spawn-exec", "spawn-exec");
    let created = ship_substrate::events::EventEnvelope::new(
        event_types::PIPELINE_RUN_STARTED,
        &payload.job_id,
        &payload,
    )
    .unwrap();
    crate::services::orchestrator::emit::persist_and_route(
        &kernel,
        created,
        &payload.slug,
        None,
    )
    .await;

    dispatch_job_internal(&kernel, &payload, &ctx).await;

    let emitted = find_spawn_failed(&payload.job_id).expect("job.spawn_failed emitted");
    assert_eq!(emitted.stage, "executor_spawn");
    assert!(
        emitted.error.contains("spawn boom"),
        "error preserved: {}",
        emitted.error
    );

    let jobs = load_jobs().unwrap();
    assert_eq!(
        jobs.get(&payload.job_id).unwrap().status,
        JobStatus::Failed
    );
}
