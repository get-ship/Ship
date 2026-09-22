//! T4-lite replay honesty for the Windowed retention tier.
//!
//! The ruled corollary: folds over Windowed types are bounded by the
//! retention horizon — a projection folding a Windowed type must yield
//! correct state from a swept log. This test proves it for the actual
//! projections of the re-triaged types (AnalyticsProjection over
//! `op.tool_use_*` + `session.turn.ended`; CanonicalSessionProjection over
//! `session.turn.*`): a swept log rebuilds to EXACTLY the projection state
//! of a log that only ever contained the surviving window, including the
//! horizon-orphan case (a completion whose start was swept).
//!
//! `dispatch.requested`, `sync.push.completed`, and `op.tool_use_denied`
//! have no projection — nothing folds them, so sweep affects only their
//! event rows (covered by the substrate sweep tests).

use ship_types::Stamped;
use chrono::{Duration as ChronoDuration, Utc};
use tempfile::tempdir;

use serde_json::json;
use ship_substrate::db::{self, retention};
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_types::envelope::EventEnvelope;

/// Envelope backdated `age_days`; payload timestamps derive from the same
/// clock so both logs fold byte-identical values.
fn aged<P: serde::Serialize>(
    event_type: &str,
    entity: &str,
    payload: &P,
    age_days: i64,
) -> EventEnvelope {
    let mut e = EventEnvelope::new(event_type, entity, payload).expect("envelope builds");
    e.created_at = Utc::now() - ChronoDuration::days(age_days);
    e
}

fn iso(age_days: i64) -> String {
    (Utc::now() - ChronoDuration::days(age_days)).to_rfc3339()
}

fn started(session: &str, tool: &str, age_days: i64) -> EventEnvelope {
    aged(
        "op.tool_use_started",
        session,
        &json!({ "session_id": session, "tool": tool, "started_at": iso(age_days) }),
        age_days,
    )
}

fn completed(session: &str, tool: &str, status: &str, age_days: i64) -> EventEnvelope {
    aged(
        "op.tool_use_completed",
        session,
        &json!({ "session_id": session, "tool": tool, "status": status, "ended_at": iso(age_days) }),
        age_days,
    )
}

fn turn_ended(session: &str, idx: u32, age_days: i64) -> EventEnvelope {
    aged(
        "session.turn.ended",
        session,
        &json!({ "session_id": session, "idx": idx }),
        age_days,
    )
}

fn session_started(session: &str, age_days: i64) -> EventEnvelope {
    aged(
        "session.started",
        session,
        &json!({
            "session_id": session,
            "agent_id": "claude",
            "harness": "floki",
            "space_id": "space-1"
        }),
        age_days,
    )
}

type Rows = Vec<Vec<Option<String>>>;

/// Dump a table's meaningful columns as strings, deterministically ordered.
fn dump(db_path: &std::path::Path, sql: &str) -> Rows {
    use sqlx::{Column as _, Row};
    let mut conn = db::open_db_at(db_path).unwrap();
    let rows = db::block_on(async { sqlx::query(sql).fetch_all(&mut conn).await }).unwrap();
    rows.iter()
        .map(|r| {
            (0..r.columns().len())
                .map(|i| {
                    r.try_get::<Option<String>, _>(i).unwrap_or_else(|_| {
                        r.try_get::<Option<i64>, _>(i)
                            .unwrap()
                            .map(|v| v.to_string())
                    })
                })
                .collect()
        })
        .collect()
}

fn event_ids(db_path: &std::path::Path) -> Vec<String> {
    dump(db_path, "SELECT id FROM events ORDER BY id")
        .into_iter()
        .map(|r| r[0].clone().unwrap())
        .collect()
}

const CANONICAL_SQL: &str = "SELECT session_id, status, CAST(turn_count AS TEXT), started_at, ended_at, outcome FROM canonical_sessions ORDER BY session_id";
const A_SESSIONS_SQL: &str = "SELECT session_key, actor_id, harness, model, started_at, ended_at, CAST(tool_use_count AS TEXT), CAST(tool_error_count AS TEXT) FROM analytics_sessions ORDER BY session_key";
const A_TOOL_USES_SQL: &str = "SELECT id, session_key, tool, status, started_at, ended_at, CAST(duration_ms AS TEXT), error FROM analytics_tool_uses ORDER BY id";
const A_TURNS_SQL: &str =
    "SELECT id, session_key, CAST(turn_idx AS TEXT) FROM analytics_turns ORDER BY id";

#[test]
fn swept_log_rebuilds_to_the_window_state_for_retriaged_projections() {
    let factory = ship_platform::platform_projection_factory();
    let now = Utc::now();

    // Fixture invariant: the re-triaged types really are Windowed in this
    // (fully domain-linked) binary, so the registry-driven sweep sees them.
    for name in [
        "op.tool_use_started",
        "op.tool_use_completed",
        "session.turn.started",
        "session.turn.ended",
    ] {
        let def = ship_types::event_registry::lookup(name).expect("registered");
        assert_eq!(
            def.retention.ttl_days(),
            Some(7),
            "{name} must be Windowed 7d"
        );
    }

    let tmp_a = tempdir().unwrap();
    let tmp_b = tempdir().unwrap();
    let db_a = db::runtime_db_path(tmp_a.path());
    let db_b = db::runtime_db_path(tmp_b.path());
    let store_a = SqliteEventStore::with_projection_factory(&db_a, &factory).unwrap();
    let store_b = SqliteEventStore::with_projection_factory(&db_b, &factory).unwrap();

    // The full history, minted in chronological order. `in_window` marks
    // what the sweep must leave: every Durable event + Windowed events
    // younger than the 7d horizon.
    let log: Vec<(EventEnvelope, bool)> = vec![
        // s-old: ended long ago; its turn/op history sits past the horizon.
        (session_started("s-old", 10), true), // Durable — survives at any age
        (started("s-old", "Bash", 10), false),
        (completed("s-old", "Bash", "ok", 10), false),
        (turn_ended("s-old", 0, 10), false),
        (turn_ended("s-old", 1, 9), false),
        (
            aged(
                "session.completed",
                "s-old",
                &json!({ "session_id": "s-old" }),
                9,
            ),
            true,
        ),
        // s-live: started long ago, still running across the horizon.
        (session_started("s-live", 10), true),
        (turn_ended("s-live", 0, 8), false),
        // Horizon orphan: the start is swept, the completion survives.
        (started("s-live", "Read", 8), false),
        (completed("s-live", "Read", "ok", 6), true),
        (turn_ended("s-live", 1, 1), true),
        (started("s-live", "Bash", 1), true),
        (completed("s-live", "Bash", "error", 1), true),
    ];

    for (e, in_window) in &log {
        store_a.append(&Stamped::unchecked(e.clone())).unwrap();
        if *in_window {
            store_b.append(&Stamped::unchecked(e.clone())).unwrap();
        }
    }

    // Sweep A through the production registry-driven entry.
    let report = retention::sweep_windowed_events_at(tmp_a.path(), now).unwrap();
    assert_eq!(
        report.rows_deleted(),
        6,
        "exactly the out-of-horizon windowed rows are swept"
    );

    // The swept log IS the window-only log.
    assert_eq!(
        event_ids(&db_a),
        event_ids(&db_b),
        "sweep leaves exactly durables + the window"
    );

    // Rebuild both; every projection of the re-triaged types must land in
    // identical state.
    db::rebuild::rebuild_scope_db(&db_a).unwrap();
    db::rebuild::rebuild_scope_db(&db_b).unwrap();

    for sql in [CANONICAL_SQL, A_SESSIONS_SQL, A_TOOL_USES_SQL, A_TURNS_SQL] {
        assert_eq!(
            dump(&db_a, sql),
            dump(&db_b, sql),
            "swept-log rebuild must equal the window-only rebuild ({sql})"
        );
    }

    // Bounded-fold spot checks on the rebuilt state.
    let canonical = dump(&db_a, CANONICAL_SQL);
    assert_eq!(canonical.len(), 2);
    // s-live: Durable session.started keeps the row; turn_count counts the
    // surviving window only.
    assert_eq!(canonical[0][0].as_deref(), Some("s-live"));
    assert_eq!(canonical[0][1].as_deref(), Some("awaiting_input"));
    assert_eq!(
        canonical[0][2].as_deref(),
        Some("1"),
        "turn_count = surviving turns"
    );
    // s-old: terminal state intact from Durable events alone.
    assert_eq!(canonical[1][0].as_deref(), Some("s-old"));
    assert_eq!(canonical[1][1].as_deref(), Some("completed"));
    assert_eq!(
        canonical[1][2].as_deref(),
        Some("0"),
        "swept turns fold to zero"
    );

    // The horizon-orphan completion folded as a closed row (counted, not
    // dropped, no crash), and the in-window pair folded normally.
    let tool_uses = dump(&db_a, A_TOOL_USES_SQL);
    assert_eq!(tool_uses.len(), 2, "orphan completion + recent pair");
    let by_tool: std::collections::HashMap<String, Vec<Option<String>>> = tool_uses
        .into_iter()
        .map(|r| (r[2].clone().unwrap(), r))
        .collect();
    let read = &by_tool["Read"];
    assert_eq!(read[3].as_deref(), Some("ok"));
    assert_eq!(
        read[4], read[5],
        "orphaned completion closes at its own timestamp"
    );
    let bash = &by_tool["Bash"];
    assert_eq!(bash[3].as_deref(), Some("error"));
    assert!(
        bash[4] < bash[5],
        "paired start/completion keeps real bounds"
    );
}
