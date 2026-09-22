//! Actor state projection — derived from actor.* events.
//!
//! Read API: `list_actors` and `get_actor` project the actors table for
//! the unified dashboard. Filterable by workspace, kind, status; ordered
//! by `last_event_at` desc so the most recently active actors surface
//! first.
//!
//! This projection maintains the `actors` table in the Runtime-scope `runtime.db`
//! as a read model. It handles the actor lifecycle events (created/woke/slept/
//! stopped/crashed) plus `process.spawned` for pid backfill; `actor.mailbox_full`
//! is not folded. On rebuild, the table is truncated and replayed from the event log.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::async_projection::AsyncProjection;
use super::registry::Projection;
use crate::events::EventEnvelope;
use crate::events::types::event_types;

mod handlers;
mod read;
mod row;

pub use read::{
    ActorEventRow, active_actor_ids, active_actor_ids_at, actor_status, find_live_window_for_space,
    get_actor, list_actor_events, list_actors, list_windows,
};
pub use row::{ActorRow, ListActorsFilter, pid_alive};

/// Projection that maintains the actors table from actor.* events.
#[derive(Default)]
pub struct ActorProjection;

impl ActorProjection {
    pub fn new() -> Self {
        Self
    }
}

const HANDLED: &[&str] = &[
    event_types::ACTOR_CREATED,
    event_types::ACTOR_WOKE,
    event_types::ACTOR_SLEPT,
    event_types::ACTOR_STOPPED,
    event_types::ACTOR_CRASHED,
    // Cross-domain: process.spawned carries the real pid the supervisor
    // captures AFTER the OS spawn returns. actor.created (emitted before
    // spawn from emit_actor_created_simple) doesn't have it. Apply the
    // pid from process.spawned to the matching actor row so live = pid_alive
    // works honestly instead of falling back to the activity_live timer.
    crate::events::process::event_types::PROCESS_SPAWNED,
];

impl Projection for ActorProjection {
    fn name(&self) -> &str {
        "actor_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        let entity = &event.entity_id;
        match event.event_type.as_str() {
            event_types::ACTOR_CREATED => {
                handlers::apply_created(entity, &event.payload_json, conn)
            }
            event_types::ACTOR_WOKE => handlers::apply_status(entity, "active", conn),
            event_types::ACTOR_SLEPT => handlers::apply_status(entity, "sleeping", conn),
            event_types::ACTOR_STOPPED => {
                handlers::apply_stopped(entity, &event.payload_json, conn)
            }
            event_types::ACTOR_CRASHED => {
                handlers::apply_crashed(entity, &event.payload_json, conn)
            }
            t if t == crate::events::process::event_types::PROCESS_SPAWNED => {
                handlers::apply_process_spawned(&event.payload_json, conn)
            }
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        crate::db::heal::truncate_or_heal(conn, "actors")
    }
}

impl AsyncProjection for ActorProjection {
    fn name(&self) -> &str {
        Projection::name(self)
    }
    fn event_types(&self) -> &[&str] {
        Projection::event_types(self)
    }
    fn apply(
        &self,
        event: &EventEnvelope,
        conn: &mut sqlx::SqliteConnection,
    ) -> anyhow::Result<()> {
        Projection::apply(self, event, conn)
    }
    fn truncate(&self, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::truncate(self, conn)
    }
}
