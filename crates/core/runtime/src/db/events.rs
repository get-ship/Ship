//! Typed event queries against the `events` table.

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::db::envelope::envelope_from_row;
use crate::db::{block_on, db_path, ensure_db, open_db_at};
use ship_types::envelope::EventEnvelope;

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    causation_id, workspace_id, session_id, \
    actor_id, parent_actor_id, elevated, created_at, target_actor_id";

pub fn list_all_events() -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
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
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
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

pub fn list_recent_events(limit: usize) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events ORDER BY id DESC LIMIT ?"
        ))
        .bind(limit as i64)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut envs: Vec<EventEnvelope> = rows.iter().map(|r| envelope_from_row(r, true)).collect::<Result<_>>()?;
    envs.reverse();
    Ok(envs)
}

/// Query events for a specific entity, filtered by event_type.
pub fn query_events_by_entity_and_type(
    entity_id: &str,
    event_type: &str,
) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
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

/// Query the most recent N events for a specific entity.
pub fn query_recent_events_for_entity(
    entity_id: &str,
    limit: usize,
) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events WHERE entity_id = ? ORDER BY id DESC LIMIT ?"
        ))
        .bind(entity_id)
        .bind(limit as i64)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut envs: Vec<EventEnvelope> = rows.iter().map(|r| envelope_from_row(r, true)).collect::<Result<_>>()?;
    envs.reverse();
    Ok(envs)
}

/// Query events with ID greater than the given cursor.
///
/// Used by the sync client to find events that haven't been pushed yet.
/// If `cursor` is None, returns all matching events.
/// If `elevated_only` is true, only returns elevated events (platform-scope).
pub fn query_events_since(
    cursor: Option<&str>,
    elevated_only: bool,
) -> Result<Vec<EventEnvelope>> {
    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    let query = match (cursor, elevated_only) {
        (Some(c), true) => {
            block_on(async {
                sqlx::query(&format!(
                    "SELECT {COLS} FROM events WHERE id > ? AND elevated = 1 ORDER BY id ASC"
                ))
                .bind(c)
                .fetch_all(&mut conn)
                .await
            })?
        }
        (Some(c), false) => {
            block_on(async {
                sqlx::query(&format!(
                    "SELECT {COLS} FROM events WHERE id > ? ORDER BY id ASC"
                ))
                .bind(c)
                .fetch_all(&mut conn)
                .await
            })?
        }
        (None, true) => {
            block_on(async {
                sqlx::query(&format!(
                    "SELECT {COLS} FROM events WHERE elevated = 1 ORDER BY id ASC"
                ))
                .fetch_all(&mut conn)
                .await
            })?
        }
        (None, false) => {
            block_on(async {
                sqlx::query(&format!("SELECT {COLS} FROM events ORDER BY id ASC"))
                    .fetch_all(&mut conn)
                    .await
            })?
        }
    };
    query.iter().map(|r| envelope_from_row(r, true)).collect()
}
