//! Event-sourced `SpacesProjection` — folds the Runtime-scoped `space.*`
//! events into the `spaces` SQL table.
//!
//! The `spaces` table is the daemon's device-local record of opened spaces —
//! the LaunchServices analogy per the operator's 2026-07-09 ruling. Ship-space
//! (this crate) is the sole writer of `space.opened` and `space.seen` events;
//! this projection is the sole writer of read-side `spaces` rows.
//!
//! Identity is minted ONCE in [`crate::identity::ensure_identity_for_root`]
//! from a nanoid — never derived from the path. `space_id` is the envelope's
//! `entity_id`; `path` / `name` / `opened_at` ride the payload.

use anyhow::Result;
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::{SPACE_OPENED, SPACE_SEEN};
use ship_types::payload::{SpaceOpened, SpaceSeen};

/// Event-sourced materialiser for the `spaces` table.
pub struct SpacesProjection;

impl Projection for SpacesProjection {
    fn name(&self) -> &str {
        "spaces"
    }

    fn event_types(&self) -> &[&str] {
        &[SPACE_OPENED, SPACE_SEEN]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            SPACE_OPENED => apply_opened(event, conn),
            SPACE_SEEN => apply_seen(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        ship_substrate::db::heal::truncate_or_heal(conn, "spaces")
    }
}

fn apply_opened(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: SpaceOpened = serde_json::from_str(&event.payload_json)?;
    let space_id = event.entity_id.clone();
    block_on(async {
        sqlx::query(
            "INSERT INTO spaces \
             (space_id, path, name, opened_at, last_seen_at) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(space_id) DO UPDATE SET \
               path         = excluded.path, \
               name         = excluded.name, \
               opened_at    = excluded.opened_at, \
               last_seen_at = excluded.last_seen_at, \
               updated_at   = datetime('now')",
        )
        .bind(&space_id)
        .bind(&payload.path)
        .bind(&payload.name)
        .bind(&payload.opened_at)
        .bind(&payload.opened_at)
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn apply_seen(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let payload: SpaceSeen = serde_json::from_str(&event.payload_json)?;
    block_on(async {
        sqlx::query(
            "UPDATE spaces SET last_seen_at = ?, updated_at = datetime('now') WHERE space_id = ?",
        )
        .bind(&payload.seen_at)
        .bind(&event.entity_id)
        .execute(conn)
        .await
    })?;
    Ok(())
}
