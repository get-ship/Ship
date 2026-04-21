//! Actor cursors — track per-actor delivery position for replay.
//!
//! ## STALE SEMANTICS — read before using
//!
//! This module was built for the per-actor-DB architecture, where each actor
//! had its own `~/.ship/actors/{id}/events.db`. That's gone; all events live
//! in the shared `events` table in `platform.db`.
//!
//! The cursor concept still fits under shared-log (actor X wants to resume
//! from event N), but the table creation (`actor_cursors`) is not integrated
//! with any live delivery path today. The feature is `#[cfg(feature =
//! "unstable")]` and has no production callers.
//!
//! Before depending on it:
//!
//!   1. Decide whether replay-cursor tracking is needed (today actors catch
//!      up by replaying from `events` filtered by `actor_id` since a given
//!      id — no separate table required).
//!   2. Either integrate `actor_cursors` into the kernel's delivery path or
//!      remove this module.

use anyhow::Result;
use std::path::Path;

use crate::db::{block_on, open_db_at};

/// Tracks the last event delivered to an actor, keyed by label.
#[derive(Debug, Clone)]
pub struct ActorCursor {
    /// The actor's stable label.
    pub actor_label: String,
    /// The ID of the last event delivered, or None if first spawn.
    pub last_event_id: Option<String>,
}

/// Ensure the cursor table exists in platform.db.
pub(crate) fn init_cursor_table(db_path: &Path) -> Result<()> {
    let mut conn = open_db_at(db_path)?;
    block_on(async {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS actor_cursors (
                actor_label  TEXT PRIMARY KEY NOT NULL,
                last_event_id TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}
