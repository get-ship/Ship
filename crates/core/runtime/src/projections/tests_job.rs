use super::{JobStatus, project};
use crate::events::EventEnvelope;
use crate::events::job::event_types;

fn make_event(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
    EventEnvelope::new(event_type, "test-entity", &payload).unwrap()
}

fn created_event(job_id: &str) -> EventEnvelope {
    make_event(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": job_id,
            "slug": "auth-tests",
            "agent": "rust-runtime",
            "branch": "job/auth-tests",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null
        }),
    )
}

#[test]
fn project_creates_pending_record() {
    let events = vec![created_event("j1")];
    let map = project(&events);
    let rec = map.get("j1").unwrap();
    assert_eq!(rec.status, JobStatus::Pending);
    assert_eq!(rec.slug, "auth-tests");
    assert_eq!(rec.agent, "rust-runtime");
}

#[test]
fn project_dispatched_sets_worktree() {
    let events = vec![
        created_event("j2"),
        make_event(
            event_types::JOB_DISPATCHED,
            serde_json::json!({"job_id": "j2", "worktree": "/tmp/wt", "pid": 1234}),
        ),
    ];
    let map = project(&events);
    let rec = map.get("j2").unwrap();
    assert_eq!(rec.status, JobStatus::Dispatched);
    assert_eq!(rec.worktree.as_deref(), Some("/tmp/wt"));
}

#[test]
fn project_gate_requested_then_passed() {
    let events = vec![
        created_event("j3"),
        make_event(
            event_types::JOB_GATE_REQUESTED,
            serde_json::json!({"job_id": "j3", "gate_agent": "gatekeeper"}),
        ),
        make_event(
            event_types::JOB_GATE_PASSED,
            serde_json::json!({"job_id": "j3"}),
        ),
    ];
    let map = project(&events);
    assert_eq!(map.get("j3").unwrap().status, JobStatus::Pending);
}

#[test]
fn project_gate_failed_stores_reason() {
    let events = vec![
        created_event("j4"),
        make_event(
            event_types::JOB_GATE_FAILED,
            serde_json::json!({"job_id": "j4", "reason": "tests failed"}),
        ),
    ];
    let map = project(&events);
    let rec = map.get("j4").unwrap();
    assert_eq!(rec.status, JobStatus::Failed);
    assert_eq!(rec.error.as_deref(), Some("tests failed"));
}

#[test]
fn project_blocked_stores_blocker() {
    let events = vec![
        created_event("j5"),
        make_event(
            event_types::JOB_BLOCKED,
            serde_json::json!({"job_id": "j5", "blocker": "waiting for infra", "needs_human": true}),
        ),
    ];
    let map = project(&events);
    let rec = map.get("j5").unwrap();
    assert_eq!(rec.status, JobStatus::Blocked);
    assert_eq!(rec.blocker.as_deref(), Some("waiting for infra"));
}

#[test]
fn project_merged() {
    let events = vec![
        created_event("j6"),
        make_event(
            event_types::JOB_MERGED,
            serde_json::json!({"job_id": "j6"}),
        ),
    ];
    let map = project(&events);
    assert_eq!(map.get("j6").unwrap().status, JobStatus::Merged);
}

#[test]
fn project_failed_stores_error() {
    let events = vec![
        created_event("j7"),
        make_event(
            event_types::JOB_FAILED,
            serde_json::json!({"job_id": "j7", "error": "panic in build"}),
        ),
    ];
    let map = project(&events);
    let rec = map.get("j7").unwrap();
    assert_eq!(rec.status, JobStatus::Failed);
    assert_eq!(rec.error.as_deref(), Some("panic in build"));
}

#[test]
fn project_completed_status() {
    let events = vec![
        created_event("j-comp"),
        make_event(
            event_types::JOB_COMPLETED,
            serde_json::json!({"job_id": "j-comp", "slug": "auth-tests"}),
        ),
    ];
    let map = project(&events);
    assert_eq!(map.get("j-comp").unwrap().status, JobStatus::Completed);
}

#[test]
fn project_depends_on_stored() {
    let events = vec![make_event(
        event_types::JOB_CREATED,
        serde_json::json!({
            "job_id": "j-dep",
            "slug": "gated-job",
            "agent": "rust-runtime",
            "branch": "job/gated-job",
            "spec_path": ".artifacts/job-spec.md",
            "plan_id": null,
            "depends_on": ["upstream-a", "upstream-b"]
        }),
    )];
    let map = project(&events);
    let rec = map.get("j-dep").unwrap();
    assert_eq!(
        rec.depends_on.as_deref(),
        Some(&["upstream-a".to_string(), "upstream-b".to_string()][..])
    );
}

#[test]
fn project_depends_on_null_is_none() {
    let events = vec![created_event("j-nodep")];
    let map = project(&events);
    assert!(map.get("j-nodep").unwrap().depends_on.is_none());
}

#[test]
fn project_skips_non_job_events() {
    let events = vec![
        make_event("session.started", serde_json::json!({})),
        created_event("j8"),
    ];
    let map = project(&events);
    assert_eq!(map.len(), 1);
    assert!(map.contains_key("j8"));
}

#[test]
fn project_ignores_event_for_unknown_job() {
    // A dispatched event for a job_id we never saw created — should not panic.
    let events = vec![make_event(
        event_types::JOB_DISPATCHED,
        serde_json::json!({"job_id": "unknown", "worktree": "/x", "pid": null}),
    )];
    let map = project(&events);
    assert!(map.is_empty());
}

#[test]
fn project_pipeline_phase_tracking() {
    // Simulate a 2-phase pipeline: created → dispatched (phase 0) → completed → dispatched (phase 1)
    let events = vec![
        make_event(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-pipe",
                "slug": "pipe-test",
                "agent": "test-writer",
                "branch": "job/pipe-test",
                "spec_path": "spec.md",
                "plan_id": null,
                "pipeline": [
                    { "agent": "test-writer", "goal": "Write tests" },
                    { "agent": "rust-runtime", "goal": "Implement" }
                ]
            }),
        ),
        // Phase 0 dispatched
        make_event(
            event_types::JOB_DISPATCHED,
            serde_json::json!({"job_id": "j-pipe", "worktree": "/tmp/wt", "pid": null}),
        ),
        // Phase 0 completed
        make_event(
            event_types::JOB_COMPLETED,
            serde_json::json!({"job_id": "j-pipe", "slug": "pipe-test"}),
        ),
        // Phase 1 dispatched (pipeline advance)
        make_event(
            event_types::JOB_DISPATCHED,
            serde_json::json!({"job_id": "j-pipe", "worktree": "/tmp/wt", "pid": null}),
        ),
    ];
    let map = project(&events);
    let rec = map.get("j-pipe").unwrap();
    assert_eq!(rec.status, JobStatus::Dispatched);
    assert_eq!(rec.current_phase, Some(1), "current_phase must be 1 after pipeline advance");
    assert!(rec.pipeline.is_some());
    assert_eq!(rec.pipeline.as_ref().unwrap().len(), 2);
}

#[test]
fn project_pipeline_full_lifecycle() {
    // 3-phase pipeline: all phases complete
    let events = vec![
        make_event(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-3p",
                "slug": "three-phase",
                "agent": "spec",
                "branch": "job/three-phase",
                "spec_path": "spec.md",
                "plan_id": null,
                "pipeline": [
                    { "agent": "spec", "goal": "Write spec" },
                    { "agent": "test-writer", "goal": "Write tests" },
                    { "agent": "rust-runtime", "goal": "Implement" }
                ]
            }),
        ),
        // Phase 0 dispatched + completed
        make_event(event_types::JOB_DISPATCHED, serde_json::json!({"job_id": "j-3p", "worktree": "/w", "pid": null})),
        make_event(event_types::JOB_COMPLETED, serde_json::json!({"job_id": "j-3p"})),
        // Phase 1 dispatched + completed
        make_event(event_types::JOB_DISPATCHED, serde_json::json!({"job_id": "j-3p", "worktree": "/w", "pid": null})),
        make_event(event_types::JOB_COMPLETED, serde_json::json!({"job_id": "j-3p"})),
        // Phase 2 dispatched + completed
        make_event(event_types::JOB_DISPATCHED, serde_json::json!({"job_id": "j-3p", "worktree": "/w", "pid": null})),
        make_event(event_types::JOB_COMPLETED, serde_json::json!({"job_id": "j-3p"})),
        // Merged after all phases
        make_event(event_types::JOB_MERGED, serde_json::json!({"job_id": "j-3p"})),
    ];
    let map = project(&events);
    let rec = map.get("j-3p").unwrap();
    assert_eq!(rec.status, JobStatus::Merged);
    assert_eq!(rec.current_phase, Some(2), "current_phase must be 2 after 3 pipeline advances");
}

#[test]
fn project_step_retry_tracks_attempt() {
    let events = vec![
        make_event(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-retry",
                "slug": "retry-test",
                "agent": "rust-lane",
                "branch": "job/retry-test",
                "spec_path": "spec.md",
                "steps": {
                    "steps": [
                        { "id": "implement", "type": "agent", "agent": "rust-lane", "max_retries": 2 },
                        { "id": "gate", "type": "gate", "agent": "reviewer", "after": ["implement"] }
                    ]
                }
            }),
        ),
        make_event(
            event_types::STEP_DISPATCHED,
            serde_json::json!({"job_id": "j-retry", "slug": "retry-test", "step_id": "implement", "step_type": "agent"}),
        ),
        make_event(
            event_types::STEP_COMPLETED,
            serde_json::json!({"job_id": "j-retry", "slug": "retry-test", "step_id": "implement"}),
        ),
        // Gate fails — step.retry emitted for the gated step
        make_event(
            event_types::STEP_RETRY,
            serde_json::json!({
                "job_id": "j-retry",
                "slug": "retry-test",
                "step_id": "implement",
                "step_name": "implement",
                "attempt": 1,
                "reason": "gate_failed"
            }),
        ),
    ];
    let map = project(&events);
    let rec = map.get("j-retry").unwrap();
    assert_eq!(rec.step_retries.get("implement"), Some(&1));
    assert!(!rec.completed_steps.contains(&"implement".to_string()),
        "step must be removed from completed on retry");
    assert_eq!(rec.status, JobStatus::Dispatched);
    assert!(rec.error.is_none(), "error must be cleared on retry");
}

#[test]
fn project_step_retry_increments_on_second_retry() {
    let events = vec![
        make_event(
            event_types::JOB_CREATED,
            serde_json::json!({
                "job_id": "j-retry2",
                "slug": "retry-test-2",
                "agent": "rust-lane",
                "branch": "job/retry-test-2",
                "spec_path": "spec.md",
            }),
        ),
        make_event(
            event_types::STEP_RETRY,
            serde_json::json!({
                "job_id": "j-retry2",
                "slug": "retry-test-2",
                "step_id": "implement",
                "step_name": "implement",
                "attempt": 1,
                "reason": "gate_failed"
            }),
        ),
        make_event(
            event_types::STEP_RETRY,
            serde_json::json!({
                "job_id": "j-retry2",
                "slug": "retry-test-2",
                "step_id": "implement",
                "step_name": "implement",
                "attempt": 2,
                "reason": "gate_failed"
            }),
        ),
    ];
    let map = project(&events);
    let rec = map.get("j-retry2").unwrap();
    assert_eq!(rec.step_retries.get("implement"), Some(&2));
}

#[test]
fn project_non_pipeline_job_has_no_phase() {
    // A normal job (no pipeline) should have current_phase = None
    let events = vec![
        created_event("j-nopipe"),
        make_event(event_types::JOB_DISPATCHED, serde_json::json!({"job_id": "j-nopipe", "worktree": "/w", "pid": null})),
        make_event(event_types::JOB_COMPLETED, serde_json::json!({"job_id": "j-nopipe"})),
    ];
    let map = project(&events);
    let rec = map.get("j-nopipe").unwrap();
    assert_eq!(rec.current_phase, None, "non-pipeline job must not track current_phase");
}

// ── Idempotency ─────────────────────────────────────────────────────────────
//
// Each projection's `apply` must produce the same terminal state regardless
// of how many times each event in the stream is processed. These tests
// concatenate the full event list with itself and assert the result equals
// single-pass projection. Property: for events E, project(E) == project(E ++ E).

mod idempotent {
    use super::*;
    use crate::projections::job::apply;

    fn pipeline_events(job_id: &str) -> Vec<EventEnvelope> {
        vec![
            created_event(job_id),
            make_event(
                event_types::JOB_DISPATCHED,
                serde_json::json!({"job_id": job_id, "worktree": "/tmp/w", "pid": 1}),
            ),
            make_event(
                event_types::STEP_COMPLETED,
                serde_json::json!({"job_id": job_id, "step_id": "step-1"}),
            ),
            make_event(
                event_types::STEP_DISPATCHED,
                serde_json::json!({"job_id": job_id, "step_id": "step-2", "step_type": "agent"}),
            ),
            make_event(
                event_types::STEP_COMPLETED,
                serde_json::json!({"job_id": job_id, "step_id": "step-2"}),
            ),
            make_event(
                event_types::JOB_COMPLETED,
                serde_json::json!({"job_id": job_id, "slug": "auth-tests"}),
            ),
        ]
    }

    fn project_and_get(events: &[EventEnvelope], job_id: &str) -> super::super::JobRecord {
        super::super::project(events).get(job_id).cloned().unwrap()
    }

    #[test]
    fn replay_entire_stream_is_noop() {
        // project(E) == project(E ++ E). Rebuilding projections from a log
        // with duplicated events must produce identical state.
        let events = pipeline_events("j-idem");
        let single = project_and_get(&events, "j-idem");

        let mut doubled_events = events.clone();
        doubled_events.extend(events.iter().cloned());
        let doubled = project_and_get(&doubled_events, "j-idem");

        assert_eq!(single.status, doubled.status);
        assert_eq!(single.completed_steps, doubled.completed_steps);
        assert_eq!(single.current_step, doubled.current_step);
        assert_eq!(single.worktree, doubled.worktree);
        assert_eq!(single.error, doubled.error);
    }

    #[test]
    fn step_completed_push_is_idempotent() {
        // Apply STEP_COMPLETED for the same step twice. Must not produce
        // duplicates in completed_steps.
        let events = vec![
            created_event("j-sc"),
            make_event(
                event_types::STEP_COMPLETED,
                serde_json::json!({"job_id": "j-sc", "step_id": "step-1"}),
            ),
            make_event(
                event_types::STEP_COMPLETED,
                serde_json::json!({"job_id": "j-sc", "step_id": "step-1"}),
            ),
        ];
        let rec = project_and_get(&events, "j-sc");
        assert_eq!(
            rec.completed_steps.iter().filter(|s| *s == "step-1").count(),
            1,
            "duplicate STEP_COMPLETED must not produce duplicate completed_steps entries"
        );
    }

    #[test]
    fn job_dispatched_replay_does_not_corrupt_completed_steps() {
        // The old `.take()` pattern would destructively read current_step on
        // the first pass, then silently skip the "move to completed" block
        // on replay, producing a different final state. This test guards the
        // idempotent rewrite.
        let events = vec![
            created_event("j-dis"),
            make_event(
                event_types::STEP_COMPLETED,
                serde_json::json!({"job_id": "j-dis", "step_id": "step-1"}),
            ),
            make_event(
                event_types::JOB_COMPLETED,
                serde_json::json!({"job_id": "j-dis", "slug": "auth-tests"}),
            ),
            make_event(
                event_types::JOB_DISPATCHED,
                serde_json::json!({"job_id": "j-dis", "worktree": "/w", "pid": 1}),
            ),
        ];

        let first = project_and_get(&events, "j-dis");

        // Build state, then apply events again on top.
        let mut map = super::super::project(&events);
        let rec = map.get_mut("j-dis").unwrap();
        for e in &events {
            apply(rec, e);
        }

        assert_eq!(
            first.completed_steps, rec.completed_steps,
            "completed_steps must not change across replay"
        );
        assert_eq!(
            first.status, rec.status,
            "status must not change across replay"
        );
    }
}
