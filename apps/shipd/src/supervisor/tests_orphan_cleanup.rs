//! Orphan worktree cleanup tests.
//!
//! After daemon restart, jobs that previously reached Merged or Failed status
//! may still have worktrees on disk. The startup scan must detect and clean them.
//! Active (Dispatched, Pending) jobs must never be touched.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_pipeline_runtime::projections::pipeline_run::{project, PipelineRunStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let pipeline_run_id = payload["pipeline_run_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, pipeline_run_id, &payload).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════
// ORPHAN DETECTION
// ═══════════════════════════════════════════════════════════════════════════

/// Orphan detection uses job projection — merged/failed jobs with worktrees present.
#[test]
fn orphan_detection_via_projection() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j-orph", "slug": "orph",
            "agent": "r", "branch": "job/orph",
            "spec_path": "s.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "pipeline_run_id": "j-orph", "worktree": "/tmp/ship-test/orph", "pid": null,
        })),
        ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
            "pipeline_run_id": "j-orph", "slug": "orph",
        })),
        ev(event_types::PIPELINE_RUN_MERGED, serde_json::json!({
            "pipeline_run_id": "j-orph", "slug": "orph",
        })),
    ];
    let map = project(&events);

    // Merged job is an orphan candidate if the worktree path still exists.
    // Here we just verify the projection gives us the data we need.
    let job = map.get("j-orph").expect("job must appear in projection");
    assert_eq!(job.status, PipelineRunStatus::Merged);
    assert_eq!(job.worktree.as_deref(), Some("/tmp/ship-test/orph"));
}

/// Active dispatched jobs must NOT appear as orphan candidates.
#[test]
fn active_jobs_are_not_orphan_candidates() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j-active", "slug": "active",
            "agent": "r", "branch": "job/active",
            "spec_path": "s.md", "plan_id": null,
        })),
        ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
            "pipeline_run_id": "j-active", "worktree": "/tmp/ship-test/active", "pid": null,
        })),
    ];
    let map = project(&events);
    let job = map.get("j-active").expect("job must appear in projection");
    assert_eq!(job.status, PipelineRunStatus::Dispatched);

    // A Dispatched job must not be considered for orphan cleanup.
    let is_orphan_candidate = matches!(job.status, PipelineRunStatus::Merged | PipelineRunStatus::Failed);
    assert!(!is_orphan_candidate, "active dispatched jobs must not be cleaned up");
}
