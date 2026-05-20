//! Pipeline reliability tests — full lifecycle projection coverage for DAG pipelines.
//!
//! These tests exercise the job projection across a variety of pipeline shapes and
//! event sequences, verifying status transitions, step tracking, and merge semantics.

use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types;
use ship_platform::projections::pipeline_run::{PipelineRunStatus, project};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity", &payload).unwrap()
}

fn created(job_id: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_STARTED,
        serde_json::json!({
            "job_id": job_id,
            "slug": job_id,
            "agent": "rust-runtime",
            "branch": format!("job/{job_id}"),
            "spec_path": "spec.md",
            "plan_id": null,
        }),
    )
}

fn created_with_steps(job_id: &str, steps: serde_json::Value) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_STARTED,
        serde_json::json!({
            "job_id": job_id,
            "slug": job_id,
            "agent": "rust-runtime",
            "branch": format!("job/{job_id}"),
            "spec_path": "spec.md",
            "plan_id": null,
            "steps": steps,
        }),
    )
}

fn dispatched(job_id: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_DISPATCHED,
        serde_json::json!({"job_id": job_id, "worktree": "/tmp/wt", "pid": null}),
    )
}

fn completed(job_id: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_COMPLETED,
        serde_json::json!({"job_id": job_id, "slug": job_id}),
    )
}

fn merged(job_id: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_MERGED,
        serde_json::json!({"job_id": job_id}),
    )
}

fn blocked(job_id: &str, reason: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_BLOCKED,
        serde_json::json!({"job_id": job_id, "blocker": reason}),
    )
}

fn linear_2step() -> serde_json::Value {
    serde_json::json!({
        "steps": [
            { "id": "step-a", "type": "agent", "agent": "test-writer", "after": [] },
            { "id": "step-b", "type": "agent", "agent": "rust-runtime", "after": ["step-a"] }
        ]
    })
}

fn linear_3step() -> serde_json::Value {
    serde_json::json!({
        "steps": [
            { "id": "spec", "type": "agent", "agent": "spec", "after": [] },
            { "id": "tests", "type": "agent", "agent": "test-writer", "after": ["spec"] },
            { "id": "impl", "type": "agent", "agent": "rust-runtime", "after": ["tests"] }
        ]
    })
}

// ── Single-step lifecycle ─────────────────────────────────────────────────────

/// Newly created pipeline job starts Pending.
#[test]
fn single_step_job_starts_pending() {
    let events = vec![created_with_steps("j-p1", serde_json::json!({
        "steps": [{ "id": "only", "type": "agent", "agent": "rust-runtime", "after": [] }]
    }))];
    assert_eq!(project(&events).get("j-p1").unwrap().status, PipelineRunStatus::Pending);
}

/// Single-step job transitions to Dispatched after first dispatch event.
#[test]
fn single_step_job_dispatches_from_pending() {
    let events = vec![
        created_with_steps("j-p2", serde_json::json!({
            "steps": [{ "id": "impl", "type": "agent", "agent": "rust-runtime", "after": [] }]
        })),
        dispatched("j-p2"),
    ];
    let rec = project(&events).remove("j-p2").unwrap();
    assert_eq!(rec.status, PipelineRunStatus::Dispatched);
    assert_eq!(rec.current_step.as_deref(), Some("impl"));
}

/// Single-step job transitions to Completed after completion event.
#[test]
fn single_step_job_completes_from_dispatched() {
    let steps = serde_json::json!({
        "steps": [{ "id": "impl", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    let events = vec![
        created_with_steps("j-p3", steps),
        dispatched("j-p3"),
        completed("j-p3"),
    ];
    assert_eq!(project(&events).get("j-p3").unwrap().status, PipelineRunStatus::Completed);
}

/// Single-step job with merge reaches Merged.
#[test]
fn single_step_job_reaches_merged() {
    let steps = serde_json::json!({
        "steps": [{ "id": "impl", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    let events = vec![
        created_with_steps("j-p4", steps),
        dispatched("j-p4"),
        completed("j-p4"),
        merged("j-p4"),
    ];
    assert_eq!(project(&events).get("j-p4").unwrap().status, PipelineRunStatus::Merged);
}

// ── Two-step linear lifecycle ─────────────────────────────────────────────────

/// Two-step pipeline: step-a is current after first dispatch.
#[test]
fn two_step_job_first_step_is_current_on_dispatch() {
    let events = vec![
        created_with_steps("j-2a", linear_2step()),
        dispatched("j-2a"),
    ];
    assert_eq!(project(&events).get("j-2a").unwrap().current_step.as_deref(), Some("step-a"));
}

/// Two-step pipeline: status is Completed after step-a finishes.
#[test]
fn two_step_job_first_step_completes() {
    let events = vec![
        created_with_steps("j-2b", linear_2step()),
        dispatched("j-2b"),
        completed("j-2b"),
    ];
    assert_eq!(project(&events).get("j-2b").unwrap().status, PipelineRunStatus::Completed);
}

/// Two-step pipeline: step-b becomes current after step-a completes and pipeline advances.
#[test]
fn two_step_job_second_step_becomes_current_after_advance() {
    let events = vec![
        created_with_steps("j-2c", linear_2step()),
        dispatched("j-2c"),
        completed("j-2c"),
        dispatched("j-2c"),
    ];
    assert_eq!(project(&events).get("j-2c").unwrap().current_step.as_deref(), Some("step-b"));
}

/// Two-step pipeline: step-a is in completed_steps after pipeline advances.
#[test]
fn two_step_job_first_step_in_completed_after_advance() {
    let events = vec![
        created_with_steps("j-2d", linear_2step()),
        dispatched("j-2d"),
        completed("j-2d"),
        dispatched("j-2d"),
    ];
    let rec = project(&events).remove("j-2d").unwrap();
    assert!(rec.completed_steps.contains(&"step-a".to_string()));
}

/// Two-step pipeline: both steps are in completed_steps after step-b completes.
#[test]
fn two_step_job_both_steps_completed_after_step_b() {
    let events = vec![
        created_with_steps("j-2e", linear_2step()),
        dispatched("j-2e"),
        completed("j-2e"),
        dispatched("j-2e"),
        completed("j-2e"),
    ];
    let rec = project(&events).remove("j-2e").unwrap();
    assert!(rec.completed_steps.contains(&"step-a".to_string()));
    assert!(rec.completed_steps.contains(&"step-b".to_string()));
}

/// Two-step pipeline reaches Merged.
#[test]
fn two_step_job_reaches_merged() {
    let events = vec![
        created_with_steps("j-2f", linear_2step()),
        dispatched("j-2f"),
        completed("j-2f"),
        dispatched("j-2f"),
        completed("j-2f"),
        merged("j-2f"),
    ];
    assert_eq!(project(&events).get("j-2f").unwrap().status, PipelineRunStatus::Merged);
}

// ── Three-step linear lifecycle ───────────────────────────────────────────────

/// Three-step pipeline: spec step is current on first dispatch.
#[test]
fn three_step_job_first_step_is_current() {
    let events = vec![
        created_with_steps("j-3a", linear_3step()),
        dispatched("j-3a"),
    ];
    assert_eq!(project(&events).get("j-3a").unwrap().current_step.as_deref(), Some("spec"));
}

/// Three-step pipeline: all 3 steps tracked after full lifecycle.
#[test]
fn three_step_job_all_steps_tracked() {
    let events = vec![
        created_with_steps("j-3b", linear_3step()),
        dispatched("j-3b"), completed("j-3b"),
        dispatched("j-3b"), completed("j-3b"),
        dispatched("j-3b"), completed("j-3b"),
    ];
    let rec = project(&events).remove("j-3b").unwrap();
    assert_eq!(rec.completed_steps.len(), 3);
    assert!(rec.completed_steps.contains(&"spec".to_string()));
    assert!(rec.completed_steps.contains(&"tests".to_string()));
    assert!(rec.completed_steps.contains(&"impl".to_string()));
}

/// Three-step pipeline: intermediate statuses are correct.
#[test]
fn three_step_job_intermediate_statuses() {
    // After 2nd dispatch
    let events = vec![
        created_with_steps("j-3c", linear_3step()),
        dispatched("j-3c"), completed("j-3c"),
        dispatched("j-3c"),
    ];
    assert_eq!(project(&events).get("j-3c").unwrap().current_step.as_deref(), Some("tests"));
}

// ── Full lifecycle (THE KEY TEST) ─────────────────────────────────────────────

/// Full lifecycle of a 3-step DAG pipeline job ending with merge.
///
/// Validates that all three steps are tracked in completed_steps and the final
/// status is Merged — including the last step, which completes without a
/// subsequent PIPELINE_RUN_DISPATCHED (merge fires instead).
#[test]
fn pipeline_full_lifecycle_reaches_merged() {
    let events = vec![
        created_with_steps("j-full", linear_3step()),
        // Step 1
        dispatched("j-full"),
        completed("j-full"),
        // Step 2
        dispatched("j-full"),
        completed("j-full"),
        // Step 3 — final step completes, then merge fires without another dispatch
        dispatched("j-full"),
        completed("j-full"),
        merged("j-full"),
    ];
    let rec = project(&events).remove("j-full").unwrap();
    assert_eq!(rec.status, PipelineRunStatus::Merged, "final status must be Merged");
    assert_eq!(rec.completed_steps.len(), 3, "all 3 steps must be tracked");
    assert!(rec.completed_steps.contains(&"spec".to_string()), "spec must be tracked");
    assert!(rec.completed_steps.contains(&"tests".to_string()), "tests must be tracked");
    assert!(
        rec.completed_steps.contains(&"impl".to_string()),
        "impl (final step) must be tracked — this was the bug"
    );
}

// ── Edge cases and invariants ─────────────────────────────────────────────────

/// Freshly created pipeline job has no completed steps.
#[test]
fn pipeline_job_has_no_completed_steps_initially() {
    let events = vec![created_with_steps("j-init", linear_2step())];
    assert!(project(&events).get("j-init").unwrap().completed_steps.is_empty());
}

/// No pipeline job has no step tracking.
#[test]
fn no_pipeline_job_has_no_current_step() {
    let events = vec![created("j-nopipe"), dispatched("j-nopipe"), completed("j-nopipe")];
    let rec = project(&events).remove("j-nopipe").unwrap();
    assert!(rec.current_step.is_none());
    assert!(rec.completed_steps.is_empty());
}

/// Blocked mid-lifecycle doesn't lose step tracking state.
#[test]
fn pipeline_blocked_mid_lifecycle_retains_state() {
    let events = vec![
        created_with_steps("j-blk", linear_2step()),
        dispatched("j-blk"),
        blocked("j-blk", "waiting for human"),
    ];
    let rec = project(&events).remove("j-blk").unwrap();
    assert_eq!(rec.status, PipelineRunStatus::Blocked);
    assert_eq!(rec.current_step.as_deref(), Some("step-a"));
    assert!(rec.completed_steps.is_empty());
}

/// completed_steps count grows monotonically across lifecycle.
#[test]
fn completed_steps_count_grows_monotonically() {
    let mut events = vec![
        created_with_steps("j-mono", linear_3step()),
        dispatched("j-mono"),
    ];
    let m0 = project(&events).remove("j-mono").unwrap().completed_steps.len();

    events.push(completed("j-mono"));
    let m1 = project(&events).remove("j-mono").unwrap().completed_steps.len();

    events.push(dispatched("j-mono"));
    events.push(completed("j-mono"));
    let m2 = project(&events).remove("j-mono").unwrap().completed_steps.len();

    assert!(m0 <= m1 && m1 <= m2, "completed_steps must grow monotonically: {m0} → {m1} → {m2}");
}

/// Final step completion (without re-dispatch) is tracked in completed_steps.
#[test]
fn final_step_tracked_without_subsequent_dispatch() {
    let steps = serde_json::json!({
        "steps": [{ "id": "last", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    let events = vec![
        created_with_steps("j-last", steps),
        dispatched("j-last"),
        completed("j-last"),
    ];
    let rec = project(&events).remove("j-last").unwrap();
    assert!(rec.completed_steps.contains(&"last".to_string()));
}

/// current_step is None after PIPELINE_RUN_COMPLETED clears it.
#[test]
fn current_step_cleared_after_completion() {
    let steps = serde_json::json!({
        "steps": [{ "id": "only", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    let events = vec![
        created_with_steps("j-clr", steps),
        dispatched("j-clr"),
        completed("j-clr"),
    ];
    assert!(project(&events).get("j-clr").unwrap().current_step.is_none());
}

/// Parallel root steps: both are unblocked simultaneously.
#[test]
fn parallel_root_steps_both_unblocked() {
    let steps = serde_json::json!({
        "steps": [
            { "id": "a", "type": "agent", "agent": "spec", "after": [] },
            { "id": "b", "type": "agent", "agent": "test-writer", "after": [] },
            { "id": "c", "type": "agent", "agent": "rust-runtime", "after": ["a", "b"] }
        ]
    });
    let events = vec![created_with_steps("j-par", steps)];
    let rec = project(&events).remove("j-par").unwrap();
    // Verify the pipeline has 2 root steps (both have empty `after`)
    let pipeline = rec.steps.as_ref().unwrap();
    assert_eq!(pipeline.root_steps().len(), 2);
}

/// Diamond DAG: final step unblocked only after both branches complete.
#[test]
fn diamond_dag_final_step_unblocked_after_both_branches() {
    let steps = serde_json::json!({
        "steps": [
            { "id": "root", "type": "agent", "agent": "spec", "after": [] },
            { "id": "left", "type": "agent", "agent": "test-writer", "after": ["root"] },
            { "id": "right", "type": "agent", "agent": "reviewer", "after": ["root"] },
            { "id": "merge", "type": "agent", "agent": "rust-runtime", "after": ["left", "right"] }
        ]
    });
    let events = vec![created_with_steps("j-dia", steps)];
    let rec = project(&events).remove("j-dia").unwrap();
    let pipeline = rec.steps.as_ref().unwrap();

    // With root completed: left and right are unblocked
    let after_root = pipeline.unblocked_steps(&["root".to_string()]);
    let ids: Vec<&str> = after_root.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"left"));
    assert!(ids.contains(&"right"));

    // With left completed but not right: merge is still blocked
    let after_left = pipeline.unblocked_steps(&["root".to_string(), "left".to_string()]);
    let ids: Vec<&str> = after_left.iter().map(|s| s.id.as_str()).collect();
    assert!(!ids.contains(&"merge"), "merge must be blocked until right also completes");

    // With both: merge is unblocked
    let after_both = pipeline.unblocked_steps(&[
        "root".to_string(), "left".to_string(), "right".to_string(),
    ]);
    let ids: Vec<&str> = after_both.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"merge"));
}

/// Pipeline job with steps field retains steps definition throughout lifecycle.
#[test]
fn pipeline_steps_definition_persists_across_events() {
    let events = vec![
        created_with_steps("j-def", linear_3step()),
        dispatched("j-def"),
        completed("j-def"),
        merged("j-def"),
    ];
    let rec = project(&events).remove("j-def").unwrap();
    assert!(rec.steps.is_some(), "steps definition must persist on the record");
    assert_eq!(rec.steps.unwrap().steps.len(), 3);
}

/// All completed_steps at merge match the pipeline step count.
#[test]
fn all_completed_steps_at_merge_match_pipeline_definition() {
    let events = vec![
        created_with_steps("j-cnt", linear_3step()),
        dispatched("j-cnt"), completed("j-cnt"),
        dispatched("j-cnt"), completed("j-cnt"),
        dispatched("j-cnt"), completed("j-cnt"),
        merged("j-cnt"),
    ];
    let rec = project(&events).remove("j-cnt").unwrap();
    let step_count = rec.steps.as_ref().unwrap().steps.len();
    assert_eq!(
        rec.completed_steps.len(),
        step_count,
        "completed_steps count at merge must equal pipeline step count"
    );
}
