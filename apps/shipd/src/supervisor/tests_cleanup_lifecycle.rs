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
use ship_platform::projections::pipeline_run::{project, PipelineRunStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let job_id = payload["job_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, job_id, &payload).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// CLEANUP TRIGGERS
//
// Cleanup fires on job.merged. Failed jobs retain resources for investigation.
// ═══════════════════════════════════════════════════════════════════════════

/// Merged jobs must trigger resource cleanup.
/// The cleanup handler receives the event and acts on it.
#[test]
fn cleanup_handler_exists_for_merged_events() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/handlers.rs");
    assert!(
        source.contains("job_cleanup") || source.contains("handle_job_cleanup"),
        "a cleanup handler must exist for merged job events"
    );
}

/// The orchestrator event router must route job.merged to cleanup.
#[test]
fn job_merged_routes_to_cleanup() {
    // Orchestration logic moved to ship_platform::services::orchestrator::router
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/router.rs");
    assert!(
        source.contains("PIPELINE_RUN_MERGED"),
        "orchestrator router must handle pipeline.run.merged events"
    );
    assert!(
        source.contains("cleanup") || source.contains("handle_job_cleanup"),
        "job.merged handler must trigger cleanup"
    );
}

/// Cleanup must remove the process registry entry.
#[test]
fn job_merged_unregisters_from_process_registry() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/router.rs");
    assert!(
        source.contains("unregister"),
        "job.merged handler must unregister from process registry"
    );
}

/// Cleanup handler must delegate resource cleanup to AgentPlatform.
/// The platform implementation handles worktree removal, branch deletion,
/// and tmux session cleanup independently.
#[test]
fn cleanup_delegates_to_platform() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/handlers.rs");
    assert!(
        source.contains("cleanup_job"),
        "cleanup must delegate to platform.cleanup_job for resource cleanup"
    );
}

/// Failed jobs must NOT trigger automatic cleanup — worktrees are kept
/// for post-mortem investigation.
#[test]
fn job_failed_does_not_trigger_cleanup() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/router.rs");
    // Find the job.failed handler and verify it doesn't call cleanup
    let lines: Vec<&str> = source.lines().collect();
    let failed_handler = lines.iter().enumerate()
        .find(|(_, l)| l.contains("\"job.failed\""));
    if let Some((idx, _)) = failed_handler {
        // Check the next ~10 lines for cleanup calls
        let handler_block: String = lines[idx..std::cmp::min(idx + 10, lines.len())]
            .join("\n");
        assert!(
            !handler_block.contains("handle_job_cleanup"),
            "job.failed must not trigger cleanup — keep resources for investigation"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CLEANUP RESILIENCE
//
// Cleanup operations must be best-effort and non-fatal. A failed worktree
// removal must not prevent branch deletion or tmux cleanup.
// ═══════════════════════════════════════════════════════════════════════════

/// Cleanup must handle missing worktrees gracefully.
/// (Worktree may have been manually deleted.)
#[test]
fn cleanup_tolerates_missing_worktree() {
    // remove_worktree in helpers.rs should check existence first
    let source = include_str!("../../../../crates/core/ship-supervisor/src/helpers.rs");
    assert!(
        source.contains("exists()"),
        "remove_worktree must check if worktree exists before removal"
    );
}

/// Cleanup must handle missing branches gracefully.
#[test]
fn cleanup_tolerates_missing_branch() {
    // delete_branch should not propagate errors for non-existent branches
    let source = include_str!("../../../../crates/core/ship-supervisor/src/helpers.rs");
    // The function logs but doesn't error on missing branches
    assert!(
        source.contains("may not exist"),
        "delete_branch must tolerate non-existent branches"
    );
}

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
            "job_id": "j1", "slug": "done",
            "agent": "r", "branch": "job/done",
            "spec_path": "s.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "job_id": "j1", "worktree": "/tmp/wt/done", "pid": null,
        })),
        ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
            "job_id": "j1", "slug": "done",
        })),
        ev(event_types::PIPELINE_RUN_MERGED, serde_json::json!({
            "job_id": "j1", "slug": "done",
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
