//! Behavioral tests for the per-step read model (R18: feed events through the
//! fold, assert on the resulting ROWS — never on source text or private shape).

use super::*;
use agent_def::canonical_session::event_types as set;
use chrono::{DateTime, TimeZone, Utc};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types as pet;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Deterministic timestamp `base + secs`, so started_at/completed_at ordering is
/// assertable (EventEnvelope::new stamps Utc::now(), which we override).
fn at(secs: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_700_000_000 + secs, 0).single().unwrap()
}

fn ev(event_type: &str, ts: i64, payload: serde_json::Value) -> EventEnvelope {
    let mut e = EventEnvelope::new(event_type, "entity", &payload).unwrap();
    e.created_at = at(ts);
    e
}

/// `pipeline.run.started` carrying a 3-step DAG: a → b → c.
fn run_started(run_id: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_RUN_STARTED,
        ts,
        serde_json::json!({
            "pipeline_run_id": run_id,
            "slug": "myrun",
            "agent": "rust-lane",
            "branch": "job/myrun",
            "spec_path": ".artifacts/job-spec.md",
            "steps": {
                "steps": [
                    { "id": "a", "type": "agent" },
                    { "id": "b", "type": "agent", "after": ["a"] },
                    { "id": "c", "type": "run", "run": "cargo test", "after": ["b"] }
                ],
                "auto_merge": false
            }
        }),
    )
}

fn step_started(run_id: &str, step_id: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_STEP_STARTED,
        ts,
        serde_json::json!({
            "pipeline_run_id": run_id, "slug": "myrun",
            "step_id": step_id, "step_type": "agent"
        }),
    )
}

fn step_completed(run_id: &str, step_id: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_STEP_COMPLETED,
        ts,
        serde_json::json!({ "pipeline_run_id": run_id, "slug": "myrun", "step_id": step_id }),
    )
}

fn step_failed(run_id: &str, step_id: &str, error: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_STEP_FAILED,
        ts,
        serde_json::json!({
            "pipeline_run_id": run_id, "slug": "myrun",
            "step_id": step_id, "error": error
        }),
    )
}

fn session_started(run_id: &str, step_id: &str, session_id: &str, ts: i64) -> EventEnvelope {
    ev(
        set::SESSION_STARTED,
        ts,
        serde_json::json!({
            "session_id": session_id,
            "agent_id": "rust-lane",
            "harness": "claude",
            "space_id": "",
            "slug": format!("myrun::{step_id}"),
            "parent": run_id
        }),
    )
}

fn session_completed(session_id: &str, ts: i64) -> EventEnvelope {
    ev(
        set::SESSION_COMPLETED,
        ts,
        serde_json::json!({ "session_id": session_id }),
    )
}

fn session_failed(session_id: &str, reason: &str, ts: i64) -> EventEnvelope {
    ev(
        set::SESSION_FAILED,
        ts,
        serde_json::json!({ "session_id": session_id, "reason": reason }),
    )
}

fn row<'m>(map: &'m StepRunMap, run_id: &str, step_id: &str) -> &'m PipelineStepRun {
    map.get(&(run_id.to_string(), step_id.to_string()))
        .unwrap_or_else(|| panic!("missing row for ({run_id}, {step_id})"))
}

// ── Primary sequence (the task's oracle) ───────────────────────────────────────

/// run.started → step.started(a) → step.completed(a)
///   → child session.started/completed(step=b) → step.failed(c)
///
/// Assert the rows carry the right status / timestamps / depends_on, and that
/// step b reaches completed via the CHILD SESSION TERMINAL (not a step event).
#[test]
fn fold_full_lifecycle_sequence() {
    let run = "run-1";
    let events = vec![
        run_started(run, 0),
        step_started(run, "a", 1),
        step_completed(run, "a", 2),
        session_started(run, "b", "sess-b", 3),
        session_completed("sess-b", 4),
        step_failed(run, "c", "compile error", 5),
    ];
    let map = project(&events);

    // a: ran then completed via durable step events.
    let a = row(&map, run, "a");
    assert_eq!(a.status, StepRunStatus::Completed);
    assert_eq!(a.started_at, Some(at(1)));
    assert_eq!(a.completed_at, Some(at(2)));
    assert_eq!(a.depends_on, None, "root step has no DAG edge");
    assert_eq!(a.step_type, "agent");

    // b: completed via the child session terminal — never saw a step.completed.
    let b = row(&map, run, "b");
    assert_eq!(
        b.status,
        StepRunStatus::Completed,
        "child session.completed must complete the step"
    );
    assert_eq!(b.started_at, Some(at(3)), "started_at from session.started");
    assert_eq!(b.completed_at, Some(at(4)), "completed_at from session.completed");
    assert_eq!(b.session_id.as_deref(), Some("sess-b"));
    assert_eq!(b.depends_on, Some(vec!["a".to_string()]));

    // c: failed via durable step.failed; DAG edge preserved.
    let c = row(&map, run, "c");
    assert_eq!(c.status, StepRunStatus::Failed);
    assert_eq!(c.completed_at, Some(at(5)));
    assert_eq!(c.depends_on, Some(vec!["b".to_string()]));
    assert_eq!(c.step_type, "run", "step_type seeded from the DAG");
    assert_eq!(c.note.as_deref(), Some("compile error"));
}

// ── Child-session terminal anchor (explicit) ───────────────────────────────────

#[test]
fn child_session_completed_marks_step_completed() {
    let run = "run-cs";
    let events = vec![
        run_started(run, 0),
        session_started(run, "b", "S1", 1),
        session_completed("S1", 2),
    ];
    let map = project(&events);
    let b = row(&map, run, "b");
    assert_eq!(b.status, StepRunStatus::Completed);
    assert_eq!(b.started_at, Some(at(1)));
    assert_eq!(b.completed_at, Some(at(2)));
}

#[test]
fn child_session_failed_marks_step_failed_with_reason() {
    let run = "run-cf";
    let events = vec![
        run_started(run, 0),
        session_started(run, "b", "S2", 1),
        session_failed("S2", "harness exit 1", 2),
    ];
    let map = project(&events);
    let b = row(&map, run, "b");
    assert_eq!(b.status, StepRunStatus::Failed);
    assert_eq!(b.completed_at, Some(at(2)));
    assert_eq!(b.note.as_deref(), Some("harness exit 1"));
}

#[test]
fn unparented_session_terminal_touches_no_step() {
    // A bare `ship use` session (no parent / no `::` slug) must never create or
    // mutate a step row.
    let bare_started = ev(
        set::SESSION_STARTED,
        0,
        serde_json::json!({
            "session_id": "bare", "agent_id": "x", "harness": "claude",
            "space_id": "sp", "slug": "just-a-slug"
        }),
    );
    let map = project(&[bare_started, session_completed("bare", 1)]);
    assert!(map.is_empty(), "bare session must not seed step rows");
}

// ── The 3 formerly-dead subscriptions ──────────────────────────────────────────

#[test]
fn done_declared_completes_step_and_records_summary() {
    let run = "run-dd";
    let declared = ev(
        pet::PIPELINE_STEP_DONE_DECLARED,
        2,
        serde_json::json!({
            "pipeline_run_id": run, "slug": "myrun",
            "step_id": "a", "summary": "implemented the parser"
        }),
    );
    let map = project(&[run_started(run, 0), step_started(run, "a", 1), declared]);
    let a = row(&map, run, "a");
    assert_eq!(a.status, StepRunStatus::Completed, "done_declared -> step status");
    assert_eq!(a.completed_at, Some(at(2)));
    assert_eq!(a.note.as_deref(), Some("implemented the parser"));
}

#[test]
fn step_output_nudges_pending_step_to_running() {
    let run = "run-out";
    let output = ev(
        pet::PIPELINE_STEP_OUTPUT,
        1,
        serde_json::json!({
            "pipeline_run_id": run, "step_id": "c",
            "stdout": "running tests...", "stderr": null
        }),
    );
    // run.started seeds c as pending; output (no prior step.started) flips it.
    let map = project(&[run_started(run, 0), output]);
    let c = row(&map, run, "c");
    assert_eq!(c.status, StepRunStatus::Running);
    assert_eq!(c.started_at, Some(at(1)));
}

// ── Retry / attempt ─────────────────────────────────────────────────────────────

#[test]
fn retry_bumps_attempt_and_reopens_lifecycle() {
    let run = "run-retry";
    let retry = ev(
        pet::PIPELINE_STEP_RETRY,
        3,
        serde_json::json!({
            "pipeline_run_id": run, "slug": "myrun",
            "step_id": "a", "step_name": "a", "attempt": 2, "reason": "gate failed"
        }),
    );
    let map = project(&[
        run_started(run, 0),
        step_started(run, "a", 1),
        step_completed(run, "a", 2),
        retry,
        step_started(run, "a", 4),
    ]);
    let a = row(&map, run, "a");
    assert_eq!(a.attempt, 2);
    // After retry + a fresh start, the step is running again with a new start.
    assert_eq!(a.status, StepRunStatus::Running);
    assert_eq!(a.started_at, Some(at(4)));
    assert_eq!(a.completed_at, None, "retry reopened the lifecycle");
}

// ── DAG seeding ─────────────────────────────────────────────────────────────────

#[test]
fn run_started_seeds_pending_rows_with_dag_edges() {
    let run = "run-seed";
    let map = project(&[run_started(run, 0)]);
    assert_eq!(map.len(), 3, "one row per DAG step");
    for (id, expect_dep) in [("a", None), ("b", Some(vec!["a".to_string()])), ("c", Some(vec!["b".to_string()]))] {
        let r = row(&map, run, id);
        assert_eq!(r.status, StepRunStatus::Pending);
        assert!(r.started_at.is_none());
        assert!(r.completed_at.is_none());
        assert_eq!(r.depends_on, expect_dep);
    }
}

#[test]
fn fanout_step_is_one_row_for_now() {
    // P0.1 defers per-lane fanout rows: a fanout step is a single row.
    let run = "run-fan";
    let started = ev(
        pet::PIPELINE_RUN_STARTED,
        0,
        serde_json::json!({
            "pipeline_run_id": run, "slug": "myrun", "agent": "x",
            "branch": "b", "spec_path": "s",
            "steps": { "steps": [{
                "id": "explore", "type": "agent",
                "fanout": {
                    "lanes": [
                        { "id": "l1", "agent": "a1" },
                        { "id": "l2", "agent": "a2" }
                    ],
                    "judge": { "step": "judge" }
                }
            }], "auto_merge": false }
        }),
    );
    let map = project(&[started]);
    assert_eq!(map.len(), 1, "fanout step folds to a single row (lanes deferred)");
    assert!(map.contains_key(&(run.to_string(), "explore".to_string())));
}

// ── Idempotent replay ───────────────────────────────────────────────────────────

#[test]
fn replaying_the_stream_is_idempotent() {
    let run = "run-idem";
    let events = vec![
        run_started(run, 0),
        step_started(run, "a", 1),
        step_completed(run, "a", 2),
    ];
    let once = project(&events);
    let twice = project(&[events.clone(), events].concat());
    assert_eq!(once.len(), twice.len());
    let a1 = row(&once, run, "a");
    let a2 = row(&twice, run, "a");
    assert_eq!(a1.status, a2.status);
    assert_eq!(a1.started_at, a2.started_at);
    assert_eq!(a1.completed_at, a2.completed_at);
}

// ── SQL round-trip (hermetic temp db) ──────────────────────────────────────────

/// Fold the primary sequence, persist every row to a fresh temp runtime.db
/// (this also exercises the new migration), read them back, and assert the row
/// fields survive the SQL serialization round-trip.
#[test]
fn sql_round_trip_persists_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let mut conn = ship_substrate::db::open_runtime_db(tmp.path()).unwrap();

    let run = "run-sql";
    let events = vec![
        run_started(run, 0),
        step_started(run, "a", 1),
        step_completed(run, "a", 2),
        session_started(run, "b", "sess-sql-b", 3),
        session_completed("sess-sql-b", 4),
        step_failed(run, "c", "boom", 5),
    ];
    let map = project(&events);
    for r in map.values() {
        upsert_row(r, &mut conn).unwrap();
    }

    let loaded = load_rows_for_run(run, &mut conn).unwrap();
    assert_eq!(loaded.len(), 3);

    let b = loaded.get(&(run.to_string(), "b".to_string())).unwrap();
    assert_eq!(b.status, StepRunStatus::Completed);
    assert_eq!(b.started_at, Some(at(3)));
    assert_eq!(b.completed_at, Some(at(4)));
    assert_eq!(b.session_id.as_deref(), Some("sess-sql-b"));
    assert_eq!(b.depends_on, Some(vec!["a".to_string()]));

    let c = loaded.get(&(run.to_string(), "c".to_string())).unwrap();
    assert_eq!(c.status, StepRunStatus::Failed);
    assert_eq!(c.step_type, "run");
    assert_eq!(c.note.as_deref(), Some("boom"));
}

// ── Projection apply() path (resolves run_id, incl. session-terminal lookup) ───

/// Drive the full sequence through `PipelineStepRunProjection::apply` against
/// the thread-isolated global runtime.db. This exercises the SQL run_id
/// resolution — in particular `run_id_for_session`, which recovers the run for
/// a `session.completed` (whose payload carries only `session_id`) from the row
/// `session.started` stamped. A unique run/session id keeps the assertion
/// independent of other rows in the shared per-thread test db.
#[test]
fn projection_apply_resolves_session_terminal_to_step() {
    let proj = PipelineStepRunProjection;
    let mut conn = open_default_conn().unwrap();
    let run = "run-apply-9f3c";
    let sess = "sess-apply-9f3c";

    for e in [
        run_started(run, 0),
        session_started(run, "b", sess, 1),
        session_completed(sess, 2),
    ] {
        proj.apply(&e, &mut conn).unwrap();
    }

    let rows = load_step_runs_for_run(run).unwrap();
    let b = rows
        .iter()
        .find(|r| r.step_id == "b")
        .expect("step b row must exist");
    assert_eq!(
        b.status,
        StepRunStatus::Completed,
        "session.completed must resolve back to step b through run_id_for_session"
    );
    assert_eq!(b.session_id.as_deref(), Some(sess));
    assert_eq!(b.completed_at, Some(at(2)));
}
