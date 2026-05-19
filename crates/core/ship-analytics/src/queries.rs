// SQLx returns large positional tuples for raw fetch; refactoring each
// into a typed struct is a separate cleanup. Allow locally for now.
#![allow(clippy::type_complexity)]

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

/// One row from analytics_turns — per-turn token observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnObservation {
    pub id: String,
    pub session_key: String,
    pub actor_id: String,
    pub turn_idx: Option<i64>,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    pub stop_reason: Option<String>,
    /// Cost as REPORTED by the harness, in USD. None when the harness
    /// didn't surface it. Captured verbatim — Ship never computes
    /// cost.
    pub harness_reported_cost_usd: Option<f64>,
    pub observed_at: String,
}

/// Per-session turn history, oldest first. Useful for rendering a
/// chronological view of token usage within one session.
pub fn list_session_turns(
    conn: &mut SqliteConnection,
    session_key: &str,
) -> Result<Vec<TurnObservation>> {
    block_on(async {
        let rows: Vec<(
            String, String, String, Option<i64>, Option<String>,
            i64, i64, i64, i64, i64, Option<String>, Option<f64>, String,
        )> = sqlx::query_as(
            "SELECT id, session_key, actor_id, turn_idx, model, \
                    input_tokens, output_tokens, reasoning_tokens, \
                    cache_creation_tokens, cache_read_tokens, \
                    stop_reason, harness_reported_cost_usd, observed_at \
             FROM analytics_turns WHERE session_key = ? \
             ORDER BY observed_at ASC",
        )
        .bind(session_key)
        .fetch_all(&mut *conn)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| TurnObservation {
                id: r.0,
                session_key: r.1,
                actor_id: r.2,
                turn_idx: r.3,
                model: r.4,
                input_tokens: r.5,
                output_tokens: r.6,
                reasoning_tokens: r.7,
                cache_creation_tokens: r.8,
                cache_read_tokens: r.9,
                stop_reason: r.10,
                harness_reported_cost_usd: r.11,
                observed_at: r.12,
            })
            .collect())
    })
}

/// Per-session aggregate token totals across all turns. Cheap for the
/// dashboard summary card; the per-turn breakdown lives in
/// list_session_turns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTokenSummary {
    pub session_key: String,
    pub turn_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
    /// Sum of harness-reported per-turn costs. None when no turn
    /// surfaced one. Source of truth = harness; never recomputed.
    pub harness_reported_cost_usd: Option<f64>,
}

pub fn session_token_summary(
    conn: &mut SqliteConnection,
    session_key: &str,
) -> Result<Option<SessionTokenSummary>> {
    block_on(async {
        let row: Option<(i64, i64, i64, i64, i64, i64, Option<f64>)> = sqlx::query_as(
            "SELECT COUNT(*), \
                    COALESCE(SUM(input_tokens), 0), \
                    COALESCE(SUM(output_tokens), 0), \
                    COALESCE(SUM(reasoning_tokens), 0), \
                    COALESCE(SUM(cache_creation_tokens), 0), \
                    COALESCE(SUM(cache_read_tokens), 0), \
                    SUM(harness_reported_cost_usd) \
             FROM analytics_turns WHERE session_key = ?",
        )
        .bind(session_key)
        .fetch_optional(&mut *conn)
        .await?;
        Ok(row.and_then(|(turn_count, input, output, reasoning, cache_w, cache_r, cost)| {
            if turn_count == 0 {
                None
            } else {
                Some(SessionTokenSummary {
                    session_key: session_key.to_string(),
                    turn_count,
                    input_tokens: input,
                    output_tokens: output,
                    reasoning_tokens: reasoning,
                    cache_creation_tokens: cache_w,
                    cache_read_tokens: cache_r,
                    harness_reported_cost_usd: cost,
                })
            }
        }))
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
