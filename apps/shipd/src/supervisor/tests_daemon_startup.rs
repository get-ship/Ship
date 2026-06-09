//! Daemon startup and reconciliation tests.
//!
//! When the daemon starts (or restarts), it must recover all in-flight work
//! from the event store. No job should be lost, stuck, or duplicated.
//! The startup sequence is: reconcile dispatched → replay completions → dispatch pending.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_pipeline_runtime::projections::pipeline_run::{project, PipelineRunStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let pipeline_run_id = payload["pipeline_run_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, pipeline_run_id, &payload).unwrap()
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
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "s",
            "agent": "r", "branch": "job/s",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "pipeline_run_id": "j1", "worktree": "/w", "pid": null,
        })),
        ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "s",
        })),
        // No merge or next dispatch — pipeline stalled while daemon was down
    ];
    let map = project(&events);
    let stale: Vec<_> = map.values()
        .filter(|j| j.status == PipelineRunStatus::Completed)
        .collect();
    assert_eq!(stale.len(), 1, "stale completed jobs must be detectable");
}

// ═══════════════════════════════════════════════════════════════════════════
// PENDING JOB DISPATCH
//
// After reconciliation and replay, scan for pending jobs whose dependencies
// are now resolved and dispatch them.
// ═══════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════
// LIVE PROCESS RE-ATTACHMENT
//
// When a dispatched job has a live process, register it in the process
// registry for graceful shutdown monitoring.
// ═══════════════════════════════════════════════════════════════════════════

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
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j-orphan", "slug": "orphan",
            "agent": "r", "branch": "job/orphan",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "pipeline_run_id": "j-orphan", "worktree": "/w", "pid": 99999,
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
        PipelineRunStatus::Dispatched,
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
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j-late", "slug": "late",
            "agent": "r", "branch": "job/late",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "pipeline_run_id": "j-late", "worktree": "/w", "pid": 99999,
        })),
        ev(ship_substrate::events::executor::EXECUTOR_SESSION_DESTROYED, serde_json::json!({
            "executor_kind": "tmux_window",
            "session_name": "late",
            "slug": "late",
            "cause": "external",
        })),
        // Eventually, liveness sweep emits job.failed after the stale
        // threshold. That transition still works.
        ev(event_types::PIPELINE_RUN_FAILED, serde_json::json!({
            "pipeline_run_id": "j-late",
            "error": "liveness: stale dispatched job — no process after 5m",
        })),
    ];
    let map = project(&events);
    let job = map.get("j-late").unwrap();
    assert_eq!(
        job.status,
        PipelineRunStatus::Failed,
        "liveness sweep still owns the eventual Failed transition"
    );
}

/// Completed jobs are not affected by reconciliation.
#[test]
fn reconcile_skips_completed_jobs() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j-done", "slug": "done",
            "agent": "r", "branch": "job/done",
            "spec_path": "spec.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "pipeline_run_id": "j-done", "worktree": "/w", "pid": 12345,
        })),
        ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
            "pipeline_run_id": "j-done", "slug": "done",
        })),
    ];
    let map = project(&events);
    let job = map.get("j-done").unwrap();
    // Completed jobs are NOT Dispatched, so reconcile_dispatched_inner skips them.
    assert_eq!(
        job.status,
        PipelineRunStatus::Completed,
        "completed jobs must not be touched by reconciliation"
    );
}

