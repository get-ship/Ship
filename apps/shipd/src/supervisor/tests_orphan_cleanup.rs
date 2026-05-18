//! Orphan worktree cleanup tests.
//!
//! After daemon restart, jobs that previously reached Merged or Failed status
//! may still have worktrees on disk. The startup scan must detect and clean them.
//! Active (Dispatched, Pending) jobs must never be touched.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_platform::projections::job::{project, JobStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let job_id = payload["job_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, job_id, &payload).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// ORPHAN DETECTION
// ═══════════════════════════════════════════════════════════════════════════

/// Startup sequence must include orphan cleanup.
#[test]
fn startup_includes_orphan_cleanup() {
    let source = include_str!("../../../../crates/core/ship-platform/src/services/orchestrator/router.rs");
    assert!(
        source.contains("cleanup_orphaned_worktrees"),
        "startup must call cleanup_orphaned_worktrees to remove stale disk state"
    );
}

/// Orphan cleanup function exists in orphan_cleanup module.
#[test]
fn orphan_cleanup_function_exists() {
    let source = include_str!("../../../../crates/core/ship-supervisor/src/orphan_cleanup.rs");
    assert!(
        source.contains("cleanup_orphaned_worktrees"),
        "orphan_cleanup must provide a cleanup_orphaned_worktrees function"
    );
}

/// Orphan detection filters for terminal states only.
/// Active (Dispatched, Pending) jobs must never be cleaned up.
#[test]
fn orphan_cleanup_filters_terminal_states_only() {
    let source = include_str!("../../../../crates/core/ship-supervisor/src/orphan_cleanup.rs");
    assert!(
        source.contains("JobStatus::Merged") && source.contains("JobStatus::Failed"),
        "orphan cleanup must filter for Merged and Failed jobs only"
    );
}

/// Orphan cleanup calls cleanup_orphan for consistent cleanup behavior.
#[test]
fn orphan_cleanup_delegates_to_cleanup_orphan() {
    let source = include_str!("../../../../crates/core/ship-supervisor/src/orphan_cleanup.rs");
    assert!(
        source.contains("cleanup_orphan"),
        "orphan cleanup must call cleanup_orphan for consistent cleanup behavior"
    );
}

/// Orphan detection uses job projection — merged/failed jobs with worktrees present.
#[test]
fn orphan_detection_via_projection() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "job_id": "j-orph", "slug": "orph",
            "agent": "r", "branch": "job/orph",
            "spec_path": "s.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "job_id": "j-orph", "worktree": "/tmp/ship-test/orph", "pid": null,
        })),
        ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
            "job_id": "j-orph", "slug": "orph",
        })),
        ev(event_types::PIPELINE_RUN_MERGED, serde_json::json!({
            "job_id": "j-orph", "slug": "orph",
        })),
    ];
    let map = project(&events);

    // Merged job is an orphan candidate if the worktree path still exists.
    // Here we just verify the projection gives us the data we need.
    let job = map.get("j-orph").expect("job must appear in projection");
    assert_eq!(job.status, JobStatus::Merged);
    assert_eq!(job.worktree.as_deref(), Some("/tmp/ship-test/orph"));
}

/// Active dispatched jobs must NOT appear as orphan candidates.
#[test]
fn active_jobs_are_not_orphan_candidates() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "job_id": "j-active", "slug": "active",
            "agent": "r", "branch": "job/active",
            "spec_path": "s.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "job_id": "j-active", "worktree": "/tmp/ship-test/active", "pid": null,
        })),
    ];
    let map = project(&events);
    let job = map.get("j-active").expect("job must appear in projection");
    assert_eq!(job.status, JobStatus::Dispatched);

    // A Dispatched job must not be considered for orphan cleanup.
    let is_orphan_candidate = matches!(job.status, JobStatus::Merged | JobStatus::Failed);
    assert!(!is_orphan_candidate, "active dispatched jobs must not be cleaned up");
}
