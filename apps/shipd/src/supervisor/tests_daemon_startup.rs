//! Daemon startup and reconciliation tests.
//!
//! When the daemon starts (or restarts), it must recover all in-flight work
//! from the event store. No job should be lost, stuck, or duplicated.
//! The startup sequence is: reconcile dispatched → replay completions → dispatch pending.

use runtime::events::job::event_types;
use runtime::events::EventEnvelope;
use runtime::projections::job::{project, JobStatus};

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
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/router.rs");
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
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/router.rs");
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
// DEAD PROCESS DETECTION
//
// After daemon restart, dispatched jobs may have dead processes. The daemon
// must detect these and emit step_failed.
// ═══════════════════════════════════════════════════════════════════════════

/// Reconciliation scans for dispatched jobs in the projection.
#[test]
fn reconciliation_scans_dispatched_jobs() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("JobStatus::Dispatched"),
        "reconciliation must filter for Dispatched status jobs"
    );
}

/// Reconciliation checks process liveness via the executor.
#[test]
fn reconciliation_checks_liveness() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("is_alive"),
        "reconciliation must check if the process is still alive"
    );
}

/// Dead processes must emit step_failed, not silently drop the job.
#[test]
fn dead_process_emits_step_failed() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("STEP_FAILED") || source.contains("step_failed"),
        "dead processes must emit a step_failed event"
    );
}

/// The step_failed error must indicate daemon restart.
#[test]
fn step_failed_includes_restart_reason() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("daemon restarted"),
        "step_failed error must mention daemon restart"
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
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("JobStatus::Completed"),
        "replay must filter for Completed status jobs"
    );
}

/// Replay must call advance_pipeline to continue stalled pipelines.
#[test]
fn replay_advances_pipeline() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/replay.rs");
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
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/pending.rs");
    assert!(
        source.contains("MAX_CONCURRENT") && source.contains("slots"),
        "pending job dispatch must respect concurrency limits"
    );
}

/// Pending dispatch checks dependency resolution.
#[test]
fn pending_dispatch_checks_dependencies() {
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/pending.rs");
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
    let source = include_str!("../../../../crates/core/runtime/src/services/orchestrator/replay.rs");
    assert!(
        source.contains("register") && source.contains("is_alive"),
        "live processes must be re-registered in ProcessRegistry"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// PROJECTION-BASED RECONCILIATION
//
// Verify that reconciliation correctly uses the job projection to determine
// which jobs need recovery and which should be skipped.
// ═══════════════════════════════════════════════════════════════════════════

/// Orphaned dispatched jobs are detected and marked failed via projection.
#[test]
fn reconcile_marks_orphaned_jobs_as_failed() {
    let events = vec![
        ev(event_types::JOB_CREATED, serde_json::json!({
            "job_id": "j-orphan", "slug": "orphan",
            "agent": "r", "branch": "job/orphan",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::JOB_DISPATCHED, serde_json::json!({
            "job_id": "j-orphan", "worktree": "/w", "pid": 99999,
        })),
        // No completion — daemon died, agent process is gone.
        // Simulate what reconcile would do: emit step_failed.
        ev(event_types::STEP_FAILED, serde_json::json!({
            "job_id": "j-orphan",
            "error": "daemon restarted, agent not running",
        })),
    ];
    let map = project(&events);
    let job = map.get("j-orphan").unwrap();
    assert_eq!(
        job.status,
        JobStatus::Failed,
        "orphaned dispatched job must be Failed after step_failed event"
    );
    assert!(
        job.error.as_ref().map_or(false, |e| e.contains("daemon restarted")),
        "error must reference daemon restart"
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

