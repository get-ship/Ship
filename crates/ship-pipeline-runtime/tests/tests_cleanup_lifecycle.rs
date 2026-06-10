//! Cleanup lifecycle tests.
//!
//! When a job reaches a terminal state (merged, failed), its resources must
//! be cleaned up: worktree removed, branch deleted, tmux session killed,
//! process registry entry cleared.
//!
//! Cleanup must be event-driven and idempotent. Running cleanup twice on
//! the same job must not error.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_pipeline_runtime::projections::pipeline_run::{project, PipelineRunStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let pipeline_run_id = payload["pipeline_run_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, pipeline_run_id, &payload).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// CLEANUP RESILIENCE
//
// Cleanup operations must be best-effort and non-fatal. A failed worktree
// removal must not prevent branch deletion or tmux cleanup.
// ═══════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════
// ORPHAN DETECTION
//
// After daemon restart, there may be worktrees on disk for jobs that are
// already merged or failed. These orphans must be detectable and cleanable.
// ═══════════════════════════════════════════════════════════════════════════

/// The projection must provide enough data to detect orphaned worktrees.
/// A worktree is orphaned if the job is in a terminal state but the
/// worktree path still exists.
#[test]
fn terminal_jobs_with_worktrees_are_orphan_candidates() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "done",
            "agent": "r", "branch": "job/done",
            "spec_path": "s.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "pipeline_run_id": "j1", "worktree": "/tmp/wt/done", "pid": null,
        })),
        ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "done",
        })),
        ev(event_types::PIPELINE_RUN_MERGED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "done",
        })),
    ];
    let map = project(&events);
    let orphan_candidates: Vec<_> = map.values()
        .filter(|j| matches!(j.status, PipelineRunStatus::Merged | PipelineRunStatus::Failed))
        .filter(|j| j.worktree.is_some())
        .collect();
    assert_eq!(orphan_candidates.len(), 1);
    assert_eq!(orphan_candidates[0].worktree.as_deref(), Some("/tmp/wt/done"));
}
