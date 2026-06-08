//! Unit tests for DAG pipeline step tracking in the job projection.
//!
//! Covers the `completed_steps` and `current_step` fields across the full
//! lifecycle of jobs with a `steps` (DAG pipeline) field.

use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types;
use ship_pipeline_runtime::projections::pipeline_run::{PipelineRunStatus, project};

fn ev(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, "test-entity", &payload).unwrap()
}

fn created_with_steps(pipeline_run_id: &str, steps: serde_json::Value) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_STARTED,
        serde_json::json!({
            "pipeline_run_id": pipeline_run_id,
            "slug": pipeline_run_id,
            "agent": "rust-runtime",
            "branch": format!("job/{pipeline_run_id}"),
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "steps": steps,
        }),
    )
}

fn dispatched(pipeline_run_id: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_DISPATCHED,
        serde_json::json!({"pipeline_run_id": pipeline_run_id, "worktree": "/tmp/wt", "pid": null}),
    )
}

fn completed(pipeline_run_id: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_COMPLETED,
        serde_json::json!({"pipeline_run_id": pipeline_run_id, "slug": pipeline_run_id}),
    )
}

fn merged(pipeline_run_id: &str) -> EventEnvelope {
    ev(
        event_types::PIPELINE_RUN_MERGED,
        serde_json::json!({"pipeline_run_id": pipeline_run_id}),
    )
}

/// Single-step pipeline: after dispatch, current_step is set to the root step.
#[test]
fn current_step_set_on_first_dispatch() {
    let steps = serde_json::json!({
        "steps": [{ "id": "implement", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    let events = vec![
        created_with_steps("j-s1", steps),
        dispatched("j-s1"),
    ];
    let map = project(&events);
    let rec = map.get("j-s1").unwrap();
    assert_eq!(rec.current_step.as_deref(), Some("implement"));
    assert!(rec.completed_steps.is_empty());
}

/// Single-step pipeline: after completion, the step appears in completed_steps.
#[test]
fn single_step_pipeline_tracked_on_completion() {
    let steps = serde_json::json!({
        "steps": [{ "id": "implement", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    let events = vec![
        created_with_steps("j-s2", steps),
        dispatched("j-s2"),
        completed("j-s2"),
    ];
    let map = project(&events);
    let rec = map.get("j-s2").unwrap();
    assert!(
        rec.completed_steps.contains(&"implement".to_string()),
        "completed step must appear in completed_steps after PIPELINE_RUN_COMPLETED"
    );
}

/// Two-step linear pipeline: after step-1 completes, step-2 becomes current.
#[test]
fn two_step_pipeline_step2_becomes_current_after_step1_completes() {
    let steps = serde_json::json!({
        "steps": [
            { "id": "write-tests", "type": "agent", "agent": "test-writer", "after": [] },
            { "id": "implement", "type": "agent", "agent": "rust-runtime", "after": ["write-tests"] }
        ]
    });
    let events = vec![
        created_with_steps("j-2step", steps),
        dispatched("j-2step"),
        completed("j-2step"),
        dispatched("j-2step"), // advance to step 2
    ];
    let map = project(&events);
    let rec = map.get("j-2step").unwrap();
    assert_eq!(rec.current_step.as_deref(), Some("implement"));
    assert!(rec.completed_steps.contains(&"write-tests".to_string()));
}

/// Two-step pipeline: the second step is tracked on completion.
#[test]
fn two_step_pipeline_step2_tracked_on_completion() {
    let steps = serde_json::json!({
        "steps": [
            { "id": "write-tests", "type": "agent", "agent": "test-writer", "after": [] },
            { "id": "implement", "type": "agent", "agent": "rust-runtime", "after": ["write-tests"] }
        ]
    });
    let events = vec![
        created_with_steps("j-2step-c", steps),
        dispatched("j-2step-c"),
        completed("j-2step-c"),
        dispatched("j-2step-c"),
        completed("j-2step-c"),
    ];
    let map = project(&events);
    let rec = map.get("j-2step-c").unwrap();
    assert!(rec.completed_steps.contains(&"write-tests".to_string()));
    assert!(rec.completed_steps.contains(&"implement".to_string()));
}

/// No job without a DAG pipeline should accumulate completed_steps.
#[test]
fn no_pipeline_job_no_step_tracking() {
    let events = vec![
        ev(
            event_types::PIPELINE_RUN_STARTED,
            serde_json::json!({
                "pipeline_run_id": "j-nopipe",
                "slug": "no-pipe",
                "agent": "rust-runtime",
                "branch": "job/no-pipe",
                "spec_path": "spec.md",
                "plan_id": null,
            }),
        ),
        dispatched("j-nopipe"),
        completed("j-nopipe"),
    ];
    let map = project(&events);
    let rec = map.get("j-nopipe").unwrap();
    assert!(rec.completed_steps.is_empty(), "non-DAG job must not accumulate completed_steps");
    assert!(rec.current_step.is_none(), "non-DAG job must not set current_step");
}

/// A step already in completed_steps is not added a second time.
#[test]
fn completed_steps_not_duplicated() {
    let steps = serde_json::json!({
        "steps": [
            { "id": "step-a", "type": "agent", "agent": "rust-runtime", "after": [] },
            { "id": "step-b", "type": "agent", "agent": "rust-runtime", "after": ["step-a"] }
        ]
    });
    // Advance twice through step-a path: complete → dispatch → complete → dispatch
    let events = vec![
        created_with_steps("j-dedup", steps),
        dispatched("j-dedup"),
        completed("j-dedup"),
        dispatched("j-dedup"),
        completed("j-dedup"),
        dispatched("j-dedup"), // no-op advance (no more steps)
    ];
    let map = project(&events);
    let rec = map.get("j-dedup").unwrap();
    let count = rec.completed_steps.iter().filter(|s| s.as_str() == "step-a").count();
    assert_eq!(count, 1, "step-a must appear exactly once in completed_steps");
}

/// Verifies current_step is cleared after completion with no subsequent dispatch.
#[test]
fn current_step_none_after_completion_no_next_dispatch() {
    let steps = serde_json::json!({
        "steps": [{ "id": "only-step", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    let events = vec![
        created_with_steps("j-clear", steps),
        dispatched("j-clear"),
        completed("j-clear"),
    ];
    let map = project(&events);
    let rec = map.get("j-clear").unwrap();
    assert!(
        rec.current_step.is_none(),
        "current_step must be None after PIPELINE_RUN_COMPLETED clears it"
    );
}

/// THE KEY TEST: A step that completes without a subsequent PIPELINE_RUN_DISPATCHED
/// (e.g. merge fires immediately after) must still appear in completed_steps.
///
/// Before the fix: `completed_steps` was only updated in `PIPELINE_RUN_DISPATCHED`.
/// After the fix: `PIPELINE_RUN_COMPLETED` also pushes `current_step` into `completed_steps`.
#[test]
fn completion_without_subsequent_dispatch_still_trackable() {
    let steps = serde_json::json!({
        "steps": [{ "id": "final-step", "type": "agent", "agent": "rust-runtime", "after": [] }]
    });
    // Sequence: created → dispatched → completed → merged (no second dispatch)
    let events = vec![
        created_with_steps("j-no-redispatch", steps),
        dispatched("j-no-redispatch"),
        completed("j-no-redispatch"),
        merged("j-no-redispatch"),
    ];
    let map = project(&events);
    let rec = map.get("j-no-redispatch").unwrap();
    assert_eq!(rec.status, PipelineRunStatus::Merged);
    assert!(
        rec.completed_steps.contains(&"final-step".to_string()),
        "final-step must be in completed_steps even when PIPELINE_RUN_MERGED fires without a second PIPELINE_RUN_DISPATCHED"
    );
}

/// THE KEY TEST: A 3-step linear pipeline where all steps complete before merge.
/// All three steps must appear in completed_steps.
///
/// Before the fix: the last step was lost because it completed without a
/// subsequent PIPELINE_RUN_DISPATCHED to trigger the completed_steps update.
#[test]
fn three_steps_completed_before_merge() {
    let steps = serde_json::json!({
        "steps": [
            { "id": "spec", "type": "agent", "agent": "spec", "after": [] },
            { "id": "tests", "type": "agent", "agent": "test-writer", "after": ["spec"] },
            { "id": "impl", "type": "agent", "agent": "rust-runtime", "after": ["tests"] }
        ]
    });
    let events = vec![
        created_with_steps("j-3steps", steps),
        // Step 1: spec
        dispatched("j-3steps"),
        completed("j-3steps"),
        // Step 2: tests
        dispatched("j-3steps"),
        completed("j-3steps"),
        // Step 3: impl — completes then immediately merged (no third dispatch)
        dispatched("j-3steps"),
        completed("j-3steps"),
        merged("j-3steps"),
    ];
    let map = project(&events);
    let rec = map.get("j-3steps").unwrap();
    assert_eq!(rec.status, PipelineRunStatus::Merged);
    assert!(
        rec.completed_steps.contains(&"spec".to_string()),
        "step 'spec' must be in completed_steps"
    );
    assert!(
        rec.completed_steps.contains(&"tests".to_string()),
        "step 'tests' must be in completed_steps"
    );
    assert!(
        rec.completed_steps.contains(&"impl".to_string()),
        "step 'impl' must be in completed_steps — this was the bug"
    );
    assert_eq!(
        rec.completed_steps.len(),
        3,
        "all three steps must be tracked"
    );
}
