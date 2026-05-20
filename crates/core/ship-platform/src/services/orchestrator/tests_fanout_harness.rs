//! Shared harness for fanout integration tests.
//!
//! Lives alongside `tests_fanout_integration.rs` so both the "all succeed",
//! "one fails", and "all fail" scenarios can reuse the same mock ports +
//! artifact/event seeding without blowing past the 300-line file cap.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::artifacts::{ArtifactScope, ArtifactService};
use ship_substrate::events::pipeline::{
    FanoutJudge, FanoutLane, FanoutSpec, OutputScope, Pipeline, PipelineStep,
    PipelineStepDoneDeclaredPayload, PipelineStepObservedPayload, StepOutput, StepTrigger, StepType, StepVerdict,
};
use ship_substrate::events::{EventEnvelope, EventSink};
use ship_substrate::persistence::event_store::SqliteEventStore;
use crate::services::dispatch_ports::{ExecutorHandle, ProcessRegistryPort};
use crate::services::orchestrator::{FanoutLaneRef, OrchestratorContext, StepActivity};

// ── Mock ports ───────────────────────────────────────────────────────────────

pub struct NoopRegistry;
#[async_trait]
impl ProcessRegistryPort for NoopRegistry {
    async fn register(&self, _slug: &str, _h: ExecutorHandle) {}
    async fn unregister(&self, _slug: &str) {}
}

pub struct NoopIsolation;
#[async_trait]
impl crate::services::dispatch_ports::IsolationStrategy for NoopIsolation {
    async fn prepare(
        &self,
        _job: &crate::services::dispatch_ports::SessionContext,
    ) -> anyhow::Result<std::path::PathBuf> {
        Ok(std::path::PathBuf::from("/tmp"))
    }
    async fn cleanup(
        &self,
        _job: &crate::services::dispatch_ports::SessionContext,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct NoopExecutor;
#[async_trait]
impl crate::services::dispatch_ports::SessionExecutor for NoopExecutor {
    async fn spawn(
        &self,
        _ctx: &crate::services::dispatch_ports::SessionContext,
    ) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("noop"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
}

pub struct NoopPlatform;
#[async_trait]
impl crate::services::dispatch_ports::AgentPlatform for NoopPlatform {
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

pub fn fresh_ship_dir() -> std::path::PathBuf {
    let dir = ship_substrate::project::get_global_dir().expect("test ship dir");
    for name in ["platform.db", "data.db"] {
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
        probe: Arc::new(crate::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: Arc::new(Mutex::new(HashMap::<
            String,
            tokio::sync::mpsc::UnboundedSender<StepActivity>,
        >::new())),
        run_locks: Arc::new(Mutex::new(HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(HashMap::new())),
    }
}

fn emit_event(
    ship_dir: &std::path::Path,
    event_type: &str,
    entity: &str,
    payload: serde_json::Value,
) {
    let store = SqliteEventStore::new_at(ship_dir).expect("event store");
    let env = EventEnvelope::new(event_type, entity, &payload).unwrap();
    store.persist(&env).expect("persist");
}

pub fn emit_lane_step_done(
    ship_dir: &std::path::Path,
    job_id: &str,
    lane_step_id: &str,
    outputs: std::collections::BTreeMap<String, String>,
) {
    let payload = PipelineStepDoneDeclaredPayload {
        job_id: job_id.into(),
        slug: "test-slug".into(),
        step_id: lane_step_id.into(),
        summary: Some("ok".into()),
        outputs,
    };
    emit_event(
        ship_dir, "pipeline.step.done_declared", job_id,
        serde_json::to_value(&payload).unwrap(),
    );
}

pub fn seed_job_created(ship_dir: &std::path::Path, job_id: &str, pipeline: &Pipeline) {
    emit_event(
        ship_dir, "pipeline.run.started", job_id,
        serde_json::json!({
            "job_id": job_id, "slug": "test-slug",
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

pub fn mk_lane_ref(job_id: &str, parent_slug: &str, lane_id: &str) -> FanoutLaneRef {
    FanoutLaneRef {
        job_id: job_id.into(), parent_slug: parent_slug.into(),
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
        job_id: "".into(),
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
    job_id: &str,
    run_scope: &ArtifactScope,
    lane_id: &str,
    body: &[u8],
) -> String {
    let art_svc = ArtifactService::new(ship_dir);
    let mut data = ship_substrate::data::DataService::open(ship_dir).unwrap();
    let rec = art_svc
        .store(
            &mut data, kernel,
            &format!("{lane_id}-draft.md"),
            "text/markdown", body, run_scope,
            None, None, Some("test"), None,
        )
        .await
        .unwrap();
    emit_lane_step_done(
        ship_dir, job_id,
        &format!("draft.{lane_id}"),
        [("draft".to_string(), rec.id.clone())].into(),
    );
    rec.id
}
