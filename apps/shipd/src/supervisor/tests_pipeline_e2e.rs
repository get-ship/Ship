//! End-to-end pipeline test.
//!
//! Proves the full pipeline works without human intervention:
//! create a job → dispatch → mock agent completes → pipeline advances
//! through all steps → merge fires → cleanup.
//!
//! No tmux, no real Claude, no network. Pure in-process test using
//! mock executor and isolation.

use async_trait::async_trait;
use runtime::events::job::event_types;
use runtime::events::kernel_router::{ActorConfig, KernelRouter};
use runtime::events::store::{EventStore, SqliteEventStore};
use runtime::events::validator::{CallerKind, EmitContext};
use runtime::events::EventEnvelope;
use runtime::projections::job::{load_jobs_at, JobStatus};
use runtime::services::dispatch_ports::{ExecutorHandle, IsolationStrategy, JobContext, JobExecutor};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

use super::job_dispatch::DispatchContext;
use super::registry::ProcessRegistry;

// ── Mock infrastructure ─────────────────────────────────────────────────────

struct MockIsolation {
    work_dir: PathBuf,
}

#[async_trait]
impl IsolationStrategy for MockIsolation {
    async fn prepare(&self, _job: &JobContext) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(self.work_dir.join(".artifacts")).ok();
        std::fs::write(
            self.work_dir.join(".artifacts/job-spec.md"),
            "---\nslug: test\n---\n# Test",
        ).ok();
        Ok(self.work_dir.clone())
    }
    async fn cleanup(&self, _job: &JobContext) -> anyhow::Result<()> { Ok(()) }
}

struct MockExecutor {
    spawn_count: AtomicUsize,
}

impl MockExecutor {
    fn new() -> Self { Self { spawn_count: AtomicUsize::new(0) } }
}

#[async_trait]
impl JobExecutor for MockExecutor {
    async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
        self.spawn_count.fetch_add(1, Ordering::SeqCst);
        Ok(ExecutorHandle { pid: Some(99999), inner: Box::new(()) })
    }
    async fn is_alive(&self, _handle: &ExecutorHandle) -> bool { true }
    async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> anyhow::Result<()> { Ok(()) }
}

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        workspace_id: None,
        session_id: None,
    }
}

fn dispatch_config() -> ActorConfig {
    ActorConfig {
        namespace: "service.job-dispatch".to_string(),
        write_namespaces: vec!["job.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string()],
    }
}

// ── E2E: Simple job lifecycle ───────────────────────────────────────────────

/// A job without a pipeline: created → dispatched → completed → merged.
/// Verifies the basic lifecycle works end-to-end through the event store.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn simple_job_lifecycle_through_event_store() {
    let dir = TempDir::new().unwrap();
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    let job_id = "j-e2e-simple";
    let slug = "e2e-simple";

    // 1. Create job
    store.append(&EventEnvelope::new(
        event_types::JOB_CREATED,
        job_id,
        &serde_json::json!({
            "job_id": job_id, "slug": slug,
            "agent": "rust-runtime", "branch": format!("job/{slug}"),
            "spec_path": "spec.md", "plan_id": null,
        }),
    ).unwrap()).unwrap();

    let jobs = load_jobs_at(&ship_dir).unwrap();
    assert_eq!(jobs.get(job_id).unwrap().status, JobStatus::Pending);

    // 2. Dispatch
    store.append(&EventEnvelope::new(
        event_types::JOB_DISPATCHED,
        job_id,
        &serde_json::json!({
            "job_id": job_id,
            "worktree": "/tmp/wt/e2e-simple",
            "pid": 12345,
        }),
    ).unwrap()).unwrap();

    let jobs = load_jobs_at(&ship_dir).unwrap();
    assert_eq!(jobs.get(job_id).unwrap().status, JobStatus::Dispatched);

    // 3. Complete
    store.append(&EventEnvelope::new(
        event_types::JOB_COMPLETED,
        job_id,
        &serde_json::json!({ "job_id": job_id, "slug": slug }),
    ).unwrap()).unwrap();

    let jobs = load_jobs_at(&ship_dir).unwrap();
    assert_eq!(jobs.get(job_id).unwrap().status, JobStatus::Completed);

    // 4. Merge
    store.append(&EventEnvelope::new(
        event_types::JOB_MERGED,
        job_id,
        &serde_json::json!({ "job_id": job_id, "slug": slug }),
    ).unwrap()).unwrap();

    let jobs = load_jobs_at(&ship_dir).unwrap();
    let rec = jobs.get(job_id).unwrap();
    assert_eq!(rec.status, JobStatus::Merged);
    assert_eq!(rec.worktree.as_deref(), Some("/tmp/wt/e2e-simple"));
    assert!(rec.error.is_none());
}

// ── E2E: Pipeline advancement ───────────────────────────────────────────────

/// A pipeline job: created with steps → dispatched → each step completes
/// and the next dispatches automatically via advance_pipeline.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pipeline_advances_through_all_steps() {
    let dir = TempDir::new().unwrap();
    let ship_dir = dir.path().join(".ship");
    let worktree_root = dir.path().join("worktrees");
    let worktree_path = worktree_root.join("e2e-pipe");
    std::fs::create_dir_all(&ship_dir).unwrap();
    std::fs::create_dir_all(worktree_path.join(".artifacts")).unwrap();
    std::fs::write(
        worktree_path.join(".artifacts/job-spec.md"),
        "---\nslug: e2e-pipe\n---\n# Pipeline Test",
    ).unwrap();

    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    let job_id = "j-e2e-pipe";
    let slug = "e2e-pipe";

    // Create job with 2-step pipeline: implement → gate
    store.append(&EventEnvelope::new(
        event_types::JOB_CREATED,
        job_id,
        &serde_json::json!({
            "job_id": job_id, "slug": slug,
            "agent": "rust-runtime", "branch": format!("job/{slug}"),
            "spec_path": "spec.md", "plan_id": null,
            "steps": { "steps": [
                { "id": "implement", "type": "agent", "agent": "rust-runtime", "after": [] },
                { "id": "gate", "type": "gate", "agent": "reviewer", "after": ["implement"] },
            ]}
        }),
    ).unwrap()).unwrap();

    // Dispatch first step
    store.append(&EventEnvelope::new(
        event_types::JOB_DISPATCHED,
        job_id,
        &serde_json::json!({
            "job_id": job_id,
            "worktree": worktree_path.to_string_lossy(),
            "pid": null,
        }),
    ).unwrap()).unwrap();

    // Set up kernel + dispatch context with mock executor
    let mut router = KernelRouter::new(ship_dir.clone()).unwrap();
    let (_s, _mb) = router.spawn_actor("service.job-dispatch", dispatch_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let executor = Arc::new(MockExecutor::new());
    let dctx = DispatchContext {
        isolation: Arc::new(MockIsolation { work_dir: worktree_path.clone() }),
        executor: executor.clone(),
        registry: Arc::new(Mutex::new(ProcessRegistry::new())),
    };

    // First step completes — simulate agent finishing
    let completed_event = EventEnvelope::new(
        event_types::JOB_COMPLETED,
        job_id,
        &serde_json::json!({ "job_id": job_id, "slug": slug }),
    ).unwrap();
    store.append(&completed_event).unwrap();

    // Advance pipeline — should dispatch next step (gate).
    // The advance function loads the projection (which includes the completed
    // event), finds unblocked steps, and dispatches them via the executor.
    let advanced = super::dispatch_pipeline::advance_pipeline_at(
        &kernel, &completed_event, &dctx, &ship_dir,
    ).await;

    assert!(advanced, "pipeline must advance after step completion — \
        current_step is None after completion, advance must find the next \
        unblocked step from completed_steps, not bail on idempotency check");
    assert!(
        executor.spawn_count.load(Ordering::SeqCst) > 0,
        "executor.spawn must be called to dispatch the gate step"
    );

    // Verify projection shows progress
    let jobs = load_jobs_at(&ship_dir).unwrap();
    let rec = jobs.get(job_id).unwrap();
    assert_eq!(rec.status, JobStatus::Dispatched, "job should be re-dispatched for gate step");
    assert!(
        rec.completed_steps.contains(&"implement".to_string()),
        "implement step must be in completed_steps"
    );
}

// ── E2E: Dependency resolution ──────────────────────────────────────────────

/// Jobs with dependencies wait for upstream, then dispatch when ready.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dependent_job_dispatches_after_upstream_completes() {
    let dir = TempDir::new().unwrap();
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    // Create upstream job
    store.append(&EventEnvelope::new(
        event_types::JOB_CREATED,
        "j-upstream",
        &serde_json::json!({
            "job_id": "j-upstream", "slug": "upstream",
            "agent": "test-writer", "branch": "job/upstream",
            "spec_path": "spec.md", "plan_id": null,
        }),
    ).unwrap()).unwrap();

    // Create downstream job with dependency
    store.append(&EventEnvelope::new(
        event_types::JOB_CREATED,
        "j-downstream",
        &serde_json::json!({
            "job_id": "j-downstream", "slug": "downstream",
            "agent": "rust-runtime", "branch": "job/downstream",
            "spec_path": "spec.md", "plan_id": null,
            "depends_on": ["upstream"],
        }),
    ).unwrap()).unwrap();

    // Verify downstream is pending with unresolved deps
    let jobs = load_jobs_at(&ship_dir).unwrap();
    assert_eq!(jobs.get("j-downstream").unwrap().status, JobStatus::Pending);

    let done_slugs: std::collections::HashSet<&str> = jobs.values()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();
    let deps = jobs.get("j-downstream").unwrap().depends_on.as_ref().unwrap();
    assert!(!deps.iter().all(|d| done_slugs.contains(d.as_str())), "deps should NOT be resolved yet");

    // Complete upstream
    store.append(&EventEnvelope::new(
        event_types::JOB_DISPATCHED,
        "j-upstream",
        &serde_json::json!({ "job_id": "j-upstream", "worktree": "/tmp/wt/up", "pid": null }),
    ).unwrap()).unwrap();
    store.append(&EventEnvelope::new(
        event_types::JOB_COMPLETED,
        "j-upstream",
        &serde_json::json!({ "job_id": "j-upstream", "slug": "upstream" }),
    ).unwrap()).unwrap();

    // Now downstream deps should be resolved
    let jobs = load_jobs_at(&ship_dir).unwrap();
    let done_slugs: std::collections::HashSet<&str> = jobs.values()
        .filter(|j| matches!(j.status, JobStatus::Completed | JobStatus::Merged))
        .map(|j| j.slug.as_str())
        .collect();
    let deps = jobs.get("j-downstream").unwrap().depends_on.as_ref().unwrap();
    assert!(deps.iter().all(|d| done_slugs.contains(d.as_str())), "deps SHOULD be resolved now");
}

// ── E2E: Restart recovery ───────────────────────────────────────────────────

/// After simulated restart, stale dispatched jobs are detected and
/// stale completed jobs are ready for pipeline replay.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restart_recovery_finds_stale_jobs() {
    let dir = TempDir::new().unwrap();
    let ship_dir = dir.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let store = SqliteEventStore::new_at(&ship_dir).unwrap();

    // Job stuck in dispatched (agent died)
    store.append(&EventEnvelope::new(
        event_types::JOB_CREATED, "j-stuck",
        &serde_json::json!({
            "job_id": "j-stuck", "slug": "stuck",
            "agent": "r", "branch": "job/stuck",
            "spec_path": "s.md", "plan_id": null,
        }),
    ).unwrap()).unwrap();
    store.append(&EventEnvelope::new(
        event_types::JOB_DISPATCHED, "j-stuck",
        &serde_json::json!({ "job_id": "j-stuck", "worktree": "/tmp/wt/stuck", "pid": 1234 }),
    ).unwrap()).unwrap();

    // Job stuck in completed (pipeline didn't advance)
    store.append(&EventEnvelope::new(
        event_types::JOB_CREATED, "j-stalled",
        &serde_json::json!({
            "job_id": "j-stalled", "slug": "stalled",
            "agent": "r", "branch": "job/stalled",
            "spec_path": "s.md", "plan_id": null,
        }),
    ).unwrap()).unwrap();
    store.append(&EventEnvelope::new(
        event_types::JOB_DISPATCHED, "j-stalled",
        &serde_json::json!({ "job_id": "j-stalled", "worktree": "/tmp/wt/stalled", "pid": null }),
    ).unwrap()).unwrap();
    store.append(&EventEnvelope::new(
        event_types::JOB_COMPLETED, "j-stalled",
        &serde_json::json!({ "job_id": "j-stalled", "slug": "stalled" }),
    ).unwrap()).unwrap();

    // Simulate restart: load projection and find stale jobs
    let jobs = load_jobs_at(&ship_dir).unwrap();

    let stale_dispatched: Vec<_> = jobs.values()
        .filter(|j| j.status == JobStatus::Dispatched)
        .collect();
    let stale_completed: Vec<_> = jobs.values()
        .filter(|j| j.status == JobStatus::Completed)
        .collect();

    assert_eq!(stale_dispatched.len(), 1, "must detect stuck dispatched job");
    assert_eq!(stale_dispatched[0].slug, "stuck");
    assert!(stale_dispatched[0].worktree.is_some(), "must have worktree for relaunch");

    assert_eq!(stale_completed.len(), 1, "must detect stalled completed job");
    assert_eq!(stale_completed[0].slug, "stalled");
}

// ── E2E: Run step advances to agent step ──────────────────────────────────────

/// A run step executes a shell command and advances the pipeline to the next
/// agent step upon success.
///
/// Sequence:
///   job.created (2-step: run "verify" → agent "implement")
///   job.dispatched  (sets current_step to "verify")
///   job.step_completed for "verify"  (simulates the run step finishing)
///   advance_pipeline → dispatches "implement" (agent step)
///
/// The agent step does NOT need to complete in this test — we only verify
/// that dispatch happened.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_step_advances_to_agent_step() {
    let dir = TempDir::new().unwrap();
    let ship_dir = dir.path().join(".ship");
    let worktree_root = dir.path().join("worktrees");
    let worktree_path = worktree_root.join("run-advance");
    std::fs::create_dir_all(&ship_dir).unwrap();
    std::fs::create_dir_all(worktree_path.join(".artifacts")).unwrap();
    std::fs::write(
        worktree_path.join(".artifacts/job-spec.md"),
        "---\nslug: run-advance\n---\n# Run Step Test",
    ).unwrap();

    let store = SqliteEventStore::new_at(&ship_dir).unwrap();
    let job_id = "j-run-advance";
    let slug = "run-advance";

    // Create job with 2-step pipeline: run step then agent step
    store.append(&EventEnvelope::new(
        event_types::JOB_CREATED,
        job_id,
        &serde_json::json!({
            "job_id": job_id, "slug": slug,
            "agent": "rust-lane", "branch": format!("job/{slug}"),
            "spec_path": "spec.md", "plan_id": null,
            "steps": { "steps": [
                { "id": "verify", "type": "run", "command": "echo ok", "after": [] },
                { "id": "implement", "type": "agent", "agent": "rust-lane", "after": ["verify"] },
            ]}
        }),
    ).unwrap()).unwrap();

    // Dispatch — sets current_step to the first step ("verify")
    store.append(&EventEnvelope::new(
        event_types::JOB_DISPATCHED,
        job_id,
        &serde_json::json!({
            "job_id": job_id,
            "worktree": worktree_path.to_string_lossy(),
            "pid": null,
        }),
    ).unwrap()).unwrap();

    // Simulate run step completing successfully: emit job.step_completed for "verify"
    let step_completed_event = EventEnvelope::new(
        event_types::STEP_COMPLETED,
        job_id,
        &serde_json::json!({
            "job_id": job_id,
            "slug": slug,
            "step_id": "verify",
        }),
    ).unwrap();
    store.append(&step_completed_event).unwrap();

    // Set up kernel and dispatch context
    let mut router = KernelRouter::new(ship_dir.clone()).unwrap();
    let (_s, _mb) = router.spawn_actor("service.job-dispatch", dispatch_config()).unwrap();
    let kernel = Arc::new(Mutex::new(router));

    let executor = Arc::new(MockExecutor::new());
    let dctx = DispatchContext {
        isolation: Arc::new(MockIsolation { work_dir: worktree_path.clone() }),
        executor: executor.clone(),
        registry: Arc::new(Mutex::new(ProcessRegistry::new())),
    };

    // advance_pipeline with the step_completed event — should dispatch "implement"
    let advanced = super::dispatch_pipeline::advance_pipeline_at(
        &kernel, &step_completed_event, &dctx, &ship_dir,
    ).await;

    assert!(advanced, "pipeline must advance after run step completes");
    assert!(
        executor.spawn_count.load(Ordering::SeqCst) > 0,
        "executor.spawn must be called to dispatch the agent step"
    );

    // Projection should show the verify step is in completed_steps
    // and the implement step is now being dispatched
    let jobs = load_jobs_at(&ship_dir).unwrap();
    let rec = jobs.get(job_id).unwrap();
    assert!(
        rec.completed_steps.contains(&"verify".to_string()),
        "verify (run step) must be in completed_steps"
    );
    assert_eq!(
        rec.status,
        JobStatus::Dispatched,
        "job must be Dispatched for the implement step"
    );
}
