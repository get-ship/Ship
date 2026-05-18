//! Tests for fanout cancellation — `cancel_job_fanout_lanes` must
//! clean up every lane, drop their oneshot senders (so the aggregator
//! wakes), and clear the per-lane registry state.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, JobContext, JobExecutor,
    ProcessRegistryPort,
};
use crate::services::orchestrator::fanout_cancel::cancel_job_fanout_lanes;
use crate::services::orchestrator::{FanoutLaneRef, LaneOutcome, OrchestratorContext, StepActivity};

// ── Mocks ──────────────────────────────────────────────────────────────────

struct RecordingPlatform {
    cleanup_slugs: Arc<Mutex<Vec<(String, String)>>>,
}
#[async_trait]
impl AgentPlatform for RecordingPlatform {
    fn worktrees_dir(&self) -> PathBuf { PathBuf::from("/tmp/test") }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, slug: &str, branch: &str) {
        self.cleanup_slugs.lock().await.push((slug.to_string(), branch.to_string()));
    }
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> { Ok(true) }
    async fn cleanup_orphaned_worktrees(&self) {}
}

struct RecordingExecutor {
    stop_calls: Arc<AtomicUsize>,
}
#[async_trait]
impl JobExecutor for RecordingExecutor {
    async fn spawn(&self, _: &JobContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("unused"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> {
        self.stop_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
}

/// Registry that holds a handle for any slug with a live entry.
struct SeededRegistry {
    handles: Mutex<HashMap<String, ExecutorHandle>>,
}
#[async_trait]
impl ProcessRegistryPort for SeededRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
    async fn take(&self, slug: &str) -> Option<ExecutorHandle> {
        self.handles.lock().await.remove(slug)
    }
}

struct NoopIsolation;
#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, _: &JobContext) -> anyhow::Result<PathBuf> { Ok(PathBuf::from("/tmp")) }
    async fn cleanup(&self, _: &JobContext) -> anyhow::Result<()> { Ok(()) }
}

fn dispatch_actor_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".into(),
        write_namespaces: vec!["job.".into(), "step.".into(), "process.".into()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["pipeline.".into()],
        see_directed_events: false,
    }
}

fn make_octx(
    platform: Arc<RecordingPlatform>,
    executor: Arc<RecordingExecutor>,
    registry: Arc<SeededRegistry>,
) -> OrchestratorContext {
    OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor,
        platform,
        registry,
        probe: Arc::new(crate::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: Arc::new(Mutex::new(HashMap::<
            String, tokio::sync::mpsc::UnboundedSender<StepActivity>,
        >::new())),
        job_locks: Arc::new(Mutex::new(HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(HashMap::new())),
    }
}

async fn seed_lanes(
    octx: &OrchestratorContext,
    job_id: &str,
    parent_slug: &str,
    lanes: &[&str],
) -> Vec<tokio::sync::oneshot::Receiver<LaneOutcome>> {
    let mut rxs = Vec::new();
    let mut refs = octx.fanout_lane_refs.lock().await;
    let mut txs = octx.fanout_lane_txs.lock().await;
    for lane_id in lanes {
        let lane_slug = format!("{parent_slug}-draft-{lane_id}");
        let (tx, rx) = tokio::sync::oneshot::channel::<LaneOutcome>();
        txs.insert(lane_slug.clone(), tx);
        refs.insert(
            lane_slug.clone(),
            FanoutLaneRef {
                job_id: job_id.into(),
                parent_slug: parent_slug.into(),
                fanout_step_id: "draft".into(),
                fanout_step_index: 0,
                lane_id: (*lane_id).into(),
            },
        );
        rxs.push(rx);
    }
    rxs
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_walks_every_lane_and_wakes_the_aggregator() {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router.spawn_actor("service.job-dispatch", dispatch_actor_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let cleanup_slugs = Arc::new(Mutex::new(Vec::new()));
    let platform = Arc::new(RecordingPlatform { cleanup_slugs: cleanup_slugs.clone() });
    let stop_calls = Arc::new(AtomicUsize::new(0));
    let executor = Arc::new(RecordingExecutor { stop_calls: stop_calls.clone() });

    // Seed the registry with a live handle for two of the three lanes —
    // mirrors the real case where one lane finished spawn + registry
    // register while another never did.
    let mut handles = HashMap::new();
    handles.insert(
        "parent-draft-lane-a".to_string(),
        ExecutorHandle { pid: Some(101), inner: Box::new(()) },
    );
    handles.insert(
        "parent-draft-lane-b".to_string(),
        ExecutorHandle { pid: Some(102), inner: Box::new(()) },
    );
    let registry = Arc::new(SeededRegistry { handles: Mutex::new(handles) });

    let octx = make_octx(platform, executor, registry);

    let rxs = seed_lanes(&octx, "j-cancel", "parent", &["lane-a", "lane-b", "lane-c"]).await;

    cancel_job_fanout_lanes(&kernel, "j-cancel", &octx).await;

    // All three lanes: cleanup called with the branch naming convention.
    let cleanup = cleanup_slugs.lock().await.clone();
    assert_eq!(cleanup.len(), 3, "cleanup_job invoked once per lane");
    for lane in ["lane-a", "lane-b", "lane-c"] {
        let slug = format!("parent-draft-{lane}");
        let branch = format!("job/parent-draft-{lane}");
        assert!(
            cleanup.iter().any(|(s, b)| s == &slug && b == &branch),
            "cleanup must include ({slug}, {branch}); got {cleanup:?}"
        );
    }

    // Two lanes had handles, so executor.stop fired twice (not three).
    assert_eq!(stop_calls.load(Ordering::SeqCst), 2, "stop called for live lanes only");

    // Every aggregator receiver resolves promptly to Err (senders dropped).
    for rx in rxs {
        match tokio::time::timeout(std::time::Duration::from_millis(50), rx).await {
            Ok(Err(_)) => {}
            Ok(Ok(_)) => panic!("expected Err — sender should have been dropped"),
            Err(_) => panic!("aggregator receiver did not wake within 50ms"),
        }
    }

    // Registry state cleared.
    assert!(octx.fanout_lane_txs.lock().await.is_empty());
    assert!(octx.fanout_lane_refs.lock().await.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_is_idempotent_when_no_lanes_are_registered() {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router.spawn_actor("service.job-dispatch", dispatch_actor_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let cleanup_slugs = Arc::new(Mutex::new(Vec::new()));
    let platform = Arc::new(RecordingPlatform { cleanup_slugs: cleanup_slugs.clone() });
    let executor = Arc::new(RecordingExecutor { stop_calls: Arc::new(AtomicUsize::new(0)) });
    let registry = Arc::new(SeededRegistry { handles: Mutex::new(HashMap::new()) });

    let octx = make_octx(platform, executor, registry);

    cancel_job_fanout_lanes(&kernel, "j-nonexistent", &octx).await;

    assert!(cleanup_slugs.lock().await.is_empty(), "no lanes → no cleanup");
}
