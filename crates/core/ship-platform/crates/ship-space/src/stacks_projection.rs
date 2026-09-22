//! Event-sourced `SpaceStacksProjection` — folds `stack.registered` into the
//! Runtime-scoped `space_stacks` table.
//!
//! Membership is REGISTRATION: the composed space is exactly the registered
//! set — the runtime never scans directories for members. `entity_id` is the
//! space's id; the payload carries the stack root path (a location on this
//! device, never an identity).

use anyhow::Result;
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::STACK_REGISTERED;
use ship_types::payload::StackRegistered;

/// Event-sourced materialiser for the `space_stacks` table.
pub struct SpaceStacksProjection;

impl Projection for SpaceStacksProjection {
    fn name(&self) -> &str {
        "space_stacks"
    }

    fn event_types(&self) -> &[&str] {
        &[STACK_REGISTERED]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        let payload: StackRegistered = serde_json::from_str(&event.payload_json)?;
        let space_id = event.entity_id.clone();
        block_on(async {
            sqlx::query(
                "INSERT INTO space_stacks (space_id, root_path, registered_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(space_id, root_path) DO UPDATE SET registered_at = ?3",
            )
            .bind(&space_id)
            .bind(&payload.root_path)
            .bind(&payload.registered_at)
            .execute(conn)
            .await
        })?;
        Ok(())
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        ship_substrate::db::heal::truncate_or_heal(conn, "space_stacks")
    }
}
