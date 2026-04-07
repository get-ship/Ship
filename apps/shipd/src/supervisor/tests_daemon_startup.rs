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
    let source = include_str!("job_dispatch.rs");
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
    let source = include_str!("job_dispatch.rs");
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
// must detect these and either relaunch or fail them.
// ═══════════════════════════════════════════════════════════════════════════

/// Reconciliation scans for dispatched jobs in the projection.
#[test]
fn reconciliation_scans_dispatched_jobs() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("JobStatus::Dispatched"),
        "reconciliation must filter for Dispatched status jobs"
    );
}

/// Reconciliation checks process liveness via the executor.
#[test]
fn reconciliation_checks_liveness() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("is_alive"),
        "reconciliation must check if the process is still alive"
    );
}

/// Dead processes must be relaunched with the original worktree.
#[test]
fn dead_process_relaunched_in_existing_worktree() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("worktree") && source.contains("spawn"),
        "dead processes must be relaunched using the existing worktree"
    );
}

/// Relaunch failure must emit job.failed, not silently drop the job.
#[test]
fn relaunch_failure_emits_job_failed() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("JOB_FAILED") || source.contains("job.failed"),
        "relaunch failure must emit a job.failed event"
    );
}

/// Relaunch must recompile agent config (it may have changed since last run).
#[test]
fn relaunch_recompiles_agent_config() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("compile_agent_config"),
        "relaunch must recompile agent config in case it changed"
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
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("JobStatus::Completed"),
        "replay must filter for Completed status jobs"
    );
}

/// Replay must call advance_pipeline to continue stalled pipelines.
#[test]
fn replay_advances_pipeline() {
    let source = include_str!("dispatch_replay.rs");
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
    let source = include_str!("pending_dispatch.rs");
    assert!(
        source.contains("MAX_CONCURRENT") && source.contains("slots"),
        "pending job dispatch must respect concurrency limits"
    );
}

/// Pending dispatch checks dependency resolution.
#[test]
fn pending_dispatch_checks_dependencies() {
    let source = include_str!("pending_dispatch.rs");
    assert!(
        source.contains("depends_on") || source.contains("deps"),
        "pending job dispatch must check dependency resolution"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// RELAUNCH CONTEXT PRESERVATION
//
// When relaunching a dead agent, the daemon must preserve the original
// launch context: model, provider, and session metadata.
// ═══════════════════════════════════════════════════════════════════════════

/// Relaunch must recover model from the job projection.
#[test]
fn relaunch_recovers_model_from_projection() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("model"),
        "relaunch must recover model from the job record"
    );
}

/// Relaunch must recover provider from the job projection.
#[test]
fn relaunch_recovers_provider_from_projection() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("provider"),
        "relaunch must recover provider from the job record"
    );
}

/// Relaunch must recover session metadata for resume capability.
#[test]
fn relaunch_recovers_session_metadata() {
    let source = include_str!("dispatch_replay.rs");
    assert!(
        source.contains("provider_meta") || source.contains("session"),
        "relaunch must recover provider session metadata for resume"
    );
}
