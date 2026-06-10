//! Pipeline-run Watch prefilters + scope keys + snapshots — the pure, testable
//! half of the typed `PipelinesService.Watch` (pipelines P0.4).
//!
//! The shipd `watch()` handler composes three pieces: a `ReactiveRegistry`
//! (ship-reactive, owns the event loop + diff + fanout), a snapshot closure
//! (the P0.3 typed run-reads), and the cheap event prefilter that decides
//! whether a kernel event should re-run the snapshot. THIS module owns the
//! prefilter + scope-key + snapshot logic — kept here in ship-pipeline-runtime
//! so it is unit-testable WITHOUT the shipd build wall, mirroring
//! `ship_platform::services::agents::watch`.
//!
//! Two scopes:
//!   * ALL  — snapshots the run-level summary rows (`PipelineRunSummariesResponse`
//!     shape). Wakes on any run-lifecycle `pipeline.*` event (run/step/gate) —
//!     a change to ANY run can move a summary row.
//!   * RUN(id) — snapshots one run's detail (run row + step rows,
//!     `PipelineRunDetailResponse` shape). Wakes on the `pipeline.*` / `session.*`
//!     events carrying this `pipeline_run_id` (prefilter isolation), so another
//!     run's event does not patch this subscriber.
//!
//! Why the projections feed it: the run-level rows fold from `pipeline.*` (the
//! `pipeline_runs` projection) and the step rows fold from `pipeline.*` +
//! `session.*` (the `pipeline_step_runs` projection). The prefilters below name
//! exactly those event surfaces so the Watch wakes when — and only when — a fold
//! input for its scope changes.

use std::path::Path;

use serde_json::Value;

use agent_def::canonical_session::event_types as session_event_types;
use ship_substrate::events::EventEnvelope;

/// Scope-key fragment for the `WatchKey` of the ALL scope. The full key is
/// `(space_id, scope, query_hash)`; the daemon prepends the bound space_id.
pub const SCOPE_ALL: &str = "pipelines:all";

/// Build the scope-key fragment for a single-run watch.
pub fn scope_run(run_id: &str) -> String {
    format!("pipeline_run:{run_id}")
}

// ── Prefilters ─────────────────────────────────────────────────────────────────

/// Is this a run-lifecycle event (`pipeline.run.*` / `pipeline.step.*` /
/// `pipeline.gate.*`)? These are exactly the events the `pipeline_runs` /
/// `pipeline_step_runs` projections fold — i.e. the ones that can move a run row
/// or a step row. Deliberately EXCLUDES `pipeline.changed` (a pipeline DEFINITION
/// edit, not a run-state change).
fn is_run_lifecycle_event(event_type: &str) -> bool {
    event_type.starts_with("pipeline.run.")
        || event_type.starts_with("pipeline.step.")
        || event_type.starts_with("pipeline.gate.")
}

/// Is this a canonical child-session terminal (`session.completed/failed/
/// cancelled`)? These carry only `session_id` — not the run id — so a cheap
/// prefilter cannot attribute them to a run. They drive composed-step completion
/// (the `pipeline_step_runs` child-session anchor), so the run scope wakes on
/// them broadly; the snapshot+diff then filters: an unrelated run's terminal
/// leaves this run's snapshot unchanged → no patch frame.
fn is_session_terminal(event_type: &str) -> bool {
    event_type == session_event_types::SESSION_COMPLETED
        || event_type == session_event_types::SESSION_FAILED
        || event_type == session_event_types::SESSION_CANCELLED
}

/// ALL-scope prefilter: does this event possibly move a run-level summary row?
/// True for any run-lifecycle `pipeline.*` event (see [`is_run_lifecycle_event`]).
pub fn all_scope_relevant(env: &EventEnvelope) -> bool {
    is_run_lifecycle_event(&env.event_type)
}

/// RUN-scope prefilter: is this event relevant to THIS run's detail snapshot?
///
/// Matches:
///   * a run-lifecycle `pipeline.*` event whose payload `pipeline_run_id` == `run_id`
///     (the run row + most step rows carry it), and
///   * `session.started` whose payload `parent` == `run_id` (the composed-step
///     session that step rows are keyed back through), and
///   * any `session.*` terminal (broad wake — see [`is_session_terminal`]).
///
/// Another run's `pipeline.*` event is ignored (prefilter isolation): its
/// `pipeline_run_id` differs.
pub fn run_scope_relevant(env: &EventEnvelope, run_id: &str) -> bool {
    let et = env.event_type.as_str();

    if is_session_terminal(et) {
        return true;
    }
    if et == session_event_types::SESSION_STARTED {
        return payload_field_eq(env, "parent", run_id);
    }
    if is_run_lifecycle_event(et) {
        return payload_field_eq(env, "pipeline_run_id", run_id);
    }
    false
}

/// Cheap payload-field equality check without fully typing the payload.
fn payload_field_eq(env: &EventEnvelope, field: &str, expected: &str) -> bool {
    serde_json::from_str::<Value>(&env.payload_json)
        .ok()
        .and_then(|p| p.get(field).and_then(|v| v.as_str()).map(str::to_string))
        .map(|got| got == expected)
        .unwrap_or(false)
}

// ── Snapshots (the P0.3 typed read messages, as JSON Value) ─────────────────────
//
// The Watch streams the SAME typed messages as the P0.3 reads. The resolver
// holds `serde_json::Value` and diffs it; these builders serialize the typed
// response so the patch values decode straight back into the proto messages.

/// ALL-scope snapshot — the run-level summary rows for every run.
pub fn all_scope_snapshot(ship_dir: &Path) -> Value {
    serde_json::to_value(crate::read_core::summaries(ship_dir)).unwrap_or(Value::Null)
}

/// RUN-scope snapshot — one run's detail (run row + step rows), `Null` when the
/// run id is unknown.
pub fn run_scope_snapshot(ship_dir: &Path, run_id: &str) -> Value {
    crate::read_core::run_detail_by_id(ship_dir, run_id)
        .and_then(|d| serde_json::to_value(d).ok())
        .unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use ship_substrate::events::pipeline::event_types as pipeline_event_types;

    fn env(event_type: &str, payload: serde_json::Value) -> EventEnvelope {
        EventEnvelope::new(event_type, "entity", &payload).unwrap()
    }

    #[test]
    fn scope_keys_are_distinct_per_run() {
        assert_eq!(scope_run("r1"), "pipeline_run:r1");
        assert_ne!(scope_run("r1"), scope_run("r2"));
        assert_ne!(SCOPE_ALL, scope_run("r1"));
    }

    #[test]
    fn all_scope_wakes_on_run_and_step_lifecycle() {
        assert!(all_scope_relevant(&env(
            pipeline_event_types::PIPELINE_RUN_STARTED,
            json!({ "pipeline_run_id": "r1" }),
        )));
        assert!(all_scope_relevant(&env(
            pipeline_event_types::PIPELINE_STEP_COMPLETED,
            json!({ "pipeline_run_id": "r1", "step_id": "a" }),
        )));
        assert!(all_scope_relevant(&env(
            pipeline_event_types::PIPELINE_RUN_MERGED,
            json!({ "pipeline_run_id": "r1" }),
        )));
    }

    #[test]
    fn all_scope_ignores_pipeline_def_change_and_unrelated() {
        // pipeline.changed is a DEFINITION edit, not run state — must not wake.
        assert!(!all_scope_relevant(&env("pipeline.changed", json!({}))));
        assert!(!all_scope_relevant(&env("session.started", json!({}))));
        assert!(!all_scope_relevant(&env("process.spawned", json!({}))));
    }

    #[test]
    fn run_scope_matches_only_its_own_pipeline_events() {
        let e = env(
            pipeline_event_types::PIPELINE_STEP_STARTED,
            json!({ "pipeline_run_id": "r1", "step_id": "a" }),
        );
        assert!(run_scope_relevant(&e, "r1"));
        assert!(!run_scope_relevant(&e, "r2"), "another run's step event must not wake this subscriber");
    }

    #[test]
    fn run_scope_matches_run_lifecycle_for_its_id() {
        let e = env(pipeline_event_types::PIPELINE_RUN_COMPLETED, json!({ "pipeline_run_id": "r1" }));
        assert!(run_scope_relevant(&e, "r1"));
        assert!(!run_scope_relevant(&e, "other"));
    }

    #[test]
    fn run_scope_matches_session_started_by_parent() {
        let e = env(
            session_event_types::SESSION_STARTED,
            json!({ "parent": "r1", "slug": "myrun::a", "session_id": "s1" }),
        );
        assert!(run_scope_relevant(&e, "r1"));
        assert!(!run_scope_relevant(&e, "r2"), "another run's child session must not wake this subscriber");
    }

    #[test]
    fn run_scope_wakes_broadly_on_session_terminals() {
        // Terminals carry only session_id — the run scope wakes on them and lets
        // the snapshot+diff decide (the child-session completion anchor).
        let e = env(session_event_types::SESSION_COMPLETED, json!({ "session_id": "s1" }));
        assert!(run_scope_relevant(&e, "r1"));
        assert!(run_scope_relevant(&e, "any-run"));
    }

    #[test]
    fn run_scope_ignores_non_pipeline_non_session() {
        let e = env("process.spawned", json!({ "pipeline_run_id": "r1" }));
        assert!(!run_scope_relevant(&e, "r1"));
        assert!(!run_scope_relevant(&env("pipeline.changed", json!({})), "r1"));
    }
}
