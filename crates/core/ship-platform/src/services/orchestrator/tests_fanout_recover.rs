//! Tests for restart recovery of orphaned fanout lanes. Seeds the event
//! store with lane dispatch events and no done_declared, then runs the
//! reconciler and asserts the recovery semantics.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_substrate::events::{EventEnvelope, EventSink};
use ship_substrate::persistence::event_store::SqliteEventStore;
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, SessionContext, SessionExecutor,
    ProcessRegistryPort,
};
use crate::services::orchestrator::fanout_recover::reconcile_fanout_lanes;
use crate::services::orchestrator::{OrchestratorContext, StepActivity};

// ── Mocks ──────────────────────────────────────────────────────────────────

struct CountingPlatform {
    cleanup_count: Arc<AtomicUsize>,
    cleanup_slugs: Arc<Mutex<Vec<String>>>,
}
#[async_trait]
impl AgentPlatform for CountingPlatform {
    fn worktrees_dir(&self) -> PathBuf { PathBuf::from("/tmp/test") }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, slug: &str, _branch: &str) {
        self.cleanup_count.fetch_add(1, Ordering::SeqCst);
        self.cleanup_slugs.lock().await.push(slug.to_string());
    }
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> { Ok(true) }
    async fn cleanup_orphaned_worktrees(&self) {}
}

struct NoopExecutor;
#[async_trait]
impl SessionExecutor for NoopExecutor {
    async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("unused"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
}

struct NoopRegistry;
#[async_trait]
impl ProcessRegistryPort for NoopRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
}

struct NoopIsolation;
#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, _: &SessionContext) -> anyhow::Result<PathBuf> { Ok(PathBuf::from("/tmp")) }
    async fn cleanup(&self, _: &SessionContext) -> anyhow::Result<()> { Ok(()) }
}

fn make_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router
        .spawn_actor(
            "service.orchestrator",
            ActorConfig {
                namespace: "service.orchestrator".into(),
                write_namespaces: vec!["job.".into(), "step.".into(), "artifact.".into()],
                read_namespaces: vec![],
                subscribe_namespaces: vec!["pipeline.".into()],
                see_directed_events: false,
            },
        )
        .unwrap();
    Arc::new(Mutex::new(router))
}

fn make_octx(platform: Arc<CountingPlatform>) -> OrchestratorContext {
    OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor: Arc::new(NoopExecutor),
        platform,
        registry: Arc::new(NoopRegistry),
        probe: Arc::new(crate::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        spawn_policy: Arc::new(crate::services::orchestrator::spawn_policy::NoopSpawnPolicy),
        projections: Arc::new(crate::services::orchestrator::projection_reader::ProjectionsReader),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: Arc::new(Mutex::new(HashMap::<
            String, tokio::sync::mpsc::UnboundedSender<StepActivity>,
        >::new())),
        run_locks: Arc::new(Mutex::new(HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(HashMap::new())),
    }
}

fn fresh_ship_dir() -> PathBuf {
    let dir = ship_substrate::project::get_global_dir().expect("test ship dir");
    for name in ["platform.db", "data.db"] {
        let p = dir.join(name);
        if p.exists() { let _ = std::fs::remove_file(&p); }
    }
    dir
}

fn persist(ship_dir: &std::path::Path, event_type: &str, entity: &str, payload: serde_json::Value) {
    // pipeline.*/step.* are Runtime-scoped → seed runtime.db (the orchestrator
    // projection loaders read it).
    let store = SqliteEventStore::new_at_db_path(
        &ship_substrate::db::runtime_db_path(ship_dir),
    )
    .expect("event store");
    let env = EventEnvelope::new(event_type, entity, &payload).unwrap();
    store.persist(&env).expect("persist");
}

fn seed_live_parent_with_three_orphan_lanes(ship_dir: &std::path::Path, pipeline_run_id: &str, slug: &str) {
    use ship_substrate::events::pipeline::event_types;
    // Parent dispatched, still live (no job.completed/failed/cancelled).
    persist(ship_dir, event_types::PIPELINE_RUN_STARTED, pipeline_run_id, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug, "agent": "orchestrator",
        "branch": format!("job/{slug}"),
        "spec_path": ".artifacts/job-spec.md", "source_spec_path": null,
    }));
    persist(ship_dir, event_types::PIPELINE_RUN_DISPATCHED, pipeline_run_id, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "worktree": format!("/tmp/{slug}"), "pid": null,
    }));
    for lane in ["a", "b", "c"] {
        persist(ship_dir, event_types::PIPELINE_STEP_STARTED, pipeline_run_id, serde_json::json!({
            "pipeline_run_id": pipeline_run_id, "slug": slug,
            "step_id": format!("draft.{lane}"),
            "step_type": "fanout_lane",
            "fanout_parent_step": "draft",
            "lane_id": lane,
            "lane_slug": format!("{slug}-draft-{lane}"),
        }));
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconcile_live_parent_emits_synthetic_failed_markers_for_orphan_lanes() {
    let ship_dir = fresh_ship_dir();
    let kernel = make_kernel();

    let cleanup_count = Arc::new(AtomicUsize::new(0));
    let cleanup_slugs = Arc::new(Mutex::new(Vec::new()));
    let platform = Arc::new(CountingPlatform {
        cleanup_count: cleanup_count.clone(),
        cleanup_slugs: cleanup_slugs.clone(),
    });
    let octx = make_octx(platform);

    seed_live_parent_with_three_orphan_lanes(&ship_dir, "j-restart-1", "slug-1");

    reconcile_fanout_lanes(&kernel, &octx).await;

    // Parent live → no worktree cleanup. Three `step.done_declared` events
    // landed carrying the `failed` marker output.
    assert_eq!(cleanup_count.load(Ordering::SeqCst), 0, "live parent must not cleanup");

    let events = ship_substrate::db::events::list_all_events().unwrap();
    let synthetic_dones: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == "pipeline.step.done_declared")
        .filter(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["step_id"].as_str().map(|s| s.starts_with("draft.")).unwrap_or(false)
        })
        .collect();
    assert_eq!(synthetic_dones.len(), 3, "one synthetic done per orphan lane");
    for e in &synthetic_dones {
        let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap();
        assert!(p["outputs"]["failed"].is_string(),
            "synthetic done must carry the `failed` marker artifact id");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconcile_terminal_parent_cleans_up_lane_worktrees() {
    use ship_substrate::events::pipeline::event_types;
    let ship_dir = fresh_ship_dir();
    let kernel = make_kernel();

    let cleanup_count = Arc::new(AtomicUsize::new(0));
    let cleanup_slugs = Arc::new(Mutex::new(Vec::new()));
    let platform = Arc::new(CountingPlatform {
        cleanup_count: cleanup_count.clone(),
        cleanup_slugs: cleanup_slugs.clone(),
    });
    let octx = make_octx(platform);

    seed_live_parent_with_three_orphan_lanes(&ship_dir, "j-restart-2", "slug-2");
    // Promote parent into a terminal state: emit job.cancelled.
    persist(&ship_dir, event_types::PIPELINE_RUN_CANCELLED, "j-restart-2", serde_json::json!({
        "pipeline_run_id": "j-restart-2", "slug": "slug-2",
        "reason": "operator", "requested_by": "test",
    }));

    reconcile_fanout_lanes(&kernel, &octx).await;

    assert_eq!(
        cleanup_count.load(Ordering::SeqCst), 3,
        "one cleanup per orphan lane when parent is terminal"
    );
    let slugs = cleanup_slugs.lock().await.clone();
    for lane in ["a", "b", "c"] {
        let expected = format!("slug-2-draft-{lane}");
        assert!(slugs.contains(&expected), "cleanup must include {expected}; got {slugs:?}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconcile_is_idempotent_second_pass_emits_no_extra_markers() {
    let ship_dir = fresh_ship_dir();
    let kernel = make_kernel();

    let platform = Arc::new(CountingPlatform {
        cleanup_count: Arc::new(AtomicUsize::new(0)),
        cleanup_slugs: Arc::new(Mutex::new(Vec::new())),
    });
    let octx = make_octx(platform);

    seed_live_parent_with_three_orphan_lanes(&ship_dir, "j-restart-3", "slug-3");
    reconcile_fanout_lanes(&kernel, &octx).await;

    let first_dones: usize = ship_substrate::db::events::list_all_events().unwrap()
        .iter()
        .filter(|e| e.event_type == "pipeline.step.done_declared")
        .count();

    reconcile_fanout_lanes(&kernel, &octx).await;

    let second_dones: usize = ship_substrate::db::events::list_all_events().unwrap()
        .iter()
        .filter(|e| e.event_type == "pipeline.step.done_declared")
        .count();
    assert_eq!(
        first_dones, second_dones,
        "second reconcile pass must not emit additional synthetic dones"
    );
}
