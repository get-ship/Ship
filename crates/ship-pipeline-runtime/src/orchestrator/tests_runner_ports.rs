//! Agents-arc keystone — the runner driven through the domain-free ports.
//!
//! These tests exercise the runner mechanism with FAKE/in-memory port impls,
//! proving two things the keystone is about:
//!   1. the spawn/op-invocation port is threaded a domain-free `Authority`
//!      principal (not an agent/config struct), and the agent flows through it
//!      as `Authority::Agent { id }`;
//!   2. the coherence sweep's orphan check consults the `ProjectionReader`
//!      port (canned slug set) rather than reaching into projections — a slug
//!      the fake reports as known is spared, one it omits is reaped.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_substrate::events::kernel_router::{ActorConfig, KernelRouter};
use ship_substrate::events::pipeline::{
    event_types, Pipeline, PipelineRunStartedPayload, PipelineStep, StepType,
};
use ship_substrate::events::EventEnvelope;

use ship_code::dispatch_ports::{
    AgentPlatform, Authority, ExecutorHandle, IsolationStrategy, ProcessRegistryPort,
    ProjectionReader, SessionContext, SessionExecutor, SpawnPolicy,
};
use crate::orchestrator::dispatch::dispatch_pipeline_run_internal;
use crate::orchestrator::sweep::run_sweep;
use crate::orchestrator::OrchestratorContext;

// ── Fakes ───────────────────────────────────────────────────────────────────

/// Captures every `(principal, actor_id)` the runner hands the spawn port and
/// returns a canned env. Proves the principal is threaded domain-free.
#[derive(Clone, Default)]
struct CapturingSpawnPolicy {
    seen: Arc<StdMutex<Vec<(Authority, String)>>>,
}

#[async_trait]
impl SpawnPolicy for CapturingSpawnPolicy {
    async fn spawn_env(&self, principal: &Authority, actor_id: &str) -> HashMap<String, String> {
        self.seen
            .lock()
            .unwrap()
            .push((principal.clone(), actor_id.to_string()));
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), actor_id.to_string());
        env
    }
}

/// Returns a fixed "known dispatch" slug set and a fixed terminal answer, so a
/// sweep can be driven without touching any projection.
struct FakeProjectionReader {
    known: Vec<String>,
}

#[async_trait]
impl ProjectionReader for FakeProjectionReader {
    async fn known_dispatch_slugs(&self) -> Vec<String> {
        self.known.clone()
    }
    async fn is_run_terminal(&self, _entity_id: &str) -> bool {
        false
    }
}

struct OkIsolation(PathBuf);
#[async_trait]
impl IsolationStrategy for OkIsolation {
    async fn prepare(&self, _: &SessionContext) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(&self.0).ok();
        Ok(self.0.clone())
    }
    async fn cleanup(&self, _: &SessionContext) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Spawns, captures the env it was handed, then errors so dispatch stops before
/// the post-spawn monitor/event work.
struct EnvRecordingExecutor {
    captured_env: Arc<StdMutex<Option<HashMap<String, String>>>>,
}
#[async_trait]
impl SessionExecutor for EnvRecordingExecutor {
    async fn spawn(&self, ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        *self.captured_env.lock().unwrap() = Some(ctx.env.clone());
        Err(anyhow::anyhow!("recording executor: refusing to spawn"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool {
        false
    }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> {
        Ok(())
    }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Records stop calls and never finds a live process on resume.
struct CountingStopExecutor {
    stop_calls: Arc<AtomicUsize>,
}
#[async_trait]
impl SessionExecutor for CountingStopExecutor {
    async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("unused"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool {
        false
    }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> {
        self.stop_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    async fn resume_handle(&self, _: &str, _: Option<&str>, _: Option<&str>) -> Option<ExecutorHandle> {
        None
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
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
}

/// Reports one live slug and yields a handle on take — drives the sweep's
/// orphan check.
struct LiveSlugRegistry {
    slug: String,
    handle: Mutex<Option<ExecutorHandle>>,
}
#[async_trait]
impl ProcessRegistryPort for LiveSlugRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
    async fn live_slugs(&self) -> Vec<String> {
        vec![self.slug.clone()]
    }
    async fn take(&self, _: &str) -> Option<ExecutorHandle> {
        self.handle.lock().await.take()
    }
    async fn drain_dead(&self, _: &dyn SessionExecutor) -> Vec<String> {
        vec![]
    }
}

// ── Harness ───────────────────────────────────────────────────────────────

fn dispatch_config() -> ActorConfig {
    ActorConfig {
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
    }
}

fn make_kernel() -> Arc<Mutex<KernelRouter>> {
    let base = ship_substrate::project::get_global_dir().unwrap();
    let mut router = KernelRouter::new(base).unwrap();
    router
        .spawn_actor("service.orchestrator", dispatch_config())
        .unwrap();
    Arc::new(Mutex::new(router))
}

#[allow(clippy::too_many_arguments)]
fn octx(
    isolation: Arc<dyn IsolationStrategy>,
    executor: Arc<dyn SessionExecutor>,
    platform: Arc<dyn AgentPlatform>,
    registry: Arc<dyn ProcessRegistryPort>,
    spawn_policy: Arc<dyn SpawnPolicy>,
    projections: Arc<dyn ProjectionReader>,
) -> OrchestratorContext {
    OrchestratorContext {
        isolation,
        executor,
        platform,
        registry,
        probe: Arc::new(ship_code::process_tree_probe::SysinfoProcessTreeProbe::new()),
        spawn_policy,
        projections,
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: Arc::new(Mutex::new(HashMap::new())),
        run_locks: Arc::new(Mutex::new(HashMap::new())),
        event_dedup: Arc::new(crate::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(HashMap::new())),
        journal_store: Arc::new(durable_execution::InMemoryJournalStore::new()),
    }
}

fn single_agent_payload(pipeline_run_id: &str, slug: &str, agent: &str) -> PipelineRunStartedPayload {
    let step = PipelineStep {
        id: "implement".into(),
        step_type: StepType::Agent,
        agent: Some(agent.into()),
        goal: Some("do work".into()),
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
        invocation: None,
    };
    PipelineRunStartedPayload {
        pipeline_run_id: pipeline_run_id.into(),
        slug: slug.into(),
        agent: agent.into(),
        branch: format!("job/{slug}"),
        spec_path: ".artifacts/job-spec.md".into(),
        source_spec_path: None,
        plan_id: None,
        model: None,
        harness: None,
        depends_on: None,
        pipeline: None,
        steps: Some(Pipeline {
            steps: vec![step],
            auto_merge: false,
        }),
        pipeline_name: None,
        vars: None,
        spec_content: None,
        step_specs: None,
        headless: true,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha: None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

/// The runner hands the spawn/op-invocation port a domain-free `Authority`
/// principal, and the step's agent flows through it as `Authority::Agent`. The
/// resolved env is what the executor is handed verbatim.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dispatch_threads_agent_as_domain_free_authority_through_spawn_policy() {
    let tmp = TempDir::new().unwrap();
    let kernel = make_kernel();

    let payload = single_agent_payload("j-ports-1", "ports-1", "designer");
    let worktree = tmp.path().join("worktrees/ports-1");

    let spawn = CapturingSpawnPolicy::default();
    let captured_env = Arc::new(StdMutex::new(None));
    let ctx = octx(
        Arc::new(OkIsolation(worktree)),
        Arc::new(EnvRecordingExecutor {
            captured_env: captured_env.clone(),
        }),
        Arc::new(TestPlatform(tmp.path().join("worktrees"))),
        Arc::new(NoopRegistry),
        Arc::new(spawn.clone()),
        Arc::new(FakeProjectionReader { known: vec![] }),
    );

    let created =
        EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, &payload.pipeline_run_id, &payload)
            .unwrap();
    ship_runner::emit::persist_and_route(&kernel, created, &payload.slug, None)
        .await;

    dispatch_pipeline_run_internal(&kernel, &payload, &ctx).await;

    // The spawn port saw exactly the domain-free principal for the step agent,
    // targeting the run slug — no AgentConfig / PipelineStep crossed the seam.
    let seen = spawn.seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 1, "spawn policy invoked once for the single step");
    assert_eq!(
        seen[0].0,
        Authority::Agent {
            id: "designer".to_string()
        },
        "principal is the domain-free Authority::Agent for the step agent",
    );
    assert_eq!(seen[0].1, "ports-1", "actor_id is the run slug");

    // The env the port returned is what the executor was handed.
    let env = captured_env.lock().unwrap().clone().expect("executor spawned");
    assert_eq!(env.get("SHIP_ACTOR_ID").map(String::as_str), Some("ports-1"));
}

/// The coherence sweep's orphan check goes through `ProjectionReader`: a live
/// slug the fake reports as known is NOT killed.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_spares_slug_reported_known_by_projection_port() {
    let _tmp = TempDir::new().unwrap();
    let kernel = make_kernel();
    let stop_calls = Arc::new(AtomicUsize::new(0));

    let registry = Arc::new(LiveSlugRegistry {
        slug: "known-lane-1".to_string(),
        handle: Mutex::new(Some(ExecutorHandle {
            pid: Some(4242),
            inner: Box::new(()),
        })),
    });
    let ctx = octx(
        Arc::new(OkIsolation(PathBuf::from("/tmp"))),
        Arc::new(CountingStopExecutor {
            stop_calls: stop_calls.clone(),
        }),
        Arc::new(TestPlatform(PathBuf::from("/tmp/wt"))),
        registry,
        Arc::new(CapturingSpawnPolicy::default()),
        Arc::new(FakeProjectionReader {
            known: vec!["known-lane-1".to_string()],
        }),
    );

    run_sweep(&kernel, &ctx).await.unwrap();

    assert_eq!(
        stop_calls.load(Ordering::SeqCst),
        0,
        "a slug the projection port reports as a known dispatch is not reaped",
    );
}

/// The mirror case: a live slug ABSENT from the port's known set is reaped as a
/// true orphan — proving the sweep's decision is driven by the port, not by a
/// hidden projection read.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sweep_reaps_slug_absent_from_projection_port() {
    let _tmp = TempDir::new().unwrap();
    let kernel = make_kernel();
    let stop_calls = Arc::new(AtomicUsize::new(0));

    let registry = Arc::new(LiveSlugRegistry {
        slug: "orphan-lane-1".to_string(),
        handle: Mutex::new(Some(ExecutorHandle {
            pid: Some(7373),
            inner: Box::new(()),
        })),
    });
    let ctx = octx(
        Arc::new(OkIsolation(PathBuf::from("/tmp"))),
        Arc::new(CountingStopExecutor {
            stop_calls: stop_calls.clone(),
        }),
        Arc::new(TestPlatform(PathBuf::from("/tmp/wt"))),
        registry,
        Arc::new(CapturingSpawnPolicy::default()),
        // Known set deliberately excludes the live slug.
        Arc::new(FakeProjectionReader {
            known: vec!["some-other-slug".to_string()],
        }),
    );

    run_sweep(&kernel, &ctx).await.unwrap();

    assert_eq!(
        stop_calls.load(Ordering::SeqCst),
        1,
        "a slug absent from the projection port's known set is reaped as a stray",
    );
}
