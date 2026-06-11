//! Tests for the coherence sweep — verifies drift detection and
//! resolution for dispatched-but-dead jobs and stray processes.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::sweep_events::event_types as sup_events;
use ship_substrate::events::EventEnvelope;
use ship_code::dispatch_ports::{
    AgentPlatform, ExecutorHandle, IsolationStrategy, SessionContext, SessionExecutor,
    ProcessRegistryPort,
};
use crate::orchestrator::sweep::run_sweep;
use crate::orchestrator::OrchestratorContext;

// ── Mocks ──────────────────────────────────────────────────────────────────

struct NoopIsolation;
#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, _job: &SessionContext) -> anyhow::Result<PathBuf> {
        Ok(PathBuf::from("/tmp"))
    }
    async fn cleanup(&self, _job: &SessionContext) -> anyhow::Result<()> {
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
    async fn cleanup_orphaned_worktrees(&self) {}
}

/// Executor that always returns None from resume_handle (process missing).
struct DeadExecutor;
#[async_trait]
impl SessionExecutor for DeadExecutor {
    async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
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
impl SessionExecutor for AliveExecutor {
    async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
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
impl SessionExecutor for RecordingStopExecutor {
    async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
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
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    let config = ActorConfig {
        namespace: "service.orchestrator".to_string(),
        write_namespaces: vec![
            "job.".to_string(),
            "step.".to_string(),
            "process.".to_string(),
            "supervisor.".to_string(),
        ],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["pipeline.".to_string()],
        see_directed_events: false,
    };
    router.spawn_actor("service.orchestrator", config).unwrap();
    Arc::new(Mutex::new(router))
}

fn make_octx(
    executor: Arc<dyn SessionExecutor>,
    registry: Arc<dyn ProcessRegistryPort>,
) -> Arc<OrchestratorContext> {
    Arc::new(OrchestratorContext {
        isolation: Arc::new(NoopIsolation),
        executor,
        platform: Arc::new(NoopPlatform),
        registry,
        probe: Arc::new(ship_code::process_tree_probe::SysinfoProcessTreeProbe::new()),
        // Sweep tests assert real orphan/known-dispatch behavior against the
        // seeded test DB, so inject the real reader (same projections the
        // old inline is_known_dispatch read).
        spawn_policy: Arc::new(crate::orchestrator::spawn_policy::NoopSpawnPolicy),
        projections: Arc::new(crate::orchestrator::projection_reader::ProjectionsReader),
        done_flags: Arc::new(Mutex::new(std::collections::HashMap::new())),
        activity_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        run_locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        event_dedup: Arc::new(crate::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        journal_store: Arc::new(durable_execution::InMemoryJournalStore::new()),
    })
}

/// Seed a job in Dispatched status by emitting job.created + job.dispatched.
async fn seed_dispatched_job(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
) {
    use ship_substrate::events::pipeline::{event_types, PipelineRunStartedPayload, PipelineRunDispatchedPayload};
    use ship_substrate::events::{CallerKind, EmitContext};

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };

    let created = PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: None,
        plan_id: None,
        depends_on: None,
        model: None,
        harness: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: None,
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, pipeline_run_id, &created)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();

    let dispatched = PipelineRunDispatchedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        worktree: format!("/tmp/test-worktrees/{slug}"),
        pid: None,
        harness_meta: None,
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_DISPATCHED, pipeline_run_id, &dispatched)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

/// Seed a dispatched job with a DAG pipeline; `current_step` advances via
/// the PIPELINE_RUN_DISPATCHED projection.
async fn seed_dispatched_job_with_pipeline(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
    pipeline: ship_substrate::events::pipeline::Pipeline,
) {
    use ship_substrate::events::pipeline::{event_types, PipelineRunStartedPayload, PipelineRunDispatchedPayload};
    use ship_substrate::events::{CallerKind, EmitContext};

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };

    let created = PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: None,
        plan_id: None,
        depends_on: None,
        model: None,
        harness: None,
        pipeline: None,
        steps: Some(pipeline),
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: None,
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, pipeline_run_id, &created)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();

    let dispatched = PipelineRunDispatchedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        worktree: format!("/tmp/test-worktrees/{slug}"),
        pid: None,
        harness_meta: None,
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_DISPATCHED, pipeline_run_id, &dispatched)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

fn one_step_pipeline(step_id: &str, step_type: ship_substrate::events::pipeline::StepType)
    -> ship_substrate::events::pipeline::Pipeline
{
    use ship_substrate::events::pipeline::{Pipeline, PipelineStep, StepType};
    let (command, agent, goal) = match step_type {
        StepType::Run => (Some("echo hi".to_string()), None, None),
        _ => (None, Some("test-agent".to_string()), Some("do work".to_string())),
    };
    Pipeline {
        auto_merge: false,
        steps: vec![PipelineStep {
            id: step_id.to_string(),
            step_type,
            agent,
            goal,
            after: vec![],
            model: None,
            harness: None,
            command,
            max_retries: None,
            gate: None,
            produces: vec![],
            consumes: vec![],
            promotes: vec![],
            fanout: None,
            invocation: None,
        }],
    }
}

/// Seed a job in Failed status.
async fn seed_failed_job(
    kernel: &Arc<Mutex<KernelRouter>>,
    pipeline_run_id: &str,
    slug: &str,
) {
    use ship_substrate::events::pipeline::{event_types, PipelineRunStartedPayload, PipelineRunFailedPayload};
    use ship_substrate::events::{CallerKind, EmitContext};

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };

    let created = PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        slug: slug.to_string(),
        agent: "test-agent".to_string(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".to_string(),
        source_spec_path: None,
        plan_id: None,
        depends_on: None,
        model: None,
        harness: None,
        pipeline: None,
        steps: None,
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: false,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: None,
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, pipeline_run_id, &created)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();

    let failed = PipelineRunFailedPayload {
        pipeline_run_id: pipeline_run_id.to_string(),
        error: "seed failure".to_string(),
    };
    let env = EventEnvelope::new(event_types::PIPELINE_RUN_FAILED, pipeline_run_id, &failed)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

fn find_events_by_type(event_type: &str) -> Vec<EventEnvelope> {
    ship_substrate::db::events::list_all_events()
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
    let failed = find_events_by_type("pipeline.run.failed");
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
    use ship_substrate::events::process::event_types as proc_events;
    let killed = find_events_by_type(proc_events::PROCESS_KILLED);
    assert!(
        killed.iter().any(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["slug"] == "orphan-1" && p["actor"] == "supervisor"
        }),
        "process.killed emitted with actor=supervisor for stray kill"
    );
}

// ── Run-step awareness (sweep-run-aware-01) ────────────────────────────────
// Run steps are synchronous and don't register handles; the sweep's live-
// process check would false-flag them on any command outlasting the 30s
// interval. These tests pin that the skip is Run-only — Agent and
// no-current-step paths still reach the existing drift branch.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_skips_run_step_jobs() {
    use ship_substrate::events::pipeline::StepType;
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(Arc::new(DeadExecutor), Arc::new(EmptyRegistry));

    seed_dispatched_job_with_pipeline(
        &kernel,
        "j-run-aware-1",
        "sweep-run-aware-1",
        one_step_pipeline("build", StepType::Run),
    )
    .await;
    run_sweep(&kernel, &octx).await.unwrap();

    // No drift event scoped to this job's slug.
    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        !drift.iter().any(|e| e.entity_id == "sweep-run-aware-1"),
        "Run-step dispatched job must not be flagged as drift"
    );

    // No job.failed for this pipeline_run_id emitted by the sweep.
    let sweep_failed = find_events_by_type("pipeline.run.failed")
        .into_iter()
        .find(|e| e.entity_id == "j-run-aware-1");
    assert!(
        sweep_failed.is_none(),
        "Run-step dispatched job must not be marked failed by the sweep"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_still_flags_agent_step_jobs_with_no_process() {
    use ship_substrate::events::pipeline::StepType;
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(Arc::new(DeadExecutor), Arc::new(EmptyRegistry));

    seed_dispatched_job_with_pipeline(
        &kernel,
        "j-agent-aware-1",
        "sweep-agent-aware-1",
        one_step_pipeline("work", StepType::Agent),
    )
    .await;
    run_sweep(&kernel, &octx).await.unwrap();

    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        drift.iter().any(|e| e.entity_id == "sweep-agent-aware-1"),
        "agent-step dispatched job with no live process must still be flagged"
    );

    let sweep_failed = find_events_by_type("pipeline.run.failed")
        .into_iter()
        .filter(|e| e.entity_id == "j-agent-aware-1")
        .find(|e| {
            let p: serde_json::Value =
                serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["error"].as_str().unwrap_or_default().contains("coherence sweep")
        });
    assert!(
        sweep_failed.is_some(),
        "agent-step dispatched job must receive coherence-sweep job.failed"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_skips_jobs_with_no_current_step() {
    // no pipeline → current_step=None. Pins that the Run filter doesn't
    // short-circuit the no-current-step path — only Run-typed steps skip.
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(Arc::new(DeadExecutor), Arc::new(EmptyRegistry));

    seed_dispatched_job(&kernel, "j-no-current-1", "sweep-no-current-1").await;
    run_sweep(&kernel, &octx).await.unwrap();

    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        drift.iter().any(|e| e.entity_id == "sweep-no-current-1"),
        "dispatched job with no pipeline must still be flagged — safety net intact"
    );
}

// ── Phase 6: is_known_dispatch consults canonical_sessions ──────────
//
// Check B's old rule ("kill registered processes whose job isn't
// currently Dispatched|GatePending") false-flagged every bare agent
// session — they don't have job rows at all. These two tests pin the
// new rule: a process owned by ANY known dispatch record (jobs OR
// canonical_sessions) is not stray-killed; true orphans (no record
// anywhere) still are.

/// Registry that reports a registered slug as live. Used to drive
/// Check B (the orphan check) — we need a live_slug to exist so the
/// classifier runs at all.
struct RegistryWithLiveSlug(&'static str);
#[async_trait]
impl ProcessRegistryPort for RegistryWithLiveSlug {
    async fn register(&self, _slug: &str, _handle: ExecutorHandle) {}
    async fn unregister(&self, _slug: &str) {}
    async fn live_slugs(&self) -> Vec<String> { vec![self.0.to_string()] }
    async fn take(&self, _slug: &str) -> Option<ExecutorHandle> { None }
    async fn drain_dead(&self, _executor: &dyn SessionExecutor) -> Vec<String> { vec![] }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_does_not_kill_process_with_bare_agent_session_record() {
    // Seed a canonical_sessions row (no pipeline-run/jobs row) and a live
    // slug matching it. The OLD Check B would treat this as stray and
    // emit process.killed + supervisor.coherence_restored. The new
    // rule recognizes canonical_sessions as a valid dispatch record and
    // leaves the process alone.
    //
    // install_defaults() registers CanonicalSessionProjection so the
    // seeded session.started event folds into canonical_sessions and
    // is_known_dispatch returns true.
    ship_platform::install_defaults();
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(
        Arc::new(DeadExecutor),
        Arc::new(RegistryWithLiveSlug("sweep-bare-session-1")),
    );

    seed_canonical_session(
        &kernel,
        "01HV-SESSION-BARE-1",
        "sweep-bare-session-1",
    )
    .await;

    run_sweep(&kernel, &octx).await.unwrap();

    let killed: Vec<_> =
        find_events_by_type("process.killed")
            .into_iter()
            .filter(|e| e.entity_id == "sweep-bare-session-1")
            .collect();
    assert!(
        killed.is_empty(),
        "sweep must not kill a process whose canonical_sessions record exists; \
         got killed events: {killed:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_still_kills_true_orphan_process_with_no_dispatch_record() {
    // No jobs row, no canonical_sessions row — a live slug with nothing
    // backing it is the orphan-cleanup case the sweep was designed
    // for. Check B must still fire here so daemon-restart leftovers
    // and rogue spawns get reaped.
    ship_platform::install_defaults();
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let octx = make_octx(
        Arc::new(DeadExecutor),
        Arc::new(RegistryWithLiveSlug("sweep-orphan-1")),
    );

    run_sweep(&kernel, &octx).await.unwrap();

    let killed: Vec<_> =
        find_events_by_type("process.killed")
            .into_iter()
            .filter(|e| e.entity_id == "sweep-orphan-1")
            .collect();
    assert!(
        !killed.is_empty(),
        "sweep must still reap true orphans (no jobs row, no canonical_sessions row)"
    );
}

/// Seed a canonical_sessions row by emitting session.started through the kernel.
/// install_defaults() must have been called so CanonicalSessionProjection is
/// registered and folds the event into canonical_sessions.
async fn seed_canonical_session(
    kernel: &Arc<Mutex<KernelRouter>>,
    session_id: &str,
    slug: &str,
) {
    use agent_def::canonical_session::{SessionStarted, event_types as canonical_event_types};
    use ship_substrate::events::{CallerKind, EmitContext};

    // session.started is Space-scoped: bind the space EXPLICITLY so the test
    // is hermetic. (It used to rely on daemon_internal(None) + the global
    // ~/.ship space-id resolver finding a registered project — which only
    // held on polluted dev machines and made this test env-dependent.)
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(
            Some(ship_types::SpaceId::new("test-space")),
            ship_types::Authority::Sdk,
        ),
        session_id: None,
    };

    let started = SessionStarted {
        session_id: session_id.to_string(),
        agent_id: "test-agent".to_string(),
        harness: "claude".to_string(),
        model: None,
        space_id: "test-space".to_string(),
        slug: Some(slug.to_string()),
        input: None,
        parent: None,
    };
    let env = EventEnvelope::new(canonical_event_types::SESSION_STARTED, session_id, &started)
        .unwrap()
        .with_actor_id("test");
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

// ── Fanout-lane drift fix: is_known_dispatch consults actor_state ─────
//
// Fanout lanes are live, registered processes that have NO jobs row
// (the pipeline-run carries the PARENT slug) and NO agent_sessions row
// (lanes are spawned directly by the fanout dispatcher). Before this
// fix Check B classified every live lane as a stray orphan and killed
// it ~one sweep after spawn — the supervisor-drift murder. The
// actor_state read model DOES name the lane (its spawn emits
// actor.created), so the sweep now recognizes it as a known dispatch.

/// Seed an `actor.created` for `slug` through the kernel. The
/// ActorProjection (always in the default projection bus) materializes
/// the `actors` row that the sweep's actor_state check reads.
async fn seed_actor_created(kernel: &Arc<Mutex<KernelRouter>>, slug: &str) {
    use ship_substrate::events::{CallerKind, EmitContext};
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };
    let payload = serde_json::json!({
        "kind": "pipeline-step",
        "environment_type": "local",
    });
    let env = EventEnvelope::new("actor.created", slug, &payload)
        .unwrap()
        .with_actor_id(slug);
    kernel.lock().await.emit(env, &ctx).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_does_not_kill_live_fanout_lane_with_actor_row() {
    // A live registered lane slug with an actor_state row but no jobs /
    // agent_sessions record. The pre-fix Check B would emit
    // process.killed; the actor_state arm of is_known_dispatch now
    // spares it.
    ship_platform::install_defaults();
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let lane_slug = "sui-eval-m7hbn6ga-build-haiku";
    let octx = make_octx(
        Arc::new(DeadExecutor),
        Arc::new(RegistryWithLiveSlug(lane_slug)),
    );

    seed_actor_created(&kernel, lane_slug).await;

    run_sweep(&kernel, &octx).await.unwrap();

    let killed: Vec<_> = find_events_by_type("process.killed")
        .into_iter()
        .filter(|e| e.entity_id == lane_slug)
        .collect();
    assert!(
        killed.is_empty(),
        "sweep must not kill a live fanout lane that has an actor_state row; \
         got: {killed:?}"
    );

    let drift: Vec<_> = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED)
        .into_iter()
        .filter(|e| e.entity_id == lane_slug)
        .collect();
    assert!(
        drift.is_empty(),
        "no drift for a live, actor-backed lane during normal lifecycle; got: {drift:?}"
    );
}

// ── Real orphan detection: dead handle (PID killed externally) ────────
//
// Acceptance criterion #3: a registered actor whose PID is killed out
// from under the daemon must be flagged drifted within one cadence. The
// executor reports the handle dead → drain_dead evicts it → the sweep
// emits supervisor.drift_detected. This is the honest "in-memory said
// alive, process is gone" signal, and it still fires for actors with no
// jobs row (bare actors / fanout lanes).

/// Registry whose single registered handle is reported dead by the
/// executor — `drain_dead` evicts and returns it, mirroring an
/// externally-killed PID. `live_slugs` is empty afterward (it was
/// drained), so Check B never sees it; the drain pass owns the signal.
struct RegistryWithDeadHandle(&'static str);
#[async_trait]
impl ProcessRegistryPort for RegistryWithDeadHandle {
    async fn register(&self, _slug: &str, _handle: ExecutorHandle) {}
    async fn unregister(&self, _slug: &str) {}
    async fn live_slugs(&self) -> Vec<String> { vec![] }
    async fn take(&self, _slug: &str) -> Option<ExecutorHandle> { None }
    async fn drain_dead(&self, _executor: &dyn SessionExecutor) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_flags_drift_for_externally_killed_actor() {
    ship_platform::install_defaults();
    let _tmp = tempfile::TempDir::new().unwrap();
    let kernel = make_kernel();
    let slug = "sweep-killed-actor-1";
    // The actor was spawned and registered (actor_state row exists), then
    // its PID was killed externally — the executor now reports it dead.
    let octx = make_octx(
        Arc::new(DeadExecutor),
        Arc::new(RegistryWithDeadHandle(slug)),
    );
    seed_actor_created(&kernel, slug).await;

    run_sweep(&kernel, &octx).await.unwrap();

    let drift = find_events_by_type(sup_events::SUPERVISOR_DRIFT_DETECTED);
    assert!(
        drift.iter().any(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["slug"] == slug && p["actual_state"] == "process_died"
        }),
        "externally-killed actor must be flagged drifted within one cadence"
    );

    let restored = find_events_by_type(sup_events::SUPERVISOR_COHERENCE_RESTORED);
    assert!(
        restored.iter().any(|e| {
            let p: serde_json::Value = serde_json::from_str(&e.payload_json).unwrap_or_default();
            p["slug"] == slug && p["action"] == "reaped_dead_handle"
        }),
        "coherence_restored with reaped_dead_handle for the bare dead actor"
    );
}
