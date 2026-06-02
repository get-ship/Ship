//! Typed analytics service — the L1 surface callers use instead of
//! opening the DB directly.
//!
//! Each function opens its own connection to the Runtime-scope `runtime.db`
//! (the analytics rollups fold `hook.*` / `session.*`, both Runtime-owned) and
//! closes it on return. Callers (`analytics_rpc.rs`) never touch
//! `ship_substrate::db` themselves.

use anyhow::Result;

use ship_substrate::db::open_runtime_db;
use ship_substrate::project::get_global_dir;

use crate::queries::{self, SessionSummary, SessionTokenSummary, ToolUsage, TurnObservation};

/// Open the per-machine `runtime.db` that backs the analytics rollup tables.
fn open_analytics_db() -> Result<sqlx::SqliteConnection> {
    open_runtime_db(&get_global_dir()?)
}

/// List most-recent sessions, newest first. `limit` is capped at 500.
pub fn list_recent_sessions(limit: i64) -> Result<Vec<SessionSummary>> {
    let mut conn = open_analytics_db()?;
    queries::list_recent_sessions(&mut conn, limit)
}

/// Fetch one session by its primary key. `None` if absent.
pub fn get_session(session_key: &str) -> Result<Option<SessionSummary>> {
    let mut conn = open_analytics_db()?;
    queries::get_session(&mut conn, session_key)
}

/// Per-tool aggregate in the given time window. Both bounds are optional
/// RFC 3339 timestamps; `None` = open-ended.
pub fn tool_usage(since: Option<&str>, until: Option<&str>) -> Result<Vec<ToolUsage>> {
    let mut conn = open_analytics_db()?;
    queries::tool_usage(&mut conn, since, until)
}

/// Per-turn token observations for one session, oldest first.
pub fn list_session_turns(session_key: &str) -> Result<Vec<TurnObservation>> {
    let mut conn = open_analytics_db()?;
    queries::list_session_turns(&mut conn, session_key)
}

/// Aggregate token totals across all turns for one session.
pub fn session_token_summary(session_key: &str) -> Result<Option<SessionTokenSummary>> {
    let mut conn = open_analytics_db()?;
    queries::session_token_summary(&mut conn, session_key)
}
