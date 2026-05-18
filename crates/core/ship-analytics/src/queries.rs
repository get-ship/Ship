//! Typed SELECT helpers over `analytics_sessions` and `analytics_tool_uses`.
//!
//! Dashboard / CLI / HTTP-endpoint code should call these instead of
//! hand-rolling SQL. Keeping queries in one module lets us:
//!   - share row types
//!   - centralise filter shapes (pagination, time windows, harness filter)
//!   - hold the line on "no n+1 — fetch the join in one query"
//!
//! Today: minimal surface (just enough for the projection unit tests
//! to dogfood their own consumer). Will grow as the dashboard story
//! firms up.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;

/// Top-level summary of one harness session — what the dashboard
/// renders as a row in the sessions table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_key: String,
    pub actor_id: String,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_ms: Option<i64>,
    pub tool_use_count: i64,
    pub tool_error_count: i64,
    pub reason: Option<String>,
}

/// Fetch one session by its primary key. `None` if not present.
pub fn get_session(
    conn: &mut SqliteConnection,
    session_key: &str,
) -> Result<Option<SessionSummary>> {
    block_on(async {
        let row: Option<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            Option<i64>,
            i64,
            i64,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT session_key, actor_id, harness, model, started_at, ended_at, \
                    duration_ms, tool_use_count, tool_error_count, reason \
             FROM analytics_sessions WHERE session_key = ?",
        )
        .bind(session_key)
        .fetch_optional(&mut *conn)
        .await?;
        Ok(row.map(row_to_session))
    })
}

/// List most-recent sessions, newest start first. `limit` capped at 500
/// to bound the response.
pub fn list_recent_sessions(
    conn: &mut SqliteConnection,
    limit: i64,
) -> Result<Vec<SessionSummary>> {
    let cap = limit.clamp(1, 500);
    block_on(async {
        let rows: Vec<(
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            Option<String>,
            Option<i64>,
            i64,
            i64,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT session_key, actor_id, harness, model, started_at, ended_at, \
                    duration_ms, tool_use_count, tool_error_count, reason \
             FROM analytics_sessions ORDER BY started_at DESC LIMIT ?",
        )
        .bind(cap)
        .fetch_all(&mut *conn)
        .await?;
        Ok(rows.into_iter().map(row_to_session).collect())
    })
}

/// Per-tool aggregate across all sessions in the given time window
/// (RFC3339 timestamps). `None` for either bound = open-ended.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsage {
    pub tool: String,
    pub calls: i64,
    pub errors: i64,
    /// Mean duration across successful calls. None when no completed calls.
    pub avg_duration_ms: Option<i64>,
}

pub fn tool_usage(
    conn: &mut SqliteConnection,
    since: Option<&str>,
    until: Option<&str>,
) -> Result<Vec<ToolUsage>> {
    block_on(async {
        let rows: Vec<(String, i64, i64, Option<f64>)> = sqlx::query_as(
            "SELECT tool, \
                    COUNT(*) AS calls, \
                    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) AS errors, \
                    AVG(duration_ms) AS avg_ms \
             FROM analytics_tool_uses \
             WHERE (?1 IS NULL OR started_at >= ?1) \
               AND (?2 IS NULL OR started_at <= ?2) \
             GROUP BY tool ORDER BY calls DESC",
        )
        .bind(since)
        .bind(until)
        .fetch_all(&mut *conn)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(tool, calls, errors, avg)| ToolUsage {
                tool,
                calls,
                errors,
                avg_duration_ms: avg.map(|f| f as i64),
            })
            .collect())
    })
}

fn row_to_session(
    r: (
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
        Option<i64>,
        i64,
        i64,
        Option<String>,
    ),
) -> SessionSummary {
    SessionSummary {
        session_key: r.0,
        actor_id: r.1,
        harness: r.2,
        model: r.3,
        started_at: r.4,
        ended_at: r.5,
        duration_ms: r.6,
        tool_use_count: r.7,
        tool_error_count: r.8,
        reason: r.9,
    }
}
