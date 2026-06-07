//! Shared row → EventEnvelope deserialization.
//!
//! `platform.db` and actor DBs share the same 18-column `events` schema.
//! The job projection uses a 13-column SELECT (omitting `target_actor_id` and
//! the three author/signature columns — it stops before column 14).
//! This module provides a single function for both cases.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::Row;

use ship_types::envelope::EventEnvelope;

use crate::db::block_on;

/// Canonical 18-column SELECT list for the `events` table.
/// Order MUST match [`envelope_from_row`]'s column indices. Columns 0–14 are
/// unchanged; 15/16/17 are the additive Phase-2 author/signature columns.
pub const EVENT_COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    causation_id, scope, scope_key, session_id, \
    actor_id, parent_actor_id, elevated, created_at, target_actor_id, \
    author_account_id, author_device_id, signature";

/// INSERT an 18-column event row using an existing connection. No transaction
/// management, no projection dispatch — that's the caller's job.
pub(crate) fn insert_event_row(
    event: &EventEnvelope,
    conn: &mut sqlx::SqliteConnection,
) -> Result<()> {
    let created_at = event.created_at.to_rfc3339();
    let scope = event.scope.as_str();
    block_on(async {
        sqlx::query(
            "INSERT INTO events \
             (id, event_type, entity_id, actor, payload_json, version, \
              causation_id, scope, scope_key, session_id, \
              actor_id, parent_actor_id, elevated, created_at, target_actor_id, \
              author_account_id, author_device_id, signature) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&event.event_type)
        .bind(&event.entity_id)
        .bind(&event.actor)
        .bind(&event.payload_json)
        .bind(event.version as i64)
        .bind(&event.causation_id)
        .bind(scope)
        .bind(&event.scope_key)
        .bind(&event.session_id)
        .bind(&event.actor_id)
        .bind(&event.parent_actor_id)
        .bind(event.elevated as i64)
        .bind(&created_at)
        .bind(&event.target_actor_id)
        .bind(&event.author_account_id)
        .bind(&event.author_device_id)
        .bind(&event.signature)
        .execute(conn)
        .await
    })?;
    Ok(())
}

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
    //          causation_id(6) scope(7) scope_key(8) session_id(9)
    //          actor_id(10) parent_actor_id(11) elevated(12) created_at(13)
    //          [target_actor_id(14) author_account_id(15) author_device_id(16)
    //           signature(17)]
    let created_at_str: String = row.get(13);
    let created_at = created_at_str
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
        })
        .map_err(|e| anyhow::anyhow!("invalid created_at '{}': {}", created_at_str, e))?;

    // The 13-col job-projection SELECT stops before column 14, so the author/
    // signature columns ride with `target_actor_id`: present in the full
    // 18-col SELECT, absent (→ None) in the narrow projection query.
    let (target_actor_id, author_account_id, author_device_id, signature) = if has_target_actor {
        (row.get(14), row.get(15), row.get(16), row.get(17))
    } else {
        (None, None, None, None)
    };

    // Fail-closed: a row whose stored scope is unrecognized (or NULL on a
    // legacy row the reconcile mapped) defaults to Runtime — the same posture
    // the registry uses for unclassified events.
    let scope_str: Option<String> = row.get(7);
    let scope = scope_str
        .as_deref()
        .and_then(ship_types::event_registry::EventScope::from_storage)
        .unwrap_or(ship_types::event_registry::EventScope::Runtime);

    Ok(EventEnvelope {
        id: row.get(0),
        event_type: row.get(1),
        entity_id: row.get(2),
        actor: row.get(3),
        payload_json: row.get(4),
        version: row.get::<i64, _>(5) as u32,
        causation_id: row.get(6),
        scope,
        scope_key: row.get(8),
        session_id: row.get(9),
        actor_id: row.get(10),
        parent_actor_id: row.get(11),
        elevated: row.get::<i64, _>(12) != 0,
        created_at,
        target_actor_id,
        author_account_id,
        author_device_id,
        signature,
    })
}
