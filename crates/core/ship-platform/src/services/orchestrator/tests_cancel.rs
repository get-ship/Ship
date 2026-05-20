//! Tests for `job.cancelled` handling — the orchestrator must terminate
//! the registered process, clean up the tmux session, and emit
//! `process.killed` for audit.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_substrate::events::pipeline::{event_types as job_event_types, PipelineRunCancelledPayload};
use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_substrate::events::process::event_types as process_event_types;
use ship_substrate::events::process::ProcessKilledPayload;
use ship_substrate::events::EventEnvelope;
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, SessionContext, SessionExecutor, ProcessRegistryPort,
};
use crate::services::orchestrator::router::handle_job_event;
use crate::services::orchestrator::OrchestratorContext;

// ── Mocks ──────────────────────────────────────────────────────────────────

struct RecordingExecutor {
    stop_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl SessionExecutor for RecordingExecutor {
    async fn spawn(&self, _ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("not used"))
    }
    async fn is_alive(&self, _h: &ExecutorHandle) -> bool { true }
    async fn stop(&self, _h: &ExecutorHandle) -> anyhow::Result<()> {
        self.stop_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn send(&self, _h: &ExecutorHandle, _m: &str) -> anyhow::Result<()> { Ok(()) }
}

struct RecordingPlatform {
    cleanup_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl AgentPlatform for RecordingPlatform {
    fn worktrees_dir(&self) -> PathBuf { PathBuf::from("/tmp/test-worktrees") }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {
        self.cleanup_calls.fetch_add(1, Ordering::SeqCst);
    }
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> { Ok(true) }
    async fn cleanup_orphaned_worktrees(&self) {}
}

/// Registry seeded with a single (slug → handle) entry. `take` returns
/// the handle once, mirroring how the real registry drains on stop.
struct SeededRegistry {
    handle: Mutex<Option<ExecutorHandle>>,
    taken_for: Mutex<Option<String>>,
}

impl SeededRegistry {
    fn new(handle: ExecutorHandle) -> Self {
        Self {
            handle: Mutex::new(Some(handle)),
            taken_for: Mutex::new(None),
        }
    }
}

#[async_trait]
impl ProcessRegistryPort for SeededRegistry {
    async fn register(&self, _slug: &str, _h: ExecutorHandle) {}
    async fn unregister(&self, _slug: &str) {}
    async fn take(&self, slug: &str) -> Option<ExecutorHandle> {
        let mut taken = self.taken_for.lock().await;
        *taken = Some(slug.to_string());
        self.handle.lock().await.take()
    }
}

// ── Harness ────────────────────────────────────────────────────────────────

fn dispatch_actor_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec![
            "job.".to_string(),
            "step.".to_string(),
            "process.".to_string(),
        ],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["pipeline.".to_string()],
        see_directed_events: false,
    }
}

fn find_process_killed(slug: &str) -> Option<ProcessKilledPayload> {
    ship_substrate::db::events::list_all_events()
        .ok()?
        .into_iter()
        .find_map(|e| {
            if e.entity_id == slug && e.event_type == process_event_types::PROCESS_KILLED {
                serde_json::from_str(&e.payload_json).ok()
            } else {
                None
            }
        })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_stops_registered_handle_and_emits_process_killed() {
    let _tmp = TempDir::new().unwrap();
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base.clone()).unwrap();
    router
        .spawn_actor("service.job-dispatch", dispatch_actor_config())
        .unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let stop_calls = Arc::new(AtomicUsize::new(0));
    let cleanup_calls = Arc::new(AtomicUsize::new(0));

    let handle = ExecutorHandle {
        pid: Some(42),
        inner: Box::new(()),
    };

    let octx = OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor: Arc::new(RecordingExecutor { stop_calls: stop_calls.clone() }),
        platform: Arc::new(RecordingPlatform { cleanup_calls: cleanup_calls.clone() }),
        registry: Arc::new(SeededRegistry::new(handle)),
        probe: Arc::new(crate::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        job_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(std::collections::HashMap::new())),
    };

    let job_id = "j-cancel-1";
    let slug = "cancel-1";
    let payload = PipelineRunCancelledPayload {
        job_id: job_id.to_string(),
        slug: slug.to_string(),
        reason: "operator test".into(),
        requested_by: "operator".into(),
    };
    let envelope = EventEnvelope::new(job_event_types::PIPELINE_RUN_CANCELLED, job_id, &payload)
        .unwrap()
        .with_actor_id("test");

    handle_job_event(&kernel, &envelope, &octx).await;

    assert_eq!(stop_calls.load(Ordering::SeqCst), 1, "executor.stop called exactly once");
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1, "platform.cleanup_job called exactly once");

    let killed = find_process_killed(slug).expect("process.killed emitted");
    assert_eq!(killed.slug, slug);
    assert_eq!(killed.pid, Some(42));
    assert_eq!(killed.reason, "cancellation");
    assert_eq!(killed.actor, "operator");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_without_registered_handle_still_cleans_and_emits() {
    let _tmp = TempDir::new().unwrap();
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base.clone()).unwrap();
    router
        .spawn_actor("service.job-dispatch", dispatch_actor_config())
        .unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let stop_calls = Arc::new(AtomicUsize::new(0));
    let cleanup_calls = Arc::new(AtomicUsize::new(0));

    /// Registry with nothing in it — mirrors the "already dead / never
    /// registered" case. `take` returns None; handler must still emit
    /// `process.killed` (with pid=None) and call cleanup_job.
    struct EmptyRegistry;
    #[async_trait]
    impl ProcessRegistryPort for EmptyRegistry {
        async fn register(&self, _slug: &str, _h: ExecutorHandle) {}
        async fn unregister(&self, _slug: &str) {}
        async fn take(&self, _slug: &str) -> Option<ExecutorHandle> { None }
    }

    let octx = OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor: Arc::new(RecordingExecutor { stop_calls: stop_calls.clone() }),
        platform: Arc::new(RecordingPlatform { cleanup_calls: cleanup_calls.clone() }),
        registry: Arc::new(EmptyRegistry),
        probe: Arc::new(crate::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        job_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(std::collections::HashMap::new())),
    };

    let job_id = "j-cancel-2";
    let slug = "cancel-2";
    let payload = PipelineRunCancelledPayload {
        job_id: job_id.to_string(),
        slug: slug.to_string(),
        reason: "registry miss".into(),
        requested_by: "operator".into(),
    };
    let envelope = EventEnvelope::new(job_event_types::PIPELINE_RUN_CANCELLED, job_id, &payload)
        .unwrap()
        .with_actor_id("test");

    handle_job_event(&kernel, &envelope, &octx).await;

    assert_eq!(stop_calls.load(Ordering::SeqCst), 0, "no handle → no stop call");
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1, "cleanup_job still runs");
    let killed = find_process_killed(slug).expect("process.killed emitted");
    assert_eq!(killed.pid, None);
}

// ── Shared NoopIsolation ───────────────────────────────────────────────────

struct NoopIsolation;
#[async_trait]
impl crate::services::dispatch_ports::IsolationStrategy for NoopIsolation {
    async fn prepare(&self, _job: &SessionContext) -> anyhow::Result<PathBuf> {
        Ok(PathBuf::from("/tmp"))
    }
    async fn cleanup(&self, _job: &SessionContext) -> anyhow::Result<()> { Ok(()) }
}
