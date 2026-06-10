//! R18 behavioral tests for the typed run-level read contract (P0.3).
//!
//! Two layers, both asserting on the TYPED response messages (never source text):
//!   1. Mapper tests — fold events through the pure projections, map the
//!      resulting records to the wire rows, assert the DOMAIN fields
//!      (status/started_at/ended_at/outcome, step id/status/timestamps/depends_on).
//!   2. Verb tests — seed raw events into a hermetic temp `runtime.db`, call the
//!      actual read verb (`summaries`/`run_detail`/`history`/`list_runs`), and
//!      assert on the typed response it builds off the hydrated projections.

use super::*;
use chrono::{DateTime, TimeZone, Utc};
use serde_json::json;
use std::path::Path;

use crate::projections::pipeline_run::project as project_runs;
use crate::projections::pipeline_step_run::project as project_steps;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types as pet;

// ── event builders (deterministic timestamps) ────────────────────────────────

fn at(secs: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(1_700_000_000 + secs, 0).single().unwrap()
}

fn ev(event_type: &str, ts: i64, payload: serde_json::Value) -> EventEnvelope {
    let entity = payload["pipeline_run_id"].as_str().unwrap_or("entity");
    let mut e = EventEnvelope::new(event_type, entity, &payload).unwrap();
    e.created_at = at(ts);
    e
}

fn run_started(run: &str, slug: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_RUN_STARTED,
        ts,
        json!({
            "pipeline_run_id": run, "slug": slug, "agent": "rust-lane",
            "branch": "job/x", "spec_path": "s",
            "steps": { "steps": [
                { "id": "a", "type": "agent" },
                { "id": "b", "type": "agent", "after": ["a"] }
            ], "auto_merge": false }
        }),
    )
}

fn dispatched(run: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_RUN_DISPATCHED,
        ts,
        json!({ "pipeline_run_id": run, "slug": "", "worktree": "/tmp/wt", "pid": 1 }),
    )
}

fn merged(run: &str, ts: i64) -> EventEnvelope {
    ev(pet::PIPELINE_RUN_MERGED, ts, json!({ "pipeline_run_id": run, "slug": "" }))
}

fn step_started(run: &str, step: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_STEP_STARTED,
        ts,
        json!({ "pipeline_run_id": run, "slug": "myrun", "step_id": step, "step_type": "agent" }),
    )
}

fn step_completed(run: &str, step: &str, ts: i64) -> EventEnvelope {
    ev(
        pet::PIPELINE_STEP_COMPLETED,
        ts,
        json!({ "pipeline_run_id": run, "slug": "myrun", "step_id": step }),
    )
}

/// Write raw event rows into a hermetic temp `runtime.db`. The typed reads
/// hydrate `pipeline_runs` / `pipeline_step_runs` from these events on first
/// load (no orchestrator, no global state).
fn seed(ship_dir: &Path, events: &[EventEnvelope]) {
    use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
    // Materialize the full runtime schema first, then append the event rows.
    drop(ship_substrate::db::open_runtime_db(ship_dir).unwrap());
    let store =
        SqliteEventStore::new_at_db_path(&ship_substrate::db::runtime_db_path(ship_dir)).unwrap();
    for e in events {
        EventStore::append(&store, e).unwrap();
    }
}

// ── mapper tests ─────────────────────────────────────────────────────────────

#[test]
fn run_row_terminal_carries_ended_and_outcome() {
    let run = "r1";
    let map = project_runs(&[run_started(run, "myrun", 0), dispatched(run, 1), merged(run, 5)]);
    let row = run_row_of(map.get(run).unwrap());

    assert_eq!(row.pipeline_run_id, "r1");
    assert_eq!(row.slug, "myrun");
    assert_eq!(row.status, "merged");
    assert_eq!(row.started_at, at(0).to_rfc3339());
    assert_eq!(row.ended_at.as_deref(), Some(at(5).to_rfc3339().as_str()));
    assert_eq!(row.outcome.as_deref(), Some("merged"));
}

#[test]
fn run_row_live_has_no_ended_or_outcome() {
    let run = "r2";
    let map = project_runs(&[run_started(run, "live", 0), dispatched(run, 2)]);
    let row = run_row_of(map.get(run).unwrap());

    assert_eq!(row.status, "dispatched");
    assert_eq!(row.started_at, at(0).to_rfc3339());
    assert!(row.ended_at.is_none(), "a live run has no ended_at");
    assert!(row.outcome.is_none(), "a live run has no outcome");
}

#[test]
fn step_rows_carry_domain_fields() {
    let run = "r3";
    let steps = project_steps(&[
        run_started(run, "myrun", 0),
        step_started(run, "a", 1),
        step_completed(run, "a", 2),
    ]);

    let a = step_row_of(steps.get(&(run.to_string(), "a".to_string())).unwrap());
    assert_eq!(a.step_id, "a");
    assert_eq!(a.status, "completed");
    assert_eq!(a.started_at.as_deref(), Some(at(1).to_rfc3339().as_str()));
    assert_eq!(a.completed_at.as_deref(), Some(at(2).to_rfc3339().as_str()));
    assert!(a.depends_on.is_empty(), "root step has no DAG edge");

    let b = step_row_of(steps.get(&(run.to_string(), "b".to_string())).unwrap());
    assert_eq!(b.status, "pending", "unstarted step is pending");
    assert_eq!(b.depends_on, vec!["a".to_string()], "DAG edge preserved");
}

// ── verb tests (against a seeded projection) ─────────────────────────────────

#[test]
fn summaries_verb_returns_typed_run_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    seed(
        dir,
        &[
            run_started("r1", "alpha", 0),
            dispatched("r1", 1),
            run_started("r2", "beta", 2),
            dispatched("r2", 3),
            merged("r2", 6),
        ],
    );

    let resp = summaries(dir);
    assert_eq!(resp.runs.len(), 2);
    // Newest first by creation: beta (ts 2) precedes alpha (ts 0).
    assert_eq!(resp.runs[0].slug, "beta");

    let beta = resp.runs.iter().find(|r| r.slug == "beta").unwrap();
    assert_eq!(beta.status, "merged");
    assert_eq!(beta.outcome.as_deref(), Some("merged"));

    let alpha = resp.runs.iter().find(|r| r.slug == "alpha").unwrap();
    assert_eq!(alpha.status, "dispatched");
    assert!(alpha.outcome.is_none());
}

#[test]
fn run_detail_verb_returns_run_plus_step_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    seed(
        dir,
        &[
            run_started("r1", "myrun", 0),
            step_started("r1", "a", 1),
            step_completed("r1", "a", 2),
            step_started("r1", "b", 3),
        ],
    );

    let detail = run_detail(dir, "myrun").expect("run with this slug exists");
    assert_eq!(detail.run.slug, "myrun");
    assert_eq!(detail.steps.len(), 2);

    let a = detail.steps.iter().find(|s| s.step_id == "a").unwrap();
    assert_eq!(a.status, "completed");
    let b = detail.steps.iter().find(|s| s.step_id == "b").unwrap();
    assert_eq!(b.status, "running");
    assert_eq!(b.depends_on, vec!["a".to_string()]);
}

#[test]
fn run_detail_verb_unknown_slug_is_none() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    seed(dir, &[run_started("r1", "myrun", 0)]);
    assert!(run_detail(dir, "ghost").is_none());
}

#[test]
fn history_verb_returns_terminal_runs_only() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    seed(
        dir,
        &[
            run_started("r1", "alpha", 0),
            dispatched("r1", 1), // active — excluded from history
            run_started("r2", "beta", 2),
            dispatched("r2", 3),
            merged("r2", 6), // terminal — in history
        ],
    );

    let resp = history(dir, None, None, 0, 50);
    assert_eq!(resp.runs.len(), 1, "only terminal-category runs are history");
    assert_eq!(resp.runs[0].slug, "beta");
    assert_eq!(resp.runs[0].status, "merged");
    assert_eq!(resp.total, 1);
    assert_eq!(resp.limit, 50);
}

#[test]
fn list_runs_verb_returns_runs_with_step_rows() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    seed(
        dir,
        &[
            run_started("r1", "myrun", 0),
            step_started("r1", "a", 1),
            step_completed("r1", "a", 2),
        ],
    );

    let resp = list_runs(dir);
    assert_eq!(resp.runs.len(), 1);
    let d = &resp.runs[0];
    assert_eq!(d.run.slug, "myrun");
    assert!(
        d.steps.iter().any(|s| s.step_id == "a" && s.status == "completed"),
        "list_runs carries step rows off pipeline_step_runs"
    );
}
