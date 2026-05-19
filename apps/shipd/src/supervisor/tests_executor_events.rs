//! Regression tests for the executor-events slice.
//!
//! The original bug: when a tmux window vanished (operator kill, substrate
//! failure), the executor sent `ExitSignal { exit_code: None }` and the
//! liveness handler emitted `job.completed`. These tests pin the fix —
//! `ExitCause` now forces the call site to declare semantics, and the
//! handler branches accordingly. Without these, the same bug can creep back.

use ship_substrate::db::events::list_recent_events_at;
use ship_substrate::events::kernel_router::KernelRouter;
use ship_substrate::events::EventEnvelope;
use ship_platform::services::dispatch_ports::{ExitCause, ExitSignal};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_platform::services::orchestrator::exit_handler;

use ship_platform::services::process_registry::{ProcessRegistry, SharedRegistry};

fn setup() -> (TempDir, std::path::PathBuf, Arc<Mutex<KernelRouter>>) {
    let dir = TempDir::new().unwrap();
    let base = ship_substrate::ShipHome::project_ship_dir(dir.path());
    let router = KernelRouter::new(base.clone()).unwrap();
    (dir, base, Arc::new(Mutex::new(router)))
}

fn types_of(events: &[EventEnvelope]) -> Vec<String> {
    events.iter().map(|e| e.event_type.clone()).collect()
}

/// Send a single ExitSignal and wait until the handler's expected
/// terminal event lands in the persisted log (or `terminal_type` is
/// `None`, in which case we wait the minimum interval).
///
/// Pre-hardening this used a flat `sleep(150ms)`. Under workspace-test
/// parallelism the two kernel emits sometimes took longer than 150ms
/// to land on disk, and the assertion ran against an incomplete log
/// — manifesting as "got [\"process.exited\"]" with no job event. The
/// fix mirrors what we did for the bun-side actor-projection race:
/// poll for the condition you're going to assert on rather than
/// guessing a sleep duration.
async fn drive_single_signal_until(
    base: &std::path::Path,
    kernel: Arc<Mutex<KernelRouter>>,
    signal: ExitSignal,
    terminal_type: Option<&str>,
) {
    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
    let port: Arc<dyn ship_platform::services::dispatch_ports::ProcessRegistryPort> =
        Arc::new(SharedRegistry(registry));
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExitSignal>();
    exit_handler::start(kernel, port, rx);
    tx.send(signal).expect("channel accepts signal");

    // 3s is generous; the handler does at most two kernel emits + a
    // disk write. Real workload completes in <50ms; the budget exists
    // for adversarial parallel scheduling.
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        let events = list_recent_events_at(base, 50).unwrap_or_default();
        match terminal_type {
            Some(t) => {
                if events.iter().any(|e| e.event_type == t) {
                    return;
                }
            }
            None => {
                // No terminal sentinel — wait long enough for the
                // handler to finish its emits, then return.
                tokio::time::sleep(Duration::from_millis(150)).await;
                return;
            }
        }
        if std::time::Instant::now() >= deadline {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

/// Back-compat shim for the few callers that don't care about a
/// specific terminal event (e.g. the orphan-emit and cleanup tests
/// that observe side effects via tmux, not the event log).
async fn drive_single_signal(
    kernel: Arc<Mutex<KernelRouter>>,
    signal: ExitSignal,
) {
    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
    let port: Arc<dyn ship_platform::services::dispatch_ports::ProcessRegistryPort> =
        Arc::new(SharedRegistry(registry));
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExitSignal>();
    exit_handler::start(kernel, port, rx);
    tx.send(signal).expect("channel accepts signal");
    tokio::time::sleep(Duration::from_millis(150)).await;
}

// ── The bug-fix test ─────────────────────────────────────────────────────────

/// Operator-killed tmux windows must surface as `job.failed` with an
/// `executor.session_destroyed` event preceding it. Pre-slice this emitted
/// `job.completed` — the exact regression this module prevents.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn session_destroyed_emits_job_failed_not_completed() {
    let (_dir, base, kernel) = setup();

    drive_single_signal_until(
        &base,
        Arc::clone(&kernel),
        ExitSignal {
            slug: "op-killed".to_string(),
            job_id: "j-op-1".to_string(),
            exit_code: None,
            cause: ExitCause::SessionDestroyed,
        },
        Some("pipeline.run.failed"),
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "executor.session_destroyed"),
        "must emit executor.session_destroyed; got {types:?}"
    );
    assert!(
        types.iter().any(|t| t == "pipeline.run.failed"),
        "must emit job.failed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "pipeline.run.completed"),
        "must NOT emit job.completed on session destruction; got {types:?}"
    );

    let destroyed = events
        .iter()
        .find(|e| e.event_type == "executor.session_destroyed")
        .expect("destroyed event present");
    let payload: serde_json::Value =
        serde_json::from_str(&destroyed.payload_json).expect("payload parses");
    assert_eq!(payload["cause"], "external");
    assert_eq!(payload["executor_kind"], "tmux_window");
    assert_eq!(payload["slug"], "op-killed");
}

// ── Agent-exited paths ───────────────────────────────────────────────────────

/// Clean agent exit (`Some(0) + AgentExited`) still emits `job.completed`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_exited_clean_still_emits_job_completed() {
    let (_dir, base, kernel) = setup();

    drive_single_signal_until(
        &base,
        Arc::clone(&kernel),
        ExitSignal {
            slug: "ok-job".to_string(),
            job_id: "j-ok-1".to_string(),
            exit_code: Some(0),
            cause: ExitCause::AgentExited,
        },
        Some("pipeline.run.completed"),
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "pipeline.run.completed"),
        "clean exit must emit job.completed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "pipeline.run.failed"),
        "clean exit must not emit job.failed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "executor.session_destroyed"),
        "clean exit must not emit executor.session_destroyed; got {types:?}"
    );
}

/// Non-zero exit (`Some(1) + AgentExited`) emits `job.failed`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_exited_nonzero_emits_job_failed() {
    let (_dir, base, kernel) = setup();

    drive_single_signal_until(
        &base,
        Arc::clone(&kernel),
        ExitSignal {
            slug: "err-job".to_string(),
            job_id: "j-err-1".to_string(),
            exit_code: Some(1),
            cause: ExitCause::AgentExited,
        },
        Some("pipeline.run.failed"),
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "pipeline.run.failed"),
        "nonzero exit must emit job.failed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "pipeline.run.completed"),
        "nonzero exit must not emit job.completed; got {types:?}"
    );
}

/// Unknown exit code (`None + AgentExited`) now emits `job.failed`. This is
/// a behavior change from the pre-slice "treat None as success" bug.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_exited_unknown_code_emits_job_failed() {
    let (_dir, base, kernel) = setup();

    drive_single_signal_until(
        &base,
        Arc::clone(&kernel),
        ExitSignal {
            slug: "unknown-job".to_string(),
            job_id: "j-unk-1".to_string(),
            exit_code: None,
            cause: ExitCause::AgentExited,
        },
        Some("pipeline.run.failed"),
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "pipeline.run.failed"),
        "agent gone with no code captured must emit job.failed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "pipeline.run.completed"),
        "agent gone with no code captured must NOT emit job.completed; got {types:?}"
    );
}

// ── cleanup-path destruction emission ───────────────────────────────────────
//
// Every tmux session we tear down must surface as a
// `executor.session_destroyed { External }` event — not only the cases where
// the liveness handler sees the substrate vanish. These tests pin the two
// cleanup call sites the emit-coverage slice wires up: `ShipdPlatform::
// cleanup_job` (end-of-life for a live job) and `cleanup_orphan` (startup
// scan for stale worktrees). Both share the same projection shape so the
// post-mortem query "every session we destroyed in the last 24h" stays
// complete regardless of how the destruction was triggered.

use async_trait::async_trait;
use ship_substrate::events::executor::EXECUTOR_SESSION_DESTROYED;
use ship_platform::services::dispatch_ports::{
    AgentPlatform, ExecutorHandle, JobContext, JobExecutor,
};

/// Stub executor so ShipdPlatform can be constructed in tests without
/// spinning up the real RoutingExecutor (which probes for tmux).
struct NoopJobExecutor;

#[async_trait]
impl JobExecutor for NoopJobExecutor {
    async fn spawn(&self, _ctx: &JobContext) -> anyhow::Result<ExecutorHandle> {
        Ok(ExecutorHandle { pid: None, inner: Box::new(()) })
    }
    async fn is_alive(&self, _handle: &ExecutorHandle) -> bool { false }
    async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _handle: &ExecutorHandle, _msg: &str) -> anyhow::Result<()> { Ok(()) }
}

fn build_platform(kernel: Arc<Mutex<KernelRouter>>) -> ship_supervisor::platform::ShipdPlatform {
    ship_supervisor::platform::ShipdPlatform {
        registry: Arc::new(Mutex::new(ProcessRegistry::new())),
        executor: Arc::new(NoopJobExecutor),
        kernel,
    }
}

/// `AgentPlatform::cleanup_job` must emit `executor.session_destroyed` with
/// `External` cause regardless of whether the tmux session actually existed.
/// Covers `platform.rs:44`'s `kill_tmux_session` call site.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cleanup_job_emits_executor_session_destroyed() {
    let (_dir, base, kernel) = setup();
    let platform = build_platform(Arc::clone(&kernel));

    platform.cleanup_job("cleanup-emit", "job/cleanup-emit").await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let destroyed = events
        .iter()
        .find(|e| {
            e.event_type == EXECUTOR_SESSION_DESTROYED && e.entity_id == "cleanup-emit"
        })
        .expect("cleanup_job must emit executor.session_destroyed for the slug");
    let payload: serde_json::Value =
        serde_json::from_str(&destroyed.payload_json).expect("payload parses");
    assert_eq!(payload["cause"], "external");
    assert_eq!(payload["executor_kind"], "tmux");
    assert_eq!(payload["session_name"], "job-cleanup-emit");
}

/// `cleanup_orphan` must emit `executor.session_destroyed` with `External`
/// cause so the post-mortem projection sees orphan-path destructions too.
/// Covers `orphan_cleanup.rs:68`'s `kill_tmux_session` call site.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cleanup_orphan_emits_executor_session_destroyed() {
    let (_dir, base, kernel) = setup();

    ship_supervisor::orphan_cleanup::tests_only::cleanup_orphan_for_test("orphan-emit", &kernel).await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let destroyed = events
        .iter()
        .find(|e| {
            e.event_type == EXECUTOR_SESSION_DESTROYED && e.entity_id == "orphan-emit"
        })
        .expect("cleanup_orphan must emit executor.session_destroyed for the slug");
    let payload: serde_json::Value =
        serde_json::from_str(&destroyed.payload_json).expect("payload parses");
    assert_eq!(payload["cause"], "external");
    assert_eq!(payload["executor_kind"], "tmux");
    assert_eq!(payload["session_name"], "job-orphan-emit");
}
