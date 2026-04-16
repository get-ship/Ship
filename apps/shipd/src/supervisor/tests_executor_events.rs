//! Regression tests for the executor-events slice.
//!
//! The original bug: when a tmux window vanished (operator kill, substrate
//! failure), the executor sent `ExitSignal { exit_code: None }` and the
//! liveness handler emitted `job.completed`. These tests pin the fix —
//! `ExitCause` now forces the call site to declare semantics, and the
//! handler branches accordingly. Without these, the same bug can creep back.

use runtime::db::events::list_recent_events_at;
use runtime::events::kernel_router::KernelRouter;
use runtime::events::EventEnvelope;
use runtime::services::dispatch_ports::{ExitCause, ExitSignal};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Mutex;

use super::liveness::start_exit_handler;
use super::registry::ProcessRegistry;

fn setup() -> (TempDir, std::path::PathBuf, Arc<Mutex<KernelRouter>>) {
    let dir = TempDir::new().unwrap();
    let base = dir.path().join(".ship");
    let router = KernelRouter::new(base.clone()).unwrap();
    (dir, base, Arc::new(Mutex::new(router)))
}

fn types_of(events: &[EventEnvelope]) -> Vec<String> {
    events.iter().map(|e| e.event_type.clone()).collect()
}

/// Send a single ExitSignal through the handler and wait for processing to
/// finish. 150ms is generous — the handler does at most two kernel emits.
async fn drive_single_signal(
    kernel: Arc<Mutex<KernelRouter>>,
    signal: ExitSignal,
) {
    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExitSignal>();
    start_exit_handler(kernel, registry, rx);
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

    drive_single_signal(
        Arc::clone(&kernel),
        ExitSignal {
            slug: "op-killed".to_string(),
            job_id: "j-op-1".to_string(),
            exit_code: None,
            cause: ExitCause::SessionDestroyed,
        },
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "executor.session_destroyed"),
        "must emit executor.session_destroyed; got {types:?}"
    );
    assert!(
        types.iter().any(|t| t == "job.failed"),
        "must emit job.failed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "job.completed"),
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

    drive_single_signal(
        Arc::clone(&kernel),
        ExitSignal {
            slug: "ok-job".to_string(),
            job_id: "j-ok-1".to_string(),
            exit_code: Some(0),
            cause: ExitCause::AgentExited,
        },
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "job.completed"),
        "clean exit must emit job.completed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "job.failed"),
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

    drive_single_signal(
        Arc::clone(&kernel),
        ExitSignal {
            slug: "err-job".to_string(),
            job_id: "j-err-1".to_string(),
            exit_code: Some(1),
            cause: ExitCause::AgentExited,
        },
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "job.failed"),
        "nonzero exit must emit job.failed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "job.completed"),
        "nonzero exit must not emit job.completed; got {types:?}"
    );
}

/// Unknown exit code (`None + AgentExited`) now emits `job.failed`. This is
/// a behavior change from the pre-slice "treat None as success" bug.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_exited_unknown_code_emits_job_failed() {
    let (_dir, base, kernel) = setup();

    drive_single_signal(
        Arc::clone(&kernel),
        ExitSignal {
            slug: "unknown-job".to_string(),
            job_id: "j-unk-1".to_string(),
            exit_code: None,
            cause: ExitCause::AgentExited,
        },
    )
    .await;

    let events = list_recent_events_at(&base, 50).unwrap();
    let types = types_of(&events);

    assert!(
        types.iter().any(|t| t == "job.failed"),
        "agent gone with no code captured must emit job.failed; got {types:?}"
    );
    assert!(
        !types.iter().any(|t| t == "job.completed"),
        "agent gone with no code captured must NOT emit job.completed; got {types:?}"
    );
}
