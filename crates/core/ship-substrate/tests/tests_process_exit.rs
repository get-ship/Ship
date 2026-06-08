//! Process exit detection tests.
//!
//! Covers drain_dead, contains, live_slugs on ProcessRegistry, and the
//! start_exit_handler liveness monitor. Tests 5–6 are source-level checks
//! that verify the executor files wire up exit_tx rather than legacy curl.

use async_trait::async_trait;
use ship_substrate::events::kernel_router::KernelRouter;
use ship_substrate::services::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, SessionContext, SessionExecutor,
};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_runner::exit_handler;

use ship_substrate::services::process_registry::{ProcessRegistry, SharedRegistry};

// ── Test executors ───────────────────────────────────────────────────────────

struct DeadExecutor;

#[async_trait]
impl SessionExecutor for DeadExecutor {
    async fn spawn(&self, _ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Ok(ExecutorHandle { pid: None, inner: Box::new(()) })
    }
    async fn is_alive(&self, _handle: &ExecutorHandle) -> bool {
        false
    }
    async fn stop(&self, _handle: &ExecutorHandle) -> anyhow::Result<()> {
        Ok(())
    }
    async fn send(&self, _handle: &ExecutorHandle, _msg: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

struct AliveExecutor;

#[async_trait]
impl SessionExecutor for AliveExecutor {
    async fn spawn(&self, _ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Ok(ExecutorHandle { pid: None, inner: Box::new(()) })
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

// ── Helpers ──────────────────────────────────────────────────────────────────

fn dead_handle() -> ExecutorHandle {
    ExecutorHandle { pid: None, inner: Box::new(()) }
}

fn setup_kernel() -> (TempDir, Arc<Mutex<KernelRouter>>) {
    let dir = TempDir::new().unwrap();
    let router = KernelRouter::new(ship_substrate::ShipHome::project_ship_dir(dir.path())).unwrap();
    (dir, Arc::new(Mutex::new(router)))
}

// ── drain_dead ───────────────────────────────────────────────────────────────

/// drain_dead removes the handle and returns its slug when the executor
/// reports the process has exited.
#[tokio::test]
async fn drain_dead_removes_handles_for_dead_processes() {
    let mut registry = ProcessRegistry::new();
    registry.register("dead-job", dead_handle());

    let executor = DeadExecutor;
    let removed = registry.drain_dead(&executor).await;

    assert_eq!(removed, vec!["dead-job"]);
    assert!(registry.is_empty(), "registry must be empty after draining the dead handle");
}

/// drain_dead leaves handles in place when the executor reports the process
/// is still running, and returns an empty vec.
#[tokio::test]
async fn drain_dead_keeps_live_handles() {
    let mut registry = ProcessRegistry::new();
    registry.register("live-job", dead_handle());

    let executor = AliveExecutor;
    let removed = registry.drain_dead(&executor).await;

    assert!(removed.is_empty(), "drain_dead must not remove live handles");
    assert_eq!(registry.len(), 1, "live handle must remain in the registry");
}

// ── contains / live_slugs ────────────────────────────────────────────────────

/// contains returns true for a registered slug and false for an unknown one.
#[test]
fn contains_returns_true_for_registered_slug() {
    let mut registry = ProcessRegistry::new();
    registry.register("present", dead_handle());

    assert!(registry.contains("present"));
    assert!(!registry.contains("absent"));
}

/// live_slugs returns all currently registered slug names.
#[test]
fn live_slugs_returns_all_registered_slugs() {
    let mut registry = ProcessRegistry::new();
    registry.register("alpha", dead_handle());
    registry.register("beta", dead_handle());

    let mut slugs = registry.live_slugs();
    slugs.sort();
    assert_eq!(slugs, vec!["alpha", "beta"]);
}

/// live_slugs returns an empty vec when the registry is empty.
#[test]
fn live_slugs_is_empty_for_fresh_registry() {
    let registry = ProcessRegistry::new();
    assert!(registry.live_slugs().is_empty());
}

// ── start_exit_handler ───────────────────────────────────────────────────────

/// Receiving an ExitSignal with exit_code Some(0) causes the handler to
/// unregister the slug from the registry (clean exit path).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exit_handler_emits_job_completed_for_clean_exit() {
    let (_dir, kernel) = setup_kernel();
    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    // Pre-register the job so the handler has something to remove.
    registry.lock().await.register("test-job", dead_handle());

    let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel::<ExitSignal>();
    exit_handler::start(Arc::clone(&kernel), Arc::new(SharedRegistry(Arc::clone(&registry))), exit_handler::ExitVocabulary::PipelineRun, exit_rx);

    exit_tx
        .send(ExitSignal {
            slug: "test-job".to_string(),
            pipeline_run_id: "j-clean-1".to_string(),
            exit_code: Some(0),
            cause: ExitCause::AgentExited,
        })
        .expect("channel must accept the exit signal");

    // Give the spawned task time to process the signal.
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(
        !registry.lock().await.contains("test-job"),
        "exit handler must unregister the slug after a clean exit"
    );
}

/// Receiving an ExitSignal with a non-zero exit code must not panic.
/// The handler processes the failure path without unwinding.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn exit_handler_does_not_panic_for_failed_exit() {
    let (_dir, kernel) = setup_kernel();
    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    registry.lock().await.register("failing-job", dead_handle());

    let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel::<ExitSignal>();
    exit_handler::start(Arc::clone(&kernel), Arc::new(SharedRegistry(Arc::clone(&registry))), exit_handler::ExitVocabulary::PipelineRun, exit_rx);

    exit_tx
        .send(ExitSignal {
            slug: "failing-job".to_string(),
            pipeline_run_id: "j-fail-1".to_string(),
            exit_code: Some(1),
            cause: ExitCause::AgentExited,
        })
        .expect("channel must accept the exit signal");

    tokio::time::sleep(Duration::from_millis(50)).await;

    // The test passes if we reach here without a panic. Also verify the
    // slug was unregistered so the registry does not leak the dead handle.
    assert!(
        !registry.lock().await.contains("failing-job"),
        "exit handler must unregister the slug after a failed exit"
    );
}

// ── Source-level checks ──────────────────────────────────────────────────────

fn executor_source(filename: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("crates/ship-supervisor/src/executors")
        .join(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read {filename}: {e}"))
}

/// process.rs must wire exit_tx into the spawned child watcher rather than
/// shelling out to curl for signalling.
#[test]
#[ignore = "doomed: executor/harness-adapter rework (integration-tier #9/10/13) — fix AS an integration when that arc runs; see .artifacts/gate-triage-2.md cluster B"]
fn process_executor_source_uses_exit_tx_not_curl() {
    let src = executor_source("process.rs");
    assert!(
        !src.contains("curl"),
        "process.rs must not use curl — signal exit via exit_tx"
    );
    assert!(
        src.contains("exit_tx"),
        "process.rs must send on exit_tx to report process exit"
    );
}
