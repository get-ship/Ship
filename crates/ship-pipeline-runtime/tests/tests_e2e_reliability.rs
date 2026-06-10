//! End-to-end reliability tests.
//!
//! These tests verify the system behaves correctly across failure modes:
//! daemon restarts, agent crashes, stuck jobs, resource leaks, and
//! concurrent operations. They define what "stable" means for v0.2.0.

use ship_substrate::events::pipeline::event_types;
use ship_substrate::events::EventEnvelope;
use ship_pipeline_runtime::projections::pipeline_run::{project, PipelineRunStatus};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    let pipeline_run_id = payload["pipeline_run_id"].as_str().unwrap_or("test");
    EventEnvelope::new(event_type, pipeline_run_id, &payload).unwrap()
}

fn created(pipeline_run_id: &str, slug: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug,
        "agent": "rust-runtime", "branch": format!("job/{slug}"),
        "spec_path": "spec.md", "plan_id": null,
    }))
}

fn dispatched(pipeline_run_id: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_DISPATCHED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "worktree": format!("/tmp/wt/{pipeline_run_id}"), "pid": 1234
    }))
}

fn completed(pipeline_run_id: &str, slug: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_COMPLETED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug
    }))
}

fn merged(pipeline_run_id: &str, slug: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_MERGED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "slug": slug
    }))
}

fn failed(pipeline_run_id: &str, error: &str) -> EventEnvelope {
    ev(event_types::PIPELINE_RUN_FAILED, serde_json::json!({
        "pipeline_run_id": pipeline_run_id, "error": error
    }))
}

// ═══════════════════════════════════════════════════════════════════════════
// STALE WORKTREE CLEANUP
//
// Merged and failed jobs must have their worktrees cleaned up.
// The cleanup handler must be reliable — not leave orphans.
// ═══════════════════════════════════════════════════════════════════════════

/// Orphan detection: after restart, any worktree for a terminal job
/// (merged/failed) should be flagged for cleanup.
#[test]
fn orphan_worktrees_detectable_from_projection() {
    let events = vec![
        created("j1", "merged-job"),
        dispatched("j1"),
        completed("j1", "merged-job"),
        merged("j1", "merged-job"),
        created("j2", "failed-job"),
        dispatched("j2"),
        failed("j2", "crash"),
        created("j3", "active-job"),
        dispatched("j3"),
    ];
    let map = project(&events);

    let orphans: Vec<_> = map.values()
        .filter(|j| matches!(j.status, PipelineRunStatus::Merged | PipelineRunStatus::Failed))
        .filter(|j| j.worktree.is_some())
        .map(|j| j.slug.as_str())
        .collect();

    let active: Vec<_> = map.values()
        .filter(|j| j.status == PipelineRunStatus::Dispatched)
        .filter(|j| j.worktree.is_some())
        .map(|j| j.slug.as_str())
        .collect();

    assert_eq!(orphans.len(), 2, "merged + failed jobs are orphan candidates");
    assert_eq!(active.len(), 1, "active job is NOT an orphan");
    assert!(orphans.contains(&"merged-job"));
    assert!(orphans.contains(&"failed-job"));
    assert!(active.contains(&"active-job"));
}

// ═══════════════════════════════════════════════════════════════════════════
// DAEMON RESTART RECOVERY (E2E-1)
//
// After restart, the daemon must recover all in-flight work from the
// event store. No job lost, no job duplicated.
// ═══════════════════════════════════════════════════════════════════════════

/// Jobs in every non-terminal state must be recoverable after restart.
#[test]
fn all_non_terminal_states_recoverable() {
    let events = vec![
        // Pending — waiting for dispatch
        created("j-pending", "pending"),

        // Dispatched — agent was running
        created("j-dispatched", "dispatched"),
        dispatched("j-dispatched"),

        // Completed — waiting for pipeline advancement
        created("j-completed", "completed"),
        dispatched("j-completed"),
        completed("j-completed", "completed"),

        // Blocked — needs human
        created("j-blocked", "blocked"),
        dispatched("j-blocked"),
        ev(event_types::PIPELINE_RUN_BLOCKED, serde_json::json!({
            "pipeline_run_id": "j-blocked", "blocker": "needs key", "needs_human": true
        })),
    ];
    let map = project(&events);

    assert_eq!(map.get("j-pending").unwrap().status, PipelineRunStatus::Pending);
    assert_eq!(map.get("j-dispatched").unwrap().status, PipelineRunStatus::Dispatched);
    assert_eq!(map.get("j-completed").unwrap().status, PipelineRunStatus::Completed);
    assert_eq!(map.get("j-blocked").unwrap().status, PipelineRunStatus::Blocked);

    // All have enough data for the reconciliation loop to act on
    assert!(map.get("j-dispatched").unwrap().worktree.is_some());
    assert!(map.get("j-completed").unwrap().worktree.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// JOB FULL LIFECYCLE (E2E-4)
//
// A job must flow from creation to merge without human intervention.
// Every state transition is an event. No state is lost.
// ═══════════════════════════════════════════════════════════════════════════

/// Full lifecycle: created → dispatched → completed → merged.
#[test]
fn job_full_lifecycle_all_events_present() {
    let events = vec![
        created("j1", "auth-fix"),
        dispatched("j1"),
        completed("j1", "auth-fix"),
        merged("j1", "auth-fix"),
    ];
    let map = project(&events);
    let rec = map.get("j1").unwrap();

    assert_eq!(rec.status, PipelineRunStatus::Merged);
    assert_eq!(rec.slug, "auth-fix");
    assert!(rec.worktree.is_some(), "worktree must be tracked through lifecycle");
    assert!(rec.error.is_none(), "successful job must have no error");
}

/// Full lifecycle with pipeline: created → [dispatch+complete per step] → merged.
#[test]
fn pipeline_full_lifecycle_all_steps_tracked() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "pipe",
            "agent": "test-writer", "branch": "job/pipe",
            "spec_path": "spec.md", "plan_id": null,
            "steps": {
                "auto_merge": true,
                "steps": [
                    { "id": "test", "type": "agent", "agent": "test-writer", "after": [] },
                    { "id": "impl", "type": "agent", "agent": "rust-runtime", "after": ["test"] },
                    { "id": "gate", "type": "review", "agent": "reviewer", "after": ["impl"] },
                ]
            }
        })),
        dispatched("j1"), completed("j1", "pipe"),  // test
        dispatched("j1"), completed("j1", "pipe"),  // impl
        dispatched("j1"), completed("j1", "pipe"),  // gate
        merged("j1", "pipe"),
    ];
    let map = project(&events);
    let rec = map.get("j1").unwrap();

    assert_eq!(rec.status, PipelineRunStatus::Merged);
    assert!(rec.completed_steps.contains(&"test".to_string()));
    assert!(rec.completed_steps.contains(&"impl".to_string()));
    assert!(rec.completed_steps.contains(&"gate".to_string()));
}

/// Failure mid-pipeline preserves progress for retry.
#[test]
fn pipeline_failure_preserves_completed_steps() {
    let events = vec![
        ev(event_types::PIPELINE_RUN_STARTED, serde_json::json!({
            "pipeline_run_id": "j1", "slug": "pipe",
            "agent": "test-writer", "branch": "job/pipe",
            "spec_path": "spec.md", "plan_id": null,
            "steps": { "steps": [
                { "id": "test", "type": "agent", "agent": "test-writer", "after": [] },
                { "id": "impl", "type": "agent", "agent": "rust-runtime", "after": ["test"] },
            ]}
        })),
        dispatched("j1"), completed("j1", "pipe"),  // test done
        dispatched("j1"),                             // impl started
        failed("j1", "agent crashed"),                // impl failed
    ];
    let map = project(&events);
    let rec = map.get("j1").unwrap();

    assert_eq!(rec.status, PipelineRunStatus::Failed);
    assert!(
        rec.completed_steps.contains(&"test".to_string()),
        "completed steps must survive failure — needed for retry"
    );
    assert!(
        rec.worktree.is_some(),
        "worktree must survive failure — needed for investigation"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// NO SILENT FAILURES
//
// Every error path must produce an observable signal — event, log, or
// error return. Nothing swallowed silently.
// ═══════════════════════════════════════════════════════════════════════════

