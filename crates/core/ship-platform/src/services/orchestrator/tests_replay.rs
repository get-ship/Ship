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

use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, JobContext, JobExecutor,
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
        probe: Arc::new(crate::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        job_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(std::collections::HashMap::new())),
    })
}

fn make_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
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
    use ship_substrate::events::job::{event_types, JobCreatedPayload, JobDispatchedPayload};
    use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope};
    let ctx = EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, project_id: None, session_id: None };
    let created = JobCreatedPayload {
        job_id: job_id.to_string(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: None,
        plan_id: None, depends_on: None, model: None, harness: None,
        pipeline: None, steps: None, pipeline_name: None, vars: None,
        spec_content: None, step_specs: None, headless: false,
        baseline_sha: None,
    };
    let env = EventEnvelope::new(event_types::JOB_CREATED, job_id, &created).unwrap().with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
    let dispatched = JobDispatchedPayload {
        job_id: job_id.to_string(),
        worktree: format!("/tmp/test-worktrees-replay/{slug}"),
        pid: None,
        harness_meta: None,
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

    let all = ship_substrate::db::events::list_all_events().unwrap_or_default();

    let destroyed: Vec<_> = all
        .iter()
        .filter(|e| e.event_type == ship_substrate::events::executor::EXECUTOR_SESSION_DESTROYED && e.entity_id == slug)
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

// ─────────────────────────────────────────────────────────────────────────
// Replay filter + cap — startup-replay-bound-01
//
// `replay_unprocessed_completions` must skip Completed jobs whose pipeline
// is fully advanced with no outstanding auto_merge, and must not replay
// more than MAX_REPLAYS_PER_STARTUP jobs per daemon startup. These tests
// pin both the `needs_replay` predicate and `select_replay_targets` ranking.
// ─────────────────────────────────────────────────────────────────────────

mod replay_filter {
    use std::collections::HashMap;

    use chrono::{Duration, Utc};

    use ship_substrate::events::job::{Pipeline, PipelineStep, StepType};
    use crate::projections::job::{JobRecord, JobStatus};
    use crate::services::orchestrator::replay::{
        MAX_REPLAYS_PER_STARTUP, needs_replay, select_replay_targets,
    };

    fn pipeline_step(id: &str) -> PipelineStep {
        PipelineStep {
            id: id.to_string(),
            step_type: StepType::Agent,
            agent: Some("agent".to_string()),
            goal: None,
            after: vec![],
            model: None,
            harness: None,
            command: None,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
        }
    }

    fn make_job(
        id: &str,
        step_ids: &[&str],
        completed: &[&str],
        auto_merge: bool,
        worktree: Option<String>,
    ) -> JobRecord {
        let now = Utc::now();
        JobRecord {
            job_id: id.to_string(),
            slug: id.to_string(),
            agent: "test-agent".to_string(),
            branch: format!("job/{id}"),
            spec_path: String::new(),
            source_spec_path: None,
            depends_on: None,
            status: JobStatus::Completed,
            worktree,
            blocker: None,
            error: None,
            model: None,
            harness: None,
            created_at: now,
            updated_at: now,
            harness_meta: None,
            pipeline: None,
            steps: Some(Pipeline {
                steps: step_ids.iter().map(|s| pipeline_step(s)).collect(),
                auto_merge,
            }),
            current_phase: None,
            completed_steps: completed.iter().map(|s| s.to_string()).collect(),
            current_step: None,
            vars: None,
            step_retries: HashMap::new(),
            headless: false,
            baseline_sha: None,
        }
    }

    #[test]
    fn replay_skips_fully_advanced_jobs() {
        // All steps completed, no auto_merge — replay would be a no-op.
        let job = make_job("j", &["a", "b", "c"], &["a", "b", "c"], false, None);
        assert!(!needs_replay(&job));
    }

    #[test]
    fn replay_includes_jobs_with_unfinished_pipeline() {
        // Completed status set early, pipeline still has work.
        let job = make_job("j", &["a", "b", "c"], &["a"], false, None);
        assert!(needs_replay(&job));
    }

    #[test]
    fn replay_includes_auto_merge_jobs_with_extant_worktree() {
        // All steps done, auto_merge pending because worktree still on disk.
        let tmp = tempfile::TempDir::new().unwrap();
        let path = tmp.path().to_string_lossy().into_owned();
        let job = make_job("j", &["a"], &["a"], true, Some(path));
        assert!(needs_replay(&job));
    }

    #[test]
    fn replay_skips_auto_merge_jobs_when_worktree_gone() {
        // auto_merge already fired — worktree removed. Nothing to replay.
        let missing = "/tmp/ship-startup-replay-test-missing-xyz123".to_string();
        assert!(!std::path::Path::new(&missing).exists());
        let job = make_job("j", &["a"], &["a"], true, Some(missing));
        assert!(!needs_replay(&job));
    }

    #[test]
    fn replay_skips_jobs_without_dag_pipeline() {
        // Legacy / phase-only jobs have nothing for advance_pipeline to do.
        let mut job = make_job("j", &["a"], &["a"], false, None);
        job.steps = None;
        assert!(!needs_replay(&job));
    }

    #[test]
    fn replay_caps_at_ten_oldest_dropped() {
        let base = Utc::now();
        let mut jobs = HashMap::new();
        for i in 0..15 {
            let mut job = make_job(
                &format!("j{i:02}"),
                &["a", "b"],
                &["a"],
                false,
                None,
            );
            // Higher i → more recent. Oldest five must be dropped.
            job.updated_at = base + Duration::seconds(i64::from(i));
            jobs.insert(job.job_id.clone(), job);
        }

        let selected = select_replay_targets(&jobs);

        assert_eq!(selected.len(), MAX_REPLAYS_PER_STARTUP);
        let ids: Vec<_> = selected.iter().map(|j| j.job_id.clone()).collect();
        for i in 5..15 {
            assert!(ids.contains(&format!("j{i:02}")), "expected recent j{i:02} in replay set");
        }
        for i in 0..5 {
            assert!(!ids.contains(&format!("j{i:02}")), "oldest j{i:02} should be dropped");
        }
    }

    #[test]
    fn replay_no_change_when_nothing_to_replay() {
        let empty: HashMap<String, JobRecord> = HashMap::new();
        assert!(select_replay_targets(&empty).is_empty());

        // Fully-advanced no-merge job is filtered out — select returns empty.
        let mut jobs = HashMap::new();
        let terminal = make_job("j", &["a", "b"], &["a", "b"], false, None);
        jobs.insert(terminal.job_id.clone(), terminal);
        assert!(select_replay_targets(&jobs).is_empty());
    }
}
