//! Shared row → EventEnvelope deserialization.
//!
//! Platform.db and actor DBs share the same 14-column `events` schema.
//! The job projection uses a 13-column SELECT (omitting `target_actor_id`).
//! This module provides a single function for both cases.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;

use ship_types::envelope::EventEnvelope;

/// Deserialize a SQLite row into an `EventEnvelope`.
///
/// `has_target_actor` controls whether column 13 (`target_actor_id`) is present.
/// - `true` → 14-column SELECT (platform.db, actor DBs)
/// - `false` → 13-column SELECT (job projection query)
pub fn envelope_from_row(
    row: &sqlx::sqlite::SqliteRow,
    has_target_actor: bool,
) -> Result<EventEnvelope> {
    // Columns: id(0) event_type(1) entity_id(2) actor(3) payload_json(4) version(5)
    //          causation_id(6) workspace_id(7) session_id(8)
    //          actor_id(9) parent_actor_id(10) elevated(11) created_at(12)
    //          [target_actor_id(13)]
    let created_at_str: String = row.get(12);
    let created_at = created_at_str
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
        })
        .map_err(|e| anyhow::anyhow!("invalid created_at '{}': {}", created_at_str, e))?;

    let target_actor_id = if has_target_actor {
        row.get(13)
    } else {
        None
    };

    Ok(EventEnvelope {
        id: row.get(0),
        event_type: row.get(1),
        entity_id: row.get(2),
        actor: row.get(3),
        payload_json: row.get(4),
        version: row.get::<i64, _>(5) as u32,
        causation_id: row.get(6),
        workspace_id: row.get(7),
        session_id: row.get(8),
        actor_id: row.get(9),
        parent_actor_id: row.get(10),
        elevated: row.get::<i64, _>(11) != 0,
        created_at,
        target_actor_id,
    })
}
