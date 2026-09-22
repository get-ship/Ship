//! Typed event queries against the `events` table.

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::db::envelope::{EVENT_COLS as COLS, envelope_from_row};
use crate::db::{block_on, open_db_at, open_runtime_db, runtime_db_path};
use ship_types::envelope::EventEnvelope;

/// Open the daemon-wide Runtime event log (`runtime.db`). These typed queries
/// scan the Runtime partition — the per-machine event feed (pipeline/session/
/// actor/process/hook are all Runtime-owned). Space/Account partitions are
/// surfaced through their own scope-bounded readers.
fn open_runtime_events() -> Result<sqlx::SqliteConnection> {
    open_runtime_db(&crate::project::get_global_dir()?)
}

pub fn list_all_events() -> Result<Vec<EventEnvelope>> {
    let mut conn = open_runtime_events()?;
    let rows = block_on(async {
        sqlx::query(&format!("SELECT {COLS} FROM events ORDER BY id ASC"))
            .fetch_all(&mut conn)
            .await
    })?;
    rows.iter().map(|r| envelope_from_row(r, true)).collect()
}

pub fn list_events_since_time(
    since: &DateTime<Utc>,
    limit: Option<usize>,
) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_runtime_events()?;
    let since_str = since.to_rfc3339();
    let rows = match limit {
        Some(n) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM events WHERE created_at >= ? ORDER BY id ASC LIMIT ?"
            ))
            .bind(&since_str)
            .bind(n as i64)
            .fetch_all(&mut conn)
            .await
        })?,
        None => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM events WHERE created_at >= ? ORDER BY id ASC"
            ))
            .bind(&since_str)
            .fetch_all(&mut conn)
            .await
        })?,
    };
    rows.iter().map(|r| envelope_from_row(r, true)).collect()
}

/// Like [`list_recent_events`] but against an explicit `ship_dir`. View
/// handlers prefer this so test harnesses stay isolated from the global
/// ship-home.
///
/// Reads the Runtime partition (`runtime.db`) — the daemon-wide event feed
/// (pipeline/session/actor/process/executor/hook are all Runtime-owned). The
/// per-space Space partitions are surfaced through the space→runtime
/// projection layer (FUTURE), not this daemon-wide reader.
pub fn list_recent_events_at(
    ship_dir: &std::path::Path,
    limit: usize,
) -> Result<Vec<EventEnvelope>> {
    let path = crate::db::runtime_db_path(ship_dir);
    crate::db::ensure_events_db_at_path(&path)?;
    let mut conn = open_db_at(&path)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events ORDER BY id DESC LIMIT ?"
        ))
        .bind(limit as i64)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut envs: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, true))
        .collect::<Result<_>>()?;
    envs.reverse();
    Ok(envs)
}

pub fn list_recent_events(limit: usize) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_runtime_events()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events ORDER BY id DESC LIMIT ?"
        ))
        .bind(limit as i64)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut envs: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, true))
        .collect::<Result<_>>()?;
    envs.reverse();
    Ok(envs)
}

/// Query events for a specific entity, filtered by event_type.
pub fn query_events_by_entity_and_type(
    entity_id: &str,
    event_type: &str,
) -> Result<Vec<EventEnvelope>> {
    query_events_by_entity_and_type_at(
        &runtime_db_path(&crate::project::get_global_dir()?),
        entity_id,
        event_type,
    )
}

/// Same as `query_events_by_entity_and_type` but against an explicit scope db
/// path (callers pass the resolved per-scope db, e.g. `runtime_db_path(...)`).
/// Handlers with access to `state.ship_dir()` should use this so test harnesses
/// don't leak state across isolated tempdirs.
pub fn query_events_by_entity_and_type_at(
    db_path: &std::path::Path,
    entity_id: &str,
    event_type: &str,
) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_db_at(db_path)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events WHERE entity_id = ? AND event_type = ? ORDER BY id ASC"
        ))
        .bind(entity_id)
        .bind(event_type)
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(|r| envelope_from_row(r, true)).collect()
}

/// Query the most recent N events for a specific entity against an explicit
/// scope db path (callers pass the resolved per-scope db, e.g.
/// `runtime_db_path(...)`).
pub fn query_recent_events_for_entity_at(
    db_path: &std::path::Path,
    entity_id: &str,
    limit: usize,
) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_db_at(db_path)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events WHERE entity_id = ? ORDER BY id DESC LIMIT ?"
        ))
        .bind(entity_id)
        .bind(limit as i64)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut envs: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, true))
        .collect::<Result<_>>()?;
    envs.reverse();
    Ok(envs)
}

/// Query events with ID greater than the given cursor.
///
/// Used by the sync client to find events that haven't been pushed yet.
/// If `cursor` is None, returns all matching events.
/// If `elevated_only` is true, only returns elevated events (platform-scope).
pub fn query_events_since(cursor: Option<&str>, elevated_only: bool) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_runtime_events()?;
    let query = match (cursor, elevated_only) {
        (Some(c), true) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM events WHERE id > ? AND elevated = 1 ORDER BY id ASC"
            ))
            .bind(c)
            .fetch_all(&mut conn)
            .await
        })?,
        (Some(c), false) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM events WHERE id > ? ORDER BY id ASC"
            ))
            .bind(c)
            .fetch_all(&mut conn)
            .await
        })?,
        (None, true) => block_on(async {
            sqlx::query(&format!(
                "SELECT {COLS} FROM events WHERE elevated = 1 ORDER BY id ASC"
            ))
            .fetch_all(&mut conn)
            .await
        })?,
        (None, false) => block_on(async {
            sqlx::query(&format!("SELECT {COLS} FROM events ORDER BY id ASC"))
                .fetch_all(&mut conn)
                .await
        })?,
    };
    query.iter().map(|r| envelope_from_row(r, true)).collect()
}
