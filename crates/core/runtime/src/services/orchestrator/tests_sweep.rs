//! Tests for the coherence sweep — verifies drift detection and
//! resolution for dispatched-but-dead jobs and stray processes.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::supervisor::event_types as sup_events;
use crate::events::EventEnvelope;
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, ExitSignal, IsolationStrategy, JobContext, JobExecutor,
    ProcessRegistryPort,
};
use crate::services::orchestrator::sweep::run_sweep;
use crate::services::orchestrator::OrchestratorContext;

// ── Mocks ──────────────────────────────────────────────────────────────────

struct NoopIsolation;
#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, _job: &JobContext) -> anyhow::Result<PathBuf> {
        Ok(PathBuf::from("/tmp"))
    }
    async fn cleanup(&self, _job: &JobContext) -> anyhow::Result<()> {
        Ok(())
    }
}

struct NoopPlatform;
#[async_trait]
impl AgentPlatform for NoopPlatform {
    fn worktrees_dir(&self) -> PathBuf { PathBuf::from("/tmp/test-worktrees") }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> { Ok(true) }
    fn start_exit_handler(
        &self,
        _: Arc<Mutex<KernelRouter>>,
        _: tokio::sync::mpsc::UnboundedReceiver<ExitSignal>,
    ) {}
    fn start_liveness_sweep(&self, _: Arc<Mutex<KernelRouter>>) {}
    async fn cleanup_orphaned_worktrees(&self) {}
}

/// Executor that always returns None from resume_handle (process missing).
struct DeadExecutor;
#[async_trait]
impl JobExecutor for DeadExecutor {
    async fn spawn(&self, _: &JobContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("unused"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn resume_handle(&self, _: &str, _: Option<&str>, _: Option<&str>) -> Option<ExecutorHandle> {
        None
    }
}

/// Executor that always finds a live process.
struct AliveExecutor;
#[async_trait]
impl JobExecutor for AliveExecutor {
    async fn spawn(&self, _: &JobContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("unused"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { true }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn resume_handle(&self, _: &str, _: Option<&str>, _: Option<&str>) -> Option<ExecutorHandle> {
        Some(ExecutorHandle { pid: Some(999), inner: Box::new(()) })
    }
}

/// Executor with recording for stop calls.
struct RecordingStopExecutor {
    stop_calls: Arc<AtomicUsize>,
}
#[async_trait]
impl JobExecutor for RecordingStopExecutor {
    async fn spawn(&self, _: &JobContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("unused"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> {
        self.stop_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn resume_handle(&self, _: &str, _: Option<&str>, _: Option<&str>) -> Option<ExecutorHandle> {
        None
    }
}

struct EmptyRegistry;
#[async_trait]
impl ProcessRegistryPort for EmptyRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
}

/// Registry that reports specific live slugs and yields a handle on take.
struct StrayRegistry {
    slugs: Vec<String>,
    handle: Mutex<Option<ExecutorHandle>>,
}

#[async_trait]
impl ProcessRegistryPort for StrayRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
    async fn live_slugs(&self) -> Vec<String> { self.slugs.clone() }
    async fn take(&self, _: &str) -> Option<ExecutorHandle> {
        self.handle.lock().await.take()
    }
}

// ── Harness ────────────────────────────────────────────────────────────────

fn make_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = crate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    let config = ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec![
            "job.".to_string(),
            "step.".to_string(),
            "process.".to_string(),
            "supervisor.".to_string(),
        ],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string(), "step.".to_string()],
    };
    router.spawn_actor("service.job-dispatch", config).unwrap();
    Arc::new(Mutex::new(router))
}

fn make_octx(
    executor: Arc<dyn JobExecutor>,
    registry: Arc<dyn ProcessRegistryPort>,
) -> Arc<OrchestratorContext> {
    Arc::new(OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor,
        platform: Arc::new(NoopPlatform),
        registry,
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        job_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
    })
}

/// Seed a job in Dispatched status by emitting job.created + job.dispatched.
async fn seed_dispatched_job(
    kernel: &Arc<Mutex<KernelRouter>>,
    job_id: &str,
    slug: &str,
) {
    use crate::events::job::{event_types, JobCreatedPayload, JobDispatchedPayload};
    use crate::events::{CallerKind, EmitContext};

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };

    let created = JobCreatedPayload {
        job_id: job_id.to_string(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: None,
        plan_id: None,
        depends_on: None,
        model: None,
        provider: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
    };
    let env = EventEnvelope::new(event_types::JOB_CREATED, job_id, &created)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();

    let dispatched = JobDispatchedPayload {
        job_id: job_id.to_string(),
        worktree: format!("/tmp/test-worktrees/{slug}"),
        pid: None,
        provider_meta: None,
    };
    let env = EventEnvelope::new(event_types::JOB_DISPATCHED, job_id, &dispatched)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

/// Seed a job in Failed status.
async fn seed_failed_job(
    kernel: &Arc<Mutex<KernelRouter>>,
    job_id: &str,
    slug: &str,
) {
    use crate::events::job::{event_types, JobCreatedPayload, JobFailedPayload};
    use crate::events::{CallerKind, EmitContext};

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    };

    let created = JobCreatedPayload {
        job_id: job_id.to_string(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: None,
        plan_id: None,
        depends_on: None,
        model: None,
        provider: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
    };
    let env = EventEnvelope::new(event_types::JOB_CREATED, job_id, &created)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();

    let failed = JobFailedPayload {
        job_id: job_id.to_string(),
        error: "seed failure".to_string(),
    };
    let env = EventEnvelope::new(event_types::JOB_FAILED, job_id, &failed)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

fn find_events_by_type(event_type: &str) -> Vec<EventEnvelope> {
    crate::db::events::list_all_events()
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.event_type == event_type)
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatched_job_no_process_gets_marked_failed() {
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(Arc::new(DeadExecutor), Arc::new(EmptyRegistry));

    seed_dispatched_job(&kernel, "j-sweep-1", "sweep-1").await;
    run_sweep(&kernel, &octx).await.unwrap();

    // Verify drift_detected
    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        drift.iter().any(|e| e.entity_id == "sweep-1"),
        "drift_detected emitted for sweep-1"
    );

    // Verify job.failed
    let failed = find_events_by_type("job.failed");
    let sweep_fail = failed.iter().find(|e| {
        let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
        p["error"].as_str().unwrap_or_default().contains("coherence sweep")
    });
    assert!(sweep_fail.is_some(), "job.failed with coherence sweep error emitted");

    // Verify coherence_restored
    let restored = find_events_by_type(sup_events::SUPERVISOR_COHERENCE_RESTORED);
    assert!(
        restored.iter().any(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["action"] == "marked_failed"
        }),
        "coherence_restored with marked_failed action"
    );

    // Verify sweep_completed counts
    let completed = find_events_by_type(sup_events::SUPERVISOR_SWEEP_COMPLETED);
    let last = completed.last().expect("sweep_completed emitted");
    let p: serde_json::Value = serde_json::from_str(&last.payload_json).unwrap();
    assert_eq!(p["drift_count"], 1);
    assert_eq!(p["actions_taken"], 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatched_job_with_live_process_not_touched() {
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(Arc::new(AliveExecutor), Arc::new(EmptyRegistry));

    seed_dispatched_job(&kernel, "j-sweep-2", "sweep-2").await;
    run_sweep(&kernel, &octx).await.unwrap();

    // No drift for this slug
    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        !drift.iter().any(|e| e.entity_id == "sweep-2"),
        "no drift for live process"
    );

    let completed = find_events_by_type(sup_events::SUPERVISOR_SWEEP_COMPLETED);
    let last = completed.last().expect("sweep_completed emitted");
    let p: serde_json::Value = serde_json::from_str(&last.payload_json).unwrap();
    assert_eq!(p["drift_count"], 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn terminal_jobs_are_skipped() {
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(Arc::new(DeadExecutor), Arc::new(EmptyRegistry));

    seed_failed_job(&kernel, "j-sweep-3", "sweep-3").await;
    run_sweep(&kernel, &octx).await.unwrap();

    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        !drift.iter().any(|e| e.entity_id == "sweep-3"),
        "no drift for terminal job"
    );

    let completed = find_events_by_type(sup_events::SUPERVISOR_SWEEP_COMPLETED);
    let last = completed.last().expect("sweep_completed emitted");
    let p: serde_json::Value = serde_json::from_str(&last.payload_json).unwrap();
    assert_eq!(p["drift_count"], 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stray_process_gets_killed() {
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let stop_calls = Arc::new(AtomicUsize::new(0));

    let registry = Arc::new(StrayRegistry {
        slugs: vec!["orphan-1".to_string()],
        handle: Mutex::new(Some(ExecutorHandle {
            pid: Some(7777),
            inner: Box::new(()),
        })),
    });

    let octx = make_octx(
        Arc::new(RecordingStopExecutor { stop_calls: stop_calls.clone() }),
        registry,
    );

    // No jobs seeded — the "orphan-1" slug in the registry has no job.
    run_sweep(&kernel, &octx).await.unwrap();

    assert_eq!(stop_calls.load(Ordering::SeqCst), 1, "executor.stop called for stray");

    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        drift.iter().any(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["actual_state"] == "stray_process" && p["slug"] == "orphan-1"
        }),
        "drift_detected for stray process"
    );

    let restored = find_events_by_type(sup_events::SUPERVISOR_COHERENCE_RESTORED);
    assert!(
        restored.iter().any(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["action"] == "process_killed" && p["slug"] == "orphan-1"
        }),
        "coherence_restored with process_killed"
    );

    // The kill itself must land on the process.* audit stream — distinct
    // from the supervisor.coherence_restored signal.
    use crate::events::process::event_types as proc_events;
    let killed = find_events_by_type(proc_events::PROCESS_KILLED);
    assert!(
        killed.iter().any(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["slug"] == "orphan-1" && p["actor"] == "supervisor"
        }),
        "process.killed emitted with actor=supervisor for stray kill"
    );
}
