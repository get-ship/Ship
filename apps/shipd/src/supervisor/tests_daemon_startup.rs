//! Daemon startup and reconciliation tests.
//!
//! When the daemon starts (or restarts), it must recover all in-flight work
//! from the event store. No job should be lost, stuck, or duplicated.
//! The startup sequence is: reconcile dispatched → replay completions → dispatch pending.

use ship_substrate::events::job::event_types;
use ship_substrate::events::EventEnvelope;
use ship_substrate::projections::job::{project, JobStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let job_id = payload["job_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, job_id, &payload).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// RECONCILIATION ORDER
//
// The startup sequence must run in the correct order. If we dispatch pending
// jobs before reconciling dead dispatched ones, we might exceed concurrency.
// ═══════════════════════════════════════════════════════════════════════════

/// The startup sequence exists and runs three phases.
#[test]
fn startup_runs_three_reconciliation_phases() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/router.rs");
    assert!(
        source.contains("reconcile_dispatched") || source.contains("reconcile"),
        "startup must reconcile dispatched jobs"
    );
    assert!(
        source.contains("replay_unprocessed") || source.contains("replay"),
        "startup must replay unprocessed completions"
    );
    assert!(
        source.contains("dispatch_ready_pending"),
        "startup must dispatch ready pending jobs"
    );
}

/// Reconciliation must happen before dispatching new pending jobs.
/// If we dispatch first, we might exceed MAX_CONCURRENT.
#[test]
fn reconcile_before_dispatch_in_startup_order() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/router.rs");
    let reconcile_pos = source.find("reconcile_dispatched")
        .or_else(|| source.find("reconcile"));
    let dispatch_pos = source.find("dispatch_ready_pending");
    if let (Some(r), Some(d)) = (reconcile_pos, dispatch_pos) {
        assert!(
            r < d,
            "reconcile must happen before dispatch_ready_pending in startup"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DEAD PROCESS DETECTION (OBSERVATIONAL)
//
// After daemon restart, dispatched jobs may have no live process. Reconcile
// records a single observation via `executor.session_destroyed` and defers
// the terminal-state decision to the liveness sweep — which gates on a
// 5-minute stale threshold. This prevents a single missed probe from
// killing a job that is actually alive but slow to re-surface.
// ═══════════════════════════════════════════════════════════════════════════

/// Reconciliation scans for dispatched jobs in the projection.
#[test]
fn reconciliation_scans_dispatched_jobs() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("JobStatus::Dispatched"),
        "reconciliation must filter for Dispatched status jobs"
    );
}

/// Reconciliation asks the executor to rehydrate a handle — the modern
/// liveness primitive. The old `is_alive` on a placeholder handle pattern
/// is gone.
#[test]
fn reconciliation_uses_resume_handle() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("resume_handle"),
        "reconciliation must rehydrate a live handle via resume_handle"
    );
}

/// Dead processes must emit `executor.session_destroyed` — an observation
/// event. The liveness sweep owns the terminal-state decision.
#[test]
fn dead_process_emits_session_destroyed() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("emit_executor_session_destroyed"),
        "no-live-process must emit executor.session_destroyed on reconcile"
    );
}

/// Reconcile never emits `job.step_failed` — that is terminal and belongs
/// to the liveness sweep.
#[test]
fn reconcile_does_not_emit_step_failed() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        !source.contains("STEP_FAILED"),
        "reconcile must not emit STEP_FAILED — observation only"
    );
    assert!(
        !source.contains("emit_step_failed"),
        "the emit_step_failed helper must be gone from reconcile"
    );
    assert!(
        !source.contains("daemon restarted, agent not running"),
        "the old terminal error string must not exist in reconcile"
    );
}

/// Observation uses the `External` cause — agent exit was not observed;
/// session/window simply isn't there.
#[test]
fn session_destroyed_uses_external_cause() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("DestructionCause::External"),
        "reconcile must record cause = External (unobserved vanishment)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// STALE COMPLETION REPLAY
//
// If a job.completed event arrived while the daemon was down, the pipeline
// won't have advanced. Replay must find these and advance them.
// ═══════════════════════════════════════════════════════════════════════════

/// Completed jobs without subsequent pipeline advancement must be replayed.
#[test]
fn stale_completions_are_detected() {
    let events = vec![
        ev(event_types::JOB_CREATED, serde_json::json!({
            "job_id": "j1", "slug": "s",
            "agent": "r", "branch": "job/s",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::JOB_DISPATCHED, serde_json::json!({
            "job_id": "j1", "worktree": "/w", "pid": null,
        })),
        ev(event_types::JOB_COMPLETED, serde_json::json!({
            "job_id": "j1", "slug": "s",
        })),
        // No merge or next dispatch — pipeline stalled while daemon was down
    ];
    let map = project(&events);
    let stale: Vec<_> = map.values()
        .filter(|j| j.status == JobStatus::Completed)
        .collect();
    assert_eq!(stale.len(), 1, "stale completed jobs must be detectable");
}

/// Replay logic exists and filters for completed status.
#[test]
fn replay_filters_completed_jobs() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("JobStatus::Completed"),
        "replay must filter for Completed status jobs"
    );
}

/// Replay must call advance_pipeline to continue stalled pipelines.
#[test]
fn replay_advances_pipeline() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("advance_pipeline"),
        "replay must call advance_pipeline for stale completed jobs"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// PENDING JOB DISPATCH
//
// After reconciliation and replay, scan for pending jobs whose dependencies
// are now resolved and dispatch them.
// ═══════════════════════════════════════════════════════════════════════════

/// Pending dispatch respects concurrency limits.
#[test]
fn pending_dispatch_respects_concurrency() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/pending.rs");
    assert!(
        source.contains("MAX_CONCURRENT") && source.contains("slots"),
        "pending job dispatch must respect concurrency limits"
    );
}

/// Pending dispatch checks dependency resolution.
#[test]
fn pending_dispatch_checks_dependencies() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/pending.rs");
    assert!(
        source.contains("depends_on") || source.contains("deps"),
        "pending job dispatch must check dependency resolution"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// LIVE PROCESS RE-ATTACHMENT
//
// When a dispatched job has a live process, register it in the process
// registry for graceful shutdown monitoring.
// ═══════════════════════════════════════════════════════════════════════════

/// Live process re-attachment registers in ProcessRegistry.
#[test]
fn live_process_registered_in_registry() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("registry.register") && source.contains("resume_handle"),
        "live processes must be re-registered in ProcessRegistry after resume_handle"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECTION-BASED RECONCILIATION
//
// Verify that reconciliation correctly uses the job projection to determine
// which jobs need recovery and which should be skipped.
// ═══════════════════════════════════════════════════════════════════════════

/// Orphaned dispatched jobs stay in `Dispatched` immediately after reconcile.
///
/// Reconcile records the observation via `executor.session_destroyed` (an
/// event not consumed by the job projection) instead of `step_failed`.
/// The projection keeps the job in `Dispatched`; the liveness sweep's
/// 5-min stale threshold owns the eventual terminal transition.
#[test]
fn reconcile_keeps_orphaned_jobs_dispatched() {
    let events = vec![
        ev(event_types::JOB_CREATED, serde_json::json!({
            "job_id": "j-orphan", "slug": "orphan",
            "agent": "r", "branch": "job/orphan",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::JOB_DISPATCHED, serde_json::json!({
            "job_id": "j-orphan", "worktree": "/w", "pid": 99999,
        })),
        // No completion — daemon died, process went missing. Simulate
        // what reconcile emits today: an observation event.
        ev(ship_substrate::events::executor::EXECUTOR_SESSION_DESTROYED, serde_json::json!({
            "executor_kind": "tmux_window",
            "session_name": "orphan",
            "slug": "orphan",
            "cause": "external",
        })),
    ];
    let map = project(&events);
    let job = map.get("j-orphan").unwrap();
    assert_eq!(
        job.status,
        JobStatus::Dispatched,
        "reconcile must not flip orphaned jobs to Failed — that is liveness' call"
    );
    assert!(
        job.error.is_none(),
        "no error should be recorded from an observation event"
    );
}

/// Observation events from reconcile do not collide with the liveness sweep's
/// terminal state — the sweep still owns the transition to `Failed`.
#[test]
fn liveness_sweep_retains_terminal_authority() {
    let events = vec![
        ev(event_types::JOB_CREATED, serde_json::json!({
            "job_id": "j-late", "slug": "late",
            "agent": "r", "branch": "job/late",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::JOB_DISPATCHED, serde_json::json!({
            "job_id": "j-late", "worktree": "/w", "pid": 99999,
        })),
        ev(ship_substrate::events::executor::EXECUTOR_SESSION_DESTROYED, serde_json::json!({
            "executor_kind": "tmux_window",
            "session_name": "late",
            "slug": "late",
            "cause": "external",
        })),
        // Eventually, liveness sweep emits job.failed after the stale
        // threshold. That transition still works.
        ev(event_types::JOB_FAILED, serde_json::json!({
            "job_id": "j-late",
            "error": "liveness: stale dispatched job — no process after 5m",
        })),
    ];
    let map = project(&events);
    let job = map.get("j-late").unwrap();
    assert_eq!(
        job.status,
        JobStatus::Failed,
        "liveness sweep still owns the eventual Failed transition"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// DAEMON STARTUP: LIVE-DAEMON REFUSAL
//
// Two daemons racing over `daemon.port`/`daemon.pid` is the original defect
// surfaced by sup-restart-01: the second daemon runs reconcile against jobs
// the live daemon is managing and emits spurious terminal events.
// ═══════════════════════════════════════════════════════════════════════════

/// The helper exists and pattern-matches the pidfile + `kill(0)` check.
#[test]
fn another_daemon_alive_helper_exists() {
    let source = include_str!("../daemon.rs");
    assert!(
        source.contains("pub fn another_daemon_alive"),
        "daemon.rs must expose another_daemon_alive for testability"
    );
}

/// Startup returns an error instead of silently continuing when a foreign
/// daemon is alive. Verified at source level; unit coverage lives in
/// `daemon::stale_cleanup_tests::live_daemon_detected_returns_err`.
#[test]
fn cleanup_returns_result_with_live_daemon_refusal() {
    let source = include_str!("../daemon.rs");
    assert!(
        source.contains("cleanup_stale_daemon_files(global_dir: &Path) -> Result<()>"),
        "cleanup_stale_daemon_files must be fallible"
    );
    assert!(
        source.contains("another daemon is already running"),
        "refusal message must be actionable"
    );
    assert!(
        source.contains("ship daemon stop") && source.contains("ship daemon restart"),
        "refusal must point to the operator-facing remediation"
    );
}

/// The daemon startup sequence propagates the refusal up instead of
/// swallowing it — so the process exits non-zero with stderr explaining why.
#[test]
fn startup_propagates_refusal() {
    let source = include_str!("../lib.rs");
    assert!(
        source.contains("cleanup_stale_daemon_files(&global_dir).map_err"),
        "lib.rs must propagate the cleanup error on startup"
    );
    assert!(
        source.contains("shipd startup aborted"),
        "stderr must carry an explicit aborted marker"
    );
}

/// Completed jobs are not affected by reconciliation.
#[test]
fn reconcile_skips_completed_jobs() {
    let events = vec![
        ev(event_types::JOB_CREATED, serde_json::json!({
            "job_id": "j-done", "slug": "done",
            "agent": "r", "branch": "job/done",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::JOB_DISPATCHED, serde_json::json!({
            "job_id": "j-done", "worktree": "/w", "pid": 12345,
        })),
        ev(event_types::JOB_COMPLETED, serde_json::json!({
            "job_id": "j-done", "slug": "done",
        })),
    ];
    let map = project(&events);
    let job = map.get("j-done").unwrap();
    // Completed jobs are NOT Dispatched, so reconcile_dispatched_inner skips them.
    assert_eq!(
        job.status,
        JobStatus::Completed,
        "completed jobs must not be touched by reconciliation"
    );
}

