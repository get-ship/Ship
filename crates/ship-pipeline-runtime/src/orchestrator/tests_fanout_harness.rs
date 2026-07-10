//! Shared harness for fanout integration tests.
//!
//! Lives alongside `tests_fanout_integration.rs` so both the "all succeed",
//! "one fails", and "all fail" scenarios can reuse the same mock ports +
//! artifact/event seeding without blowing past the 300-line file cap.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_artifacts::service::ArtifactService;
use ship_artifacts::types::ArtifactScope;
use ship_substrate::events::pipeline::{
    FanoutJudge, FanoutLane, FanoutSpec, OutputScope, Pipeline, PipelineStep,
    PipelineStepDoneDeclaredPayload, PipelineStepObservedPayload, StepOutput, StepTrigger, StepType, StepVerdict,
};
use ship_substrate::events::{EventEnvelope, EventSink};
use ship_substrate::persistence::event_store::SqliteEventStore;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};
use ship_code::dispatch_ports::ExecutorHandle;
use crate::orchestrator::{FanoutLaneRef, OrchestratorContext, StepActivity};

// ── Mock ports ───────────────────────────────────────────────────────────────

pub use super::tests_fakes::NoopRegistry;
pub use super::tests_fakes::NoopIsolation;

pub struct NoopExecutor;
#[async_trait]
impl ship_code::dispatch_ports::SessionExecutor for NoopExecutor {
    async fn spawn(
        &self,
        _ctx: &ship_code::dispatch_ports::SessionContext,
    ) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("noop"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
}

pub struct NoopPlatform;
#[async_trait]
impl ship_code::dispatch_ports::AgentPlatform for NoopPlatform {
    fn worktrees_dir(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("/tmp/worktrees")
    }
    fn compile_agent_config(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> { Ok(()) }
    async fn graceful_agent_exit(&self, _: &str, _: Option<&str>) {}
    async fn cleanup_job(&self, _: &str, _: &str) {}
    fn git_merge(&self, _: &str) -> anyhow::Result<bool> { Ok(true) }
    async fn cleanup_orphaned_worktrees(&self) {}
}

// ── Harness helpers ──────────────────────────────────────────────────────────

/// The run's space CONTENT-ID — what the host resolves via the `projects`
/// pointer and hands the orchestrator (`OrchestratorContext::space_id`).
pub const TEST_SPACE: &str = "space-test-content-id";

pub fn fresh_ship_dir() -> std::path::PathBuf {
    // The artifact/collection/auth projections moved off the L0 default bus
    // to the platform factory (keystone-A): install the SAME production set
    // before any SqliteEventStore is built, or lane artifacts never
    // materialize rows. Idempotent.
    ship_platform::install_defaults();
    let dir = ship_substrate::project::get_global_dir().expect("test ship dir");
    for name in ["data.db"] {
        let p = dir.join(name);
        if p.exists() { let _ = std::fs::remove_file(&p); }
    }
    dir
}

pub fn make_octx() -> OrchestratorContext {
    OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor: Arc::new(NoopExecutor),
        platform: Arc::new(NoopPlatform),
        registry: Arc::new(NoopRegistry),
        probe: Arc::new(ship_code::process_tree_probe::SysinfoProcessTreeProbe::new()),
        spawn_policy: Arc::new(crate::orchestrator::spawn_policy::NoopSpawnPolicy),
        projections: Arc::new(crate::orchestrator::projection_reader::ProjectionsReader),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: Arc::new(Mutex::new(HashMap::<
            String,
            tokio::sync::mpsc::UnboundedSender<StepActivity>,
        >::new())),
        run_locks: Arc::new(Mutex::new(HashMap::new())),
        event_dedup: Arc::new(ship_substrate::dispatch_gate::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(HashMap::new())),
        journal_store: Arc::new(durable_execution::InMemoryJournalStore::new()),
        space_id: Some(TEST_SPACE.to_string()),
    }
}

fn emit_event(
    ship_dir: &std::path::Path,
    event_type: &str,
    entity: &str,
    payload: serde_json::Value,
) {
    // pipeline.*/step.* are Runtime-scoped → seed runtime.db, the db the
    // projection loaders the orchestrator reads from resolve.
    let store = SqliteEventStore::new_at_db_path(
        &ship_substrate::db::runtime_db_path(ship_dir),
    )
    .expect("event store");
    let env = EventEnvelope::new(event_type, entity, &payload).unwrap();
    store.persist(&env).expect("persist");
}

pub fn emit_lane_step_done(
    ship_dir: &std::path::Path,
    pipeline_run_id: &str,
    lane_step_id: &str,
    outputs: std::collections::BTreeMap<String, String>,
) {
    let payload = PipelineStepDoneDeclaredPayload {
        pipeline_run_id: pipeline_run_id.into(),
        slug: "test-slug".into(),
        step_id: lane_step_id.into(),
        summary: Some("ok".into()),
        outputs,
    };
    emit_event(
        ship_dir, "pipeline.step.done_declared", pipeline_run_id,
        serde_json::to_value(&payload).unwrap(),
    );
}

pub fn seed_pipeline_run_started(ship_dir: &std::path::Path, pipeline_run_id: &str, pipeline: &Pipeline) {
    emit_event(
        ship_dir, "pipeline.run.started", pipeline_run_id,
        serde_json::json!({
            "pipeline_run_id": pipeline_run_id, "slug": "test-slug",
            "agent": "orchestrator", "branch": "test",
            "spec_path": ".artifacts/job-spec.md", "steps": pipeline,
        }),
    );
}

pub fn mk_step(id: &str, agent: &str) -> PipelineStep {
    PipelineStep {
        id: id.into(), step_type: StepType::Agent,
        agent: Some(agent.into()), goal: None, after: vec![],
        model: None, harness: None, command: None,
        max_retries: None, gate: None,
        produces: vec![], consumes: vec![], promotes: vec![], fanout: None,
            invocation: None,
    }
}

pub fn mk_lane(id: &str) -> FanoutLane {
    FanoutLane { id: id.into(), agent: "implementer".into(), model: None, harness: None }
}

pub fn mk_pipeline_with_three_lanes() -> Pipeline {
    let mut fanout_step = mk_step("draft", "implementer");
    fanout_step.fanout = Some(FanoutSpec {
        lanes: vec![mk_lane("lane-a"), mk_lane("lane-b"), mk_lane("lane-c")],
        judge: FanoutJudge { step: "choose".into() },
        min_successful_lanes: 1,
    });
    fanout_step.produces = vec![StepOutput {
        name: "draft".into(), scope: OutputScope::Run, required: false,
    }];

    let mut judge = mk_step("choose", "judge");
    judge.after = vec!["draft".into()];
    judge.consumes = vec![
        "lane-a.draft".into(),
        "lane-b.draft".into(),
        "lane-c.draft".into(),
    ];

    Pipeline { steps: vec![fanout_step, judge], auto_merge: false }
}

pub fn mk_lane_ref(pipeline_run_id: &str, parent_slug: &str, lane_id: &str) -> FanoutLaneRef {
    FanoutLaneRef {
        pipeline_run_id: pipeline_run_id.into(), parent_slug: parent_slug.into(),
        fanout_step_id: "draft".into(),
        fanout_step_index: 0,
        lane_id: lane_id.into(),
    }
}

pub fn mk_observed(
    slug: &str,
    step_name: &str,
    verdict: StepVerdict,
    reason: Option<&str>,
) -> PipelineStepObservedPayload {
    PipelineStepObservedPayload {
        pipeline_run_id: "".into(),
        slug: slug.into(),
        step_index: 0,
        step_name: step_name.into(),
        trigger: StepTrigger::DoneDeclared,
        done_declared: true,
        done_result: Some(matches!(verdict, StepVerdict::Advance)),
        exit_code: None,
        commits_since_start: 0,
        uncommitted_files: 0,
        duration_secs: 0,
        reason: reason.map(str::to_string),
        succeeded: matches!(verdict, StepVerdict::Advance),
        verdict,
        start_sha: None,
        log_tail: None,
    }
}

/// Seed a lane's successful output: store the artifact and emit the lane's
/// `step.done_declared`. Returns the artifact id.
pub async fn seed_lane_success(
    ship_dir: &std::path::Path,
    kernel: &Arc<Mutex<ship_substrate::events::kernel_router::KernelRouter>>,
    pipeline_run_id: &str,
    run_scope: &ArtifactScope,
    lane_id: &str,
    body: &[u8],
) -> String {
    // The fixture's `run_scope` always carries a space_id (the project slug);
    // require it rather than fabricate a `global` partition (S2 home-space kill).
    let space_id = run_scope
        .space_id()
        .expect("fanout test fixture scope must carry a space_id");
    // Artifact metadata is event-sourced: bind to the per-scope space.db where
    // `ArtifactProjection` materializes the row (consumer consistency).
    let space_db = ship_substrate::db::space_db_path(ship_dir, space_id);
    let data_svc = ship_substrate::data::DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(data_svc, space_id));
    let kernel_log = Arc::new(KernelLog::new(kernel.clone()));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(space_id), store, kernel_log));
    let art_svc = ArtifactService::new(ctx, ship_dir.to_path_buf());
    let rec = art_svc
        .store(
            &format!("{lane_id}-draft.md"),
            "text/markdown", body, run_scope,
            None, None, Some("test"), None,
        )
        .await
        .unwrap();
    emit_lane_step_done(
        ship_dir, pipeline_run_id,
        &format!("draft.{lane_id}"),
        [("draft".to_string(), rec.id.clone())].into(),
    );
    rec.id
}
