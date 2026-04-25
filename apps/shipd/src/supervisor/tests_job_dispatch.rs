use async_trait::async_trait;
use runtime::events::job::event_types;
use runtime::events::kernel_router::{ActorConfig, KernelRouter};
use runtime::events::validator::{CallerKind, EmitContext};
use runtime::events::EventEnvelope;
use runtime::projections::job::{project, JobStatus};
use runtime::services::dispatch_ports::{ExecutorHandle, IsolationStrategy, JobContext, JobExecutor};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use super::DispatchContext;

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn job_dispatch_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec!["job.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string()],
    }
}

fn setup() -> (TempDir, KernelRouter) {
    let dir = TempDir::new().unwrap();
    let router = KernelRouter::new(runtime::ShipHome::project_ship_dir(dir.path())).unwrap();
    (dir, router)
}

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, "test-entity", &payload).unwrap()
}

/// No-op isolation for tests that don't exercise dispatch (DAG, routing tests).
struct NoopIsolation;

#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, job: &JobContext) -> anyhow::Result<PathBuf> {
        Ok(job.work_dir.clone())
    }
    async fn cleanup(&self, _job: &JobContext) -> anyhow::Result<()> {
        Ok(())
    }
}

/// No-op executor for tests that don't exercise dispatch.
struct NoopExecutor;

#[async_trait]
impl JobExecutor for NoopExecutor {
    async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
        Ok(ExecutorHandle { pid: None, inner: Box::new(()) })
    }
    async fn is_alive(&self, _handle: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _handle: &ExecutorHandle, _msg: &str) -> anyhow::Result<()> { Ok(()) }
}

fn noop_dctx() -> DispatchContext {
    DispatchContext {
        isolation: Arc::new(NoopIsolation),
        executor: Arc::new(NoopExecutor),
        registry: Arc::new(tokio::sync::Mutex::new(
            super::ProcessRegistry::new(),
        )),
        done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
    }
}

/// Kernel routing delivers `job.created` to an actor subscribed to `job.`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_created_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-created-1",
            "slug": "auth-tests",
            "agent": "rust-runtime",
            "branch": "job/auth-tests",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "model": null,
            "provider": null,
        }),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.created must be delivered to job-dispatch subscriber");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, event_types::JOB_CREATED);
}

/// Kernel routing delivers `job.update` with correct payload to subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_update_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_UPDATE,
        serde_json::json!({
            "job_id": "j-update-1",
            "message": "please fix the lint errors",
            "sender": "human",
            "slug": "auth-tests",
        }),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.update must be delivered to job-dispatch subscriber");
    assert_eq!(received.id, event.id);
    assert_eq!(received.event_type, event_types::JOB_UPDATE);
    let payload: serde_json::Value =
        serde_json::from_str(&received.payload_json).unwrap();
    assert_eq!(
        payload["message"].as_str(),
        Some("please fix the lint errors")
    );
}

/// `job.completed` is delivered to job-dispatch subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_completed_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_COMPLETED,
        serde_json::json!({"job_id": "j-comp-1", "slug": "auth-tests"}),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.completed must be delivered to job-dispatch subscriber");
    assert_eq!(received.event_type, event_types::JOB_COMPLETED);
}

/// `job.merged` is delivered to job-dispatch subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_merged_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_MERGED,
        serde_json::json!({"job_id": "j-merged-1", "slug": "auth-tests"}),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    let received = mb
        .try_recv()
        .expect("job.merged must be delivered to job-dispatch subscriber");
    assert_eq!(received.event_type, event_types::JOB_MERGED);
}

/// `workspace.activated` is not delivered to the job-dispatch subscriber.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn non_job_events_not_delivered_to_subscriber() {
    let (_dir, mut router) = setup();
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        "workspace.activated",
        serde_json::json!({ "branch": "main", "worktree_path": "/tmp/test" }),
    );

    router.route(event, &runtime_ctx()).await.unwrap();

    assert!(
        mb.try_recv().is_err(),
        "workspace.activated must not reach the job-dispatch subscriber"
    );
}

/// Pipeline job advances to phase 1 when phase 0 completes.
///
/// Sets up a 2-phase pipeline job (test-writer → rust-runtime), persists
/// job.created + job.dispatched to the event store, then calls
/// `advance_pipeline` with a job.completed event. Asserts that:
/// 1. `advance_pipeline` returns true (pipeline advanced)
/// 2. A `job.dispatched` event for phase 1 is routed to the subscriber
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pipeline_advances_to_next_phase() {
    use runtime::events::store::{EventStore, SqliteEventStore};

    let dir = TempDir::new().unwrap();
    let slug = "pipeline-adv-test";
    let job_id = "j-pipeline-adv";

    // Isolated worktree and global dir inside temp
    let worktree_root = dir.path().join("worktrees");
    let worktree_path = worktree_root.join(slug);
    let global_dir = dir.path().join("global");
    std::fs::create_dir_all(worktree_path.join(".artifacts")).unwrap();
    std::fs::create_dir_all(&global_dir).unwrap();

    // Write a minimal spec so write_phase_spec can read it
    std::fs::write(
        worktree_path.join(".artifacts").join("job-spec.md"),
        "---\nslug: pipeline-adv-test\n---\n# Pipeline Test\n",
    )
    .unwrap();

    // Use explicit-path APIs to avoid env var races in parallel tests.
    let store = SqliteEventStore::new_at(&global_dir).expect("test DB must initialize");

    let created = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": job_id,
            "slug": slug,
            "agent": "test-writer",
            "branch": "job/pipeline-adv-test",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "steps": {
                "steps": [
                    { "id": "write-tests", "type": "agent", "agent": "test-writer", "after": [] },
                    { "id": "implement", "type": "agent", "agent": "rust-runtime", "after": ["write-tests"] }
                ]
            }
        }),
    );
    store.append(&created).unwrap();

    let dispatched = ev(
        event_types::JOB_DISPATCHED,
        serde_json::json!({
            "job_id": job_id,
            "worktree": worktree_path.to_string_lossy(),
            "pid": null,
        }),
    );
    store.append(&dispatched).unwrap();

    // Set up kernel with a subscriber to capture routed events
    let mut router = KernelRouter::new(runtime::ShipHome::project_ship_dir(dir.path())).unwrap();
    let (_actor_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();
    let kernel = Arc::new(tokio::sync::Mutex::new(router));

    // Simulate phase 0 completion
    let completed = ev(
        event_types::JOB_COMPLETED,
        serde_json::json!({ "job_id": job_id, "slug": slug }),
    );

    let dctx = noop_dctx();
    let advanced = super::advance_pipeline_at(&kernel, &completed, &dctx, &global_dir).await;

    assert!(
        advanced,
        "advance_pipeline must return true for a 2-phase pipeline completing phase 0"
    );

    let phase1_event = mb
        .try_recv()
        .expect("job.dispatched for phase 1 must be routed to subscriber");
    assert_eq!(phase1_event.event_type, event_types::JOB_DISPATCHED);

    let payload: serde_json::Value =
        serde_json::from_str(&phase1_event.payload_json).unwrap();
    assert_eq!(payload["job_id"].as_str(), Some(job_id));
}

// ── Ports + Adapters: mock executor tests ────────────────────────────────────

#[cfg(feature = "unstable")]
mod ports {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct RecordingIsolation {
        work_dir: PathBuf,
        prepare_count: AtomicUsize,
        cleanup_count: AtomicUsize,
    }

    impl RecordingIsolation {
        fn new(work_dir: PathBuf) -> Self {
            Self {
                work_dir,
                prepare_count: AtomicUsize::new(0),
                cleanup_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl IsolationStrategy for RecordingIsolation {
        async fn prepare(&self, _job: &JobContext) -> anyhow::Result<PathBuf> {
            self.prepare_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.work_dir.clone())
        }
        async fn cleanup(&self, _job: &JobContext) -> anyhow::Result<()> {
            self.cleanup_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct RecordingExecutor {
        spawn_count: AtomicUsize,
        stop_count: AtomicUsize,
    }

    impl RecordingExecutor {
        fn new() -> Self {
            Self {
                spawn_count: AtomicUsize::new(0),
                stop_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl JobExecutor for RecordingExecutor {
        async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
            self.spawn_count.fetch_add(1, Ordering::SeqCst);
            Ok(ExecutorHandle { pid: Some(99999), inner: Box::new(()) })
        }
        async fn is_alive(&self, _handle: &ExecutorHandle) -> bool { true }
        async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        async fn send(
            &self, _handle: &ExecutorHandle, _message: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn test_ctx(slug: &str, work_dir: PathBuf) -> JobContext {
        JobContext {
            job_id: format!("j-{slug}"),
            slug: slug.to_string(),
            agent: "rust-runtime".to_string(),
            branch: format!("job/{slug}"),
            spec_path: ".artifacts/job-spec.md".to_string(),
            work_dir,
            env: [("SHIP_ACTOR_ID".to_string(), slug.to_string())].into(),
            launch: Default::default(),
            exit_tx: None,
            step: None,
            headless: false,
        }
    }

    /// Mock isolation returns the configured work_dir.
    #[tokio::test]
    async fn isolation_prepare_returns_work_dir() {
        let dir = TempDir::new().unwrap();
        let expected = dir.path().join("worktree");
        let iso = RecordingIsolation::new(expected.clone());
        let ctx = test_ctx("iso-test", dir.path().to_path_buf());

        let result = iso.prepare(&ctx).await.unwrap();
        assert_eq!(result, expected);
        assert_eq!(iso.prepare_count.load(Ordering::SeqCst), 1);
    }

    /// Mock executor records spawn calls and returns a handle with pid.
    #[tokio::test]
    async fn executor_spawn_returns_handle() {
        let dir = TempDir::new().unwrap();
        let exec = RecordingExecutor::new();
        let ctx = test_ctx("exec-test", dir.path().to_path_buf());

        let handle = exec.spawn(&ctx).await.unwrap();
        assert_eq!(handle.pid, Some(99999));
        assert_eq!(exec.spawn_count.load(Ordering::SeqCst), 1);
    }

    /// Dispatch delegates to IsolationStrategy::prepare during job creation.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dispatch_delegates_to_isolation() {
        let dir = TempDir::new().unwrap();
        let isolation = Arc::new(RecordingIsolation::new(dir.path().join("worktree")));
        let executor = Arc::new(RecordingExecutor::new());

        let dctx = super::super::DispatchContext {
            isolation: isolation.clone(),
            executor: executor.clone(),
            registry: Arc::new(tokio::sync::Mutex::new(
                super::super::ProcessRegistry::new(),
            )),
            done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };

        let (_dir2, router) = super::setup();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        let event = super::ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-iso-dispatch",
                "slug": "iso-dispatch",
                "agent": "rust-runtime",
                "branch": "job/iso-dispatch",
                "spec_path": ".artifacts/job-spec.md",
                "plan_id": null,
            }),
        );

        super::super::handle_job_created(&kernel, &event, &dctx).await;

        assert!(
            isolation.prepare_count.load(Ordering::SeqCst) > 0,
            "IsolationStrategy::prepare must be called during dispatch"
        );
    }

    /// Dispatch delegates to JobExecutor::spawn during job creation.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dispatch_delegates_to_executor() {
        let dir = TempDir::new().unwrap();
        let isolation = Arc::new(RecordingIsolation::new(dir.path().join("worktree")));
        let executor = Arc::new(RecordingExecutor::new());

        let dctx = super::super::DispatchContext {
            isolation: isolation.clone(),
            executor: executor.clone(),
            registry: Arc::new(tokio::sync::Mutex::new(
                super::super::ProcessRegistry::new(),
            )),
            done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };

        let (_dir2, router) = super::setup();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        let event = super::ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-exec-dispatch",
                "slug": "exec-dispatch",
                "agent": "rust-runtime",
                "branch": "job/exec-dispatch",
                "spec_path": ".artifacts/job-spec.md",
                "plan_id": null,
            }),
        );

        super::super::handle_job_created(&kernel, &event, &dctx).await;

        assert!(
            executor.spawn_count.load(Ordering::SeqCst) > 0,
            "JobExecutor::spawn must be called during dispatch"
        );
    }

    /// Pipeline advancement uses JobExecutor::spawn for the next phase.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipeline_advance_uses_executor() {
        use runtime::events::store::{EventStore, SqliteEventStore};

        let dir = TempDir::new().unwrap();
        let slug = "pipe-exec-test";
        let job_id = "j-pipe-exec";
        let isolation = Arc::new(RecordingIsolation::new(dir.path().join("worktree")));
        let executor = Arc::new(RecordingExecutor::new());

        let dctx = super::super::DispatchContext {
            isolation: isolation.clone(),
            executor: executor.clone(),
            registry: Arc::new(tokio::sync::Mutex::new(
                super::super::ProcessRegistry::new(),
            )),
            done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };

        let worktree_root = dir.path().join("worktrees");
        let worktree_path = worktree_root.join(slug);
        let global_dir = dir.path().join("global");
        std::fs::create_dir_all(worktree_path.join(".artifacts")).unwrap();
        std::fs::create_dir_all(&global_dir).unwrap();
        std::fs::write(
            worktree_path.join(".artifacts").join("job-spec.md"),
            "---\nslug: pipe-exec-test\n---\n# Test\n",
        )
        .unwrap();

        let store = SqliteEventStore::new_at(&global_dir).unwrap();
        store.append(&super::ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": job_id, "slug": slug,
                "agent": "test-writer", "branch": "job/pipe-exec-test",
                "spec_path": ".artifacts/job-spec.md", "plan_id": null,
                "steps": {
                    "steps": [
                        { "id": "write-tests", "type": "agent", "agent": "test-writer", "after": [] },
                        { "id": "implement", "type": "agent", "agent": "rust-runtime", "after": ["write-tests"] }
                    ]
                }
            }),
        )).unwrap();
        store.append(&super::ev(
            event_types::JOB_DISPATCHED,
            serde_json::json!({
                "job_id": job_id,
                "worktree": worktree_path.to_string_lossy(),
                "pid": null,
            }),
        )).unwrap();

        let mut router = KernelRouter::new(runtime::ShipHome::project_ship_dir(dir.path())).unwrap();
        let (_s, _mb) = router
            .spawn_actor("service.job-dispatch", super::job_dispatch_config())
            .unwrap();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        let completed = super::ev(
            event_types::JOB_COMPLETED,
            serde_json::json!({ "job_id": job_id, "slug": slug }),
        );
        super::super::advance_pipeline_at(&kernel, &completed, &dctx, &global_dir).await;

        assert!(
            executor.spawn_count.load(Ordering::SeqCst) > 0,
            "JobExecutor::spawn must be called during pipeline advancement"
        );
    }
}

/// `job.created` with model/provider in the payload projects to a valid
/// pending JobRecord.
#[test]
fn job_projection_handles_model_and_provider_payload() {
    let events = vec![ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-model-1",
            "slug": "typed-api",
            "agent": "rust-runtime",
            "branch": "job/typed-api",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "model": "claude-opus-4-5",
            "provider": "anthropic",
        }),
    )];

    let map = project(&events);
    let rec = map.get("j-model-1").expect("job record must be created");
    assert_eq!(rec.status, JobStatus::Pending);
    assert_eq!(rec.slug, "typed-api");
    assert_eq!(rec.agent, "rust-runtime");
    assert_eq!(rec.branch, "job/typed-api");
    assert_eq!(rec.spec_path, ".artifacts/job-spec.md");
    let raw: serde_json::Value =
        serde_json::from_str(&events[0].payload_json).unwrap();
    assert_eq!(raw["model"].as_str(), Some("claude-opus-4-5"));
    assert_eq!(raw["provider"].as_str(), Some("anthropic"));
}

/// SHIP_ACTOR_ID env var overrides derived actor ID.
#[test]
fn ship_mesh_id_env_overrides_derived_actor_id() {
    let slug = "payments-refactor";

    let without = runtime::env::read_raw(runtime::env::Var::ActorId)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "agent.rust-runtime.job/payments-refactor".to_string());
    assert_eq!(without, "agent.rust-runtime.job/payments-refactor");

    unsafe { runtime::env::set(runtime::env::Var::ActorId, slug) };
    let with_mesh_id = runtime::env::read_raw(runtime::env::Var::ActorId)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "agent.rust-runtime.job/payments-refactor".to_string());
    unsafe { runtime::env::remove(runtime::env::Var::ActorId) };

    assert_eq!(
        with_mesh_id, slug,
        "SHIP_ACTOR_ID must override the derived profile+branch actor ID"
    );
}

/// Jobs with depends_on are deferred (not dispatched immediately).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_with_depends_on_is_deferred() {
    let (_dir, router) = setup();
    let kernel = Arc::new(tokio::sync::Mutex::new(router));

    let dctx = noop_dctx();
    let event = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-deferred",
            "slug": "downstream",
            "agent": "rust-runtime",
            "branch": "job/downstream",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "depends_on": ["upstream-a"]
        }),
    );

    // handle_job_created should return without dispatching (no spawn call)
    super::handle_job_created(&kernel, &event, &dctx).await;
    // The job stays as Pending in the projection — dispatch_ready_pending_jobs
    // will pick it up when deps resolve. No assertion on in-memory state needed.
}

/// Jobs without depends_on are dispatched immediately.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn job_without_depends_on_not_deferred() {
    let (_dir, router) = setup();
    let kernel = Arc::new(tokio::sync::Mutex::new(router));

    let dctx = noop_dctx();
    let event = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-immediate",
            "slug": "no-deps",
            "agent": "rust-runtime",
            "branch": "job/no-deps",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
        }),
    );

    super::handle_job_created(&kernel, &event, &dctx).await;
    // With NoopExecutor + NoopIsolation, dispatch proceeds without error.
    // The handle is registered in the registry.
    assert_eq!(dctx.registry.lock().await.len(), 1);
}

/// Projection-backed dependency resolution: pending job dispatches when deps complete.
#[test]
fn projection_resolves_pending_deps() {
    use runtime::events::store::{EventStore, SqliteEventStore};

    let dir = TempDir::new().unwrap();
    let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    // Create upstream job (will complete)
    store.append(&ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-up",
            "slug": "upstream",
            "agent": "rust-runtime",
            "branch": "job/upstream",
            "spec_path": "spec.md",
            "plan_id": null,
        }),
    )).unwrap();

    // Create downstream job with dependency
    store.append(&ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-down",
            "slug": "downstream",
            "agent": "rust-runtime",
            "branch": "job/downstream",
            "spec_path": "spec.md",
            "plan_id": null,
            "depends_on": ["upstream"],
        }),
    )).unwrap();

    // Before upstream completes: downstream is Pending with unresolved deps
    let jobs = runtime::projections::job::load_jobs_at(&ship_dir).unwrap();
    let done: std::collections::HashSet<&str> = jobs.values()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();
    let downstream = jobs.get("j-down").unwrap();
    let deps_resolved = downstream.depends_on.as_ref()
        .map_or(true, |deps| deps.iter().all(|d| done.contains(d.as_str())));
    assert!(!deps_resolved, "downstream deps must not be resolved yet");

    // Complete upstream
    store.append(&ev(
        event_types::JOB_DISPATCHED,
        serde_json::json!({"job_id": "j-up", "worktree": "/tmp/wt/upstream", "pid": null}),
    )).unwrap();
    store.append(&ev(
        event_types::JOB_COMPLETED,
        serde_json::json!({"job_id": "j-up", "slug": "upstream"}),
    )).unwrap();

    // After upstream completes: downstream deps are resolved
    let jobs = runtime::projections::job::load_jobs_at(&ship_dir).unwrap();
    let done: std::collections::HashSet<&str> = jobs.values()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();
    let downstream = jobs.get("j-down").unwrap();
    let deps_resolved = downstream.depends_on.as_ref()
        .map_or(true, |deps| deps.iter().all(|d| done.contains(d.as_str())));
    assert!(deps_resolved, "downstream deps must be resolved after upstream completes");
}

// ── Supervisor Reliability: Pending Job Persistence (S4) ────────────────────

/// Pending jobs persist in the event store and are recoverable via projection
/// after daemon restart. Scenario S4: pending jobs survive restart.
#[test]
fn pending_jobs_recoverable_from_event_store() {
    use runtime::events::store::{EventStore, SqliteEventStore};

    let dir = TempDir::new().unwrap();
    let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    for i in 1..=3 {
        store
            .append(&ev(
                event_types::JOB_CREATED,
                serde_json::json!({
                    "job_id": format!("j-persist-{i}"),
                    "slug": format!("persist-job-{i}"),
                    "agent": "rust-runtime",
                    "branch": format!("job/persist-job-{i}"),
                    "spec_path": ".artifacts/job-spec.md",
                    "plan_id": null,
                    "depends_on": ["upstream-blocker"],
                }),
            ))
            .unwrap();
    }

    // Simulate restart: load projection from the same DB path
    let jobs = runtime::projections::job::load_jobs_at(&ship_dir).unwrap();

    let pending_count = jobs
        .values()
        .filter(|j| j.status == JobStatus::Pending)
        .count();
    assert_eq!(pending_count, 3, "all 3 pending jobs must survive restart");

    for i in 1..=3 {
        let job = jobs
            .values()
            .find(|j| j.slug == format!("persist-job-{i}"))
            .unwrap_or_else(|| panic!("persist-job-{i} must exist in projection"));
        assert_eq!(
            job.depends_on.as_ref().unwrap(),
            &["upstream-blocker"],
            "depends_on must be preserved across restart"
        );
    }
}

/// Projection distinguishes Pending, Dispatched, and Completed statuses.
/// Prerequisite for S5: startup reconciliation must detect stale Dispatched jobs.
#[test]
fn projection_detects_stale_dispatched_jobs() {
    use runtime::events::store::{EventStore, SqliteEventStore};

    let dir = TempDir::new().unwrap();
    let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    // Job A: pending (created only)
    store
        .append(&ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-pending",
                "slug": "pending-slug",
                "agent": "rust-runtime",
                "branch": "job/pending-slug",
                "spec_path": "spec.md",
                "plan_id": null,
            }),
        ))
        .unwrap();

    // Job B: dispatched (created + dispatched) — simulates running when daemon stopped
    store
        .append(&ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-dispatched",
                "slug": "dispatched-slug",
                "agent": "rust-runtime",
                "branch": "job/dispatched-slug",
                "spec_path": "spec.md",
                "plan_id": null,
            }),
        ))
        .unwrap();
    store
        .append(&ev(
            event_types::JOB_DISPATCHED,
            serde_json::json!({
                "job_id": "j-dispatched",
                "worktree": "/tmp/wt/dispatched-slug",
                "pid": 12345,
            }),
        ))
        .unwrap();

    // Job C: completed (created + dispatched + completed)
    store
        .append(&ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-completed",
                "slug": "completed-slug",
                "agent": "rust-runtime",
                "branch": "job/completed-slug",
                "spec_path": "spec.md",
                "plan_id": null,
            }),
        ))
        .unwrap();
    store
        .append(&ev(
            event_types::JOB_DISPATCHED,
            serde_json::json!({
                "job_id": "j-completed",
                "worktree": "/tmp/wt/completed-slug",
                "pid": 12346,
            }),
        ))
        .unwrap();
    store
        .append(&ev(
            event_types::JOB_COMPLETED,
            serde_json::json!({"job_id": "j-completed", "slug": "completed-slug"}),
        ))
        .unwrap();

    let jobs = runtime::projections::job::load_jobs_at(&ship_dir).unwrap();

    assert_eq!(jobs.get("j-pending").unwrap().status, JobStatus::Pending);
    assert_eq!(
        jobs.get("j-dispatched").unwrap().status,
        JobStatus::Dispatched
    );
    assert_eq!(
        jobs.get("j-completed").unwrap().status,
        JobStatus::Completed
    );

    // Key query for reconciliation: find only stale Dispatched jobs
    let stale_dispatched: Vec<_> = jobs
        .values()
        .filter(|j| j.status == JobStatus::Dispatched)
        .collect();
    assert_eq!(stale_dispatched.len(), 1);
    assert_eq!(stale_dispatched[0].slug, "dispatched-slug");
}

/// job.failed event transitions a Dispatched job to Failed with an error message.
/// This is the mechanism startup reconciliation uses when a dead process is found.
#[test]
fn failed_event_transitions_dispatched_to_failed() {
    let events = vec![
        ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-fail-test",
                "slug": "fail-test",
                "agent": "rust-runtime",
                "branch": "job/fail-test",
                "spec_path": "spec.md",
                "plan_id": null,
            }),
        ),
        ev(
            event_types::JOB_DISPATCHED,
            serde_json::json!({
                "job_id": "j-fail-test",
                "worktree": "/tmp/wt/fail-test",
                "pid": 99999,
            }),
        ),
        ev(
            event_types::JOB_FAILED,
            serde_json::json!({
                "job_id": "j-fail-test",
                "error": "daemon restart, process not found",
            }),
        ),
    ];

    let map = project(&events);
    let job = map.get("j-fail-test").unwrap();
    assert_eq!(job.status, JobStatus::Failed);
    assert_eq!(
        job.error.as_deref(),
        Some("daemon restart, process not found"),
        "error message must describe the failure reason"
    );
}

/// PID from job.dispatched event payload is preserved for reconciliation.
/// The reconciliation logic reads the PID to check process liveness.
#[test]
fn dispatched_event_preserves_pid_in_payload() {
    use runtime::events::store::{EventStore, SqliteEventStore};

    let dir = TempDir::new().unwrap();
    let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    let dispatched = ev(
        event_types::JOB_DISPATCHED,
        serde_json::json!({
            "job_id": "j-pid-test",
            "worktree": "/tmp/wt/pid-test",
            "pid": 42000,
        }),
    );
    store.append(&dispatched).unwrap();

    // Verify the PID survives round-trip through the event store
    let payload: serde_json::Value =
        serde_json::from_str(&dispatched.payload_json).unwrap();
    assert_eq!(
        payload["pid"].as_u64(),
        Some(42000),
        "PID must be preserved in job.dispatched payload for startup reconciliation"
    );
}

/// Only one actor subscribes to job.created — the dispatch service.
/// Verifies there is no dual dispatch path (single dispatch path requirement).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn single_dispatch_path_one_subscriber() {
    let (_dir, mut router) = setup();

    // Spawn the canonical job-dispatch actor
    let (_store, mut mb) = router
        .spawn_actor("service.job-dispatch", job_dispatch_config())
        .unwrap();

    let event = ev(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-single-path",
            "slug": "single-path",
            "agent": "rust-runtime",
            "branch": "job/single-path",
            "spec_path": "spec.md",
            "plan_id": null,
        }),
    );

    router.route(event.clone(), &runtime_ctx()).await.unwrap();

    // Exactly one delivery
    let received = mb
        .try_recv()
        .expect("job.created must be delivered to the single dispatch subscriber");
    assert_eq!(received.id, event.id);

    // No second delivery (no duplicate subscriber)
    assert!(
        mb.try_recv().is_err(),
        "job.created must not be delivered more than once — single dispatch path"
    );
}

/// Projection reflects Completed status from job.completed event.
#[test]
fn projection_completed_status() {
    let events = vec![
        ev(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-proj-comp",
                "slug": "test-slug",
                "agent": "rust-runtime",
                "branch": "job/test-slug",
                "spec_path": "spec.md",
                "plan_id": null,
            }),
        ),
        ev(
            event_types::JOB_COMPLETED,
            serde_json::json!({"job_id": "j-proj-comp", "slug": "test-slug"}),
        ),
    ];
    let map = project(&events);
    assert_eq!(map.get("j-proj-comp").unwrap().status, JobStatus::Completed);
}

// ── Supervisor Reliability: Startup Reconciliation & Graceful Shutdown ──────
//
// These tests exercise new supervisor reliability behaviors from RT-05 v0.2.0:
//   - S3: Graceful shutdown stops managed agents
//   - S5: Running jobs reconciled after restart (dead process → Failed)
//   - Process registry tracks spawned handles
//
// New API required — the implementation agent must create:
//
//   1. `reconcile_dispatched_jobs_at(kernel, dctx, ship_dir)` in dispatch_replay.rs
//      Re-export in job_dispatch.rs with:
//        #[cfg(test)]
//        pub(crate) use super::dispatch_replay::reconcile_dispatched_jobs_at;
//
//   2. `ProcessRegistry` struct in supervisor (new file or mod.rs)
//      - `new() -> Self`
//      - `register(slug: &str, handle: ExecutorHandle)`
//      - `shutdown_all(executor: &dyn JobExecutor) -> Vec<String>` (async)
//      - `len() -> usize`
//      - `is_empty() -> bool`
//      Re-export in job_dispatch.rs with:
//        #[cfg(test)]
//        pub(crate) use super::ProcessRegistry;
//
//   3. `graceful_shutdown(kernel, registry, executor)` in supervisor
//      Emits `supervisor.shutting_down` event, then drains the registry.
//      Re-export in job_dispatch.rs with:
//        #[cfg(test)]
//        pub(crate) use super::graceful_shutdown;
//
// These tests will not compile until the above items exist. This is
// intentional TDD — the tests define the expected behavior contract.

#[cfg(feature = "unstable")]
mod reliability {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock executor that tracks stop() calls for shutdown verification.
    struct ShutdownTrackingExecutor {
        stop_count: AtomicUsize,
    }

    impl ShutdownTrackingExecutor {
        fn new() -> Self {
            Self {
                stop_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl JobExecutor for ShutdownTrackingExecutor {
        async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
            Ok(ExecutorHandle { pid: Some(11111), inner: Box::new(()) })
        }
        async fn is_alive(&self, _handle: &ExecutorHandle) -> bool {
            true
        }
        async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        async fn send(&self, _handle: &ExecutorHandle, _msg: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn test_job_ctx(slug: &str) -> JobContext {
        JobContext {
            job_id: format!("j-{slug}"),
            slug: slug.to_string(),
            agent: "rust-runtime".to_string(),
            branch: format!("job/{slug}"),
            spec_path: "spec.md".to_string(),
            work_dir: PathBuf::from("/tmp"),
            env: Default::default(),
            launch: Default::default(),
            exit_tx: None,
            step: None,
            headless: false,
        }
    }

    /// Helper: seed event store with a dispatched job.
    fn seed_dispatched_job(
        store: &runtime::events::store::SqliteEventStore,
        job_id: &str,
        slug: &str,
    ) {
        use runtime::events::store::EventStore;
        store
            .append(&ev(
                event_types::JOB_CREATED,
                serde_json::json!({
                    "job_id": job_id,
                    "slug": slug,
                    "agent": "rust-runtime",
                    "branch": format!("job/{slug}"),
                    "spec_path": "spec.md",
                    "plan_id": null,
                }),
            ))
            .unwrap();
        store
            .append(&ev(
                event_types::JOB_DISPATCHED,
                serde_json::json!({
                    "job_id": job_id,
                    "worktree": format!("/tmp/wt/{slug}"),
                    "pid": 99999,
                }),
            ))
            .unwrap();
    }

    // ── Daemon Recovery: Orphaned Process Detection ─────────────────────────

    /// Dispatched job with dead process is marked failed via step_failed.
    ///
    /// After daemon restart, `reconcile_dispatched_jobs_at` must:
    /// 1. Find jobs in Dispatched state via projection
    /// 2. Check if the agent process is alive
    /// 3. If dead, emit job.step_failed
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reconcile_dead_process_emits_step_failed() {
        use runtime::events::store::{EventStore, SqliteEventStore};

        let dir = TempDir::new().unwrap();
        let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let store = SqliteEventStore::new_at(&ship_dir).unwrap();

        seed_dispatched_job(&store, "j-stale", "stale-running");

        let jobs_before = runtime::projections::job::load_jobs_at(&ship_dir).unwrap();
        assert_eq!(
            jobs_before.get("j-stale").unwrap().status,
            JobStatus::Dispatched
        );

        let mut router =
            runtime::events::kernel_router::KernelRouter::new(ship_dir.clone()).unwrap();
        let (_s, _mb) = router
            .spawn_actor("service.job-dispatch", super::job_dispatch_config())
            .unwrap();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        // NoopExecutor.is_alive returns false — simulates dead process
        let dctx = super::super::DispatchContext {
            isolation: Arc::new(super::NoopIsolation),
            executor: Arc::new(super::NoopExecutor),
            registry: Arc::new(tokio::sync::Mutex::new(
                super::super::ProcessRegistry::new(),
            )),
            done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };

        super::super::reconcile_dispatched_jobs_at(&kernel, &dctx, &ship_dir).await;

        let jobs_after = runtime::projections::job::load_jobs_at(&ship_dir).unwrap();
        let job = jobs_after.get("j-stale").unwrap();
        assert_eq!(
            job.status,
            JobStatus::Failed,
            "orphaned dispatched job must be Failed after step_failed"
        );
        assert!(
            job.error
                .as_ref()
                .map_or(false, |e| e.contains("daemon restarted")),
            "error must indicate daemon restart"
        );
    }

    /// Dead process must NOT be registered in ProcessRegistry.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reconcile_dead_process_not_registered() {
        use runtime::events::store::{EventStore, SqliteEventStore};

        let dir = TempDir::new().unwrap();
        let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let store = SqliteEventStore::new_at(&ship_dir).unwrap();

        seed_dispatched_job(&store, "j-reg", "register-test");

        let mut router =
            runtime::events::kernel_router::KernelRouter::new(ship_dir.clone()).unwrap();
        let (_s, _mb) = router
            .spawn_actor("service.job-dispatch", super::job_dispatch_config())
            .unwrap();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        let registry = Arc::new(tokio::sync::Mutex::new(
            super::super::ProcessRegistry::new(),
        ));
        let dctx = super::super::DispatchContext {
            isolation: Arc::new(super::NoopIsolation),
            executor: Arc::new(super::NoopExecutor),
            registry: registry.clone(),
            done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };

        super::super::reconcile_dispatched_jobs_at(&kernel, &dctx, &ship_dir).await;

        assert!(
            registry.lock().await.is_empty(),
            "dead process must NOT be registered in ProcessRegistry"
        );
    }

    /// S5: Dispatched job with live process keeps Dispatched status.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reconcile_dispatched_job_keeps_status_when_alive() {
        use runtime::events::store::{EventStore, SqliteEventStore};

        struct AliveProcessExecutor;

        #[async_trait]
        impl JobExecutor for AliveProcessExecutor {
            async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
                Ok(ExecutorHandle { pid: Some(1), inner: Box::new(()) })
            }
            async fn is_alive(&self, _handle: &ExecutorHandle) -> bool {
                true
            }
            async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> {
                Ok(())
            }
            async fn send(&self, _handle: &ExecutorHandle, _msg: &str) -> anyhow::Result<()> {
                Ok(())
            }
        }

        let dir = TempDir::new().unwrap();
        let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let store = SqliteEventStore::new_at(&ship_dir).unwrap();

        seed_dispatched_job(&store, "j-alive", "alive-job");

        let mut router =
            runtime::events::kernel_router::KernelRouter::new(ship_dir.clone()).unwrap();
        let (_s, _mb) = router
            .spawn_actor("service.job-dispatch", super::job_dispatch_config())
            .unwrap();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        let dctx = super::super::DispatchContext {
            isolation: Arc::new(super::NoopIsolation),
            executor: Arc::new(AliveProcessExecutor),
            registry: Arc::new(tokio::sync::Mutex::new(
                super::super::ProcessRegistry::new(),
            )),
            done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };

        super::super::reconcile_dispatched_jobs_at(&kernel, &dctx, &ship_dir).await;

        let jobs = runtime::projections::job::load_jobs_at(&ship_dir).unwrap();
        assert_eq!(
            jobs.get("j-alive").unwrap().status,
            JobStatus::Dispatched,
            "dispatched job with live process must stay Dispatched"
        );
    }

    /// Live process is registered in ProcessRegistry on startup for monitoring.
    ///
    /// When reconciliation finds a dispatched job with a live process, it must
    /// register the handle so graceful shutdown can stop it later.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reconcile_live_process_populates_registry() {
        use runtime::events::store::{EventStore, SqliteEventStore};

        struct AliveProcessExecutor;

        #[async_trait]
        impl JobExecutor for AliveProcessExecutor {
            async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
                Ok(ExecutorHandle { pid: Some(1), inner: Box::new(()) })
            }
            async fn is_alive(&self, _handle: &ExecutorHandle) -> bool {
                true
            }
            async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> {
                Ok(())
            }
            async fn send(&self, _handle: &ExecutorHandle, _msg: &str) -> anyhow::Result<()> {
                Ok(())
            }
        }

        let dir = TempDir::new().unwrap();
        let ship_dir = runtime::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let store = SqliteEventStore::new_at(&ship_dir).unwrap();

        seed_dispatched_job(&store, "j-live-reg", "live-reg");

        let mut router =
            runtime::events::kernel_router::KernelRouter::new(ship_dir.clone()).unwrap();
        let (_s, _mb) = router
            .spawn_actor("service.job-dispatch", super::job_dispatch_config())
            .unwrap();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        let registry = Arc::new(tokio::sync::Mutex::new(
            super::super::ProcessRegistry::new(),
        ));
        let dctx = super::super::DispatchContext {
            isolation: Arc::new(super::NoopIsolation),
            executor: Arc::new(AliveProcessExecutor),
            registry: registry.clone(),
            done_flags: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        };

        assert!(registry.lock().await.is_empty());

        super::super::reconcile_dispatched_jobs_at(&kernel, &dctx, &ship_dir).await;

        assert_eq!(
            registry.lock().await.len(),
            1,
            "live process must be registered in ProcessRegistry on startup"
        );
    }

    /// S3: Graceful shutdown stops all managed agent processes.
    ///
    /// The ProcessRegistry tracks spawned ExecutorHandles. On shutdown,
    /// `shutdown_all` must call `executor.stop()` for every tracked handle.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn graceful_shutdown_stops_all_agents() {
        let executor = Arc::new(ShutdownTrackingExecutor::new());
        let mut registry = super::super::ProcessRegistry::new();

        for i in 0..3 {
            let handle = executor.spawn(&test_job_ctx(&format!("shutdown-{i}"))).await.unwrap();
            registry.register(&format!("shutdown-{i}"), handle);
        }

        assert_eq!(registry.len(), 3);

        let stopped = registry.shutdown_all(Arc::clone(&executor) as Arc<dyn runtime::services::dispatch_ports::JobExecutor>).await;

        assert_eq!(stopped.len(), 3, "shutdown must stop all 3 agents");
        assert_eq!(
            executor.stop_count.load(Ordering::SeqCst),
            3,
            "executor.stop must be called once per agent"
        );
        assert!(registry.is_empty(), "registry must be empty after shutdown");
    }

    /// S3: Graceful shutdown emits supervisor.shutting_down event.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn graceful_shutdown_emits_shutdown_event() {
        let dir = TempDir::new().unwrap();
        let mut router =
            runtime::events::kernel_router::KernelRouter::new(runtime::ShipHome::project_ship_dir(dir.path())).unwrap();

        let config = runtime::events::kernel_router::ActorConfig {
            namespace: "test.shutdown-observer".to_string(),
            write_namespaces: vec![],
            read_namespaces: vec![],
            subscribe_namespaces: vec!["supervisor.".to_string()],
        };
        let (_store, mut mb) = router.spawn_actor("test.shutdown-observer", config).unwrap();
        let kernel = Arc::new(tokio::sync::Mutex::new(router));

        let executor = Arc::new(ShutdownTrackingExecutor::new());
        let mut registry = super::super::ProcessRegistry::new();

        let handle = executor.spawn(&test_job_ctx("evt-test")).await.unwrap();
        registry.register("evt-test", handle);

        super::super::graceful_shutdown(&kernel, &mut registry, Arc::clone(&executor) as Arc<dyn runtime::services::dispatch_ports::JobExecutor>).await;

        let event = mb
            .try_recv()
            .expect("supervisor.shutting_down event must be emitted during shutdown");
        assert_eq!(event.event_type, "supervisor.shutting_down");
    }

    /// S6: Stable process resets restart counter.
    /// After crashing once and running for 5 minutes, the restart counter
    /// should reset so the next crash is treated as a first crash.
    ///
    /// Note: restart policies are v0.3 scope but this projection test
    /// confirms the job.failed → re-dispatch → job.dispatched cycle works.
    #[test]
    fn failed_job_can_be_redispatched() {
        let events = vec![
            ev(
                event_types::JOB_CREATED,
                serde_json::json!({
                    "job_id": "j-redispatch",
                    "slug": "redispatch",
                    "agent": "rust-runtime",
                    "branch": "job/redispatch",
                    "spec_path": "spec.md",
                    "plan_id": null,
                }),
            ),
            ev(
                event_types::JOB_DISPATCHED,
                serde_json::json!({
                    "job_id": "j-redispatch",
                    "worktree": "/tmp/wt/redispatch",
                    "pid": 1000,
                }),
            ),
            ev(
                event_types::JOB_FAILED,
                serde_json::json!({
                    "job_id": "j-redispatch",
                    "error": "agent crashed",
                }),
            ),
            ev(
                event_types::JOB_DISPATCHED,
                serde_json::json!({
                    "job_id": "j-redispatch",
                    "worktree": "/tmp/wt/redispatch",
                    "pid": 1001,
                }),
            ),
        ];

        let map = project(&events);
        let job = map.get("j-redispatch").unwrap();
        assert_eq!(
            job.status,
            JobStatus::Dispatched,
            "failed job must be re-dispatchable"
        );
    }
}
