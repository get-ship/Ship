//! ActorSnapshot — portable serialization of an actor's complete state.
//!
//! An actor's full state is its event store. This module provides a struct
//! that captures the SQLite DB bytes (post WAL-checkpoint) plus the config
//! needed to respawn the actor on any host.
//!
//! ## STALE SEMANTICS — read before using
//!
//! This module was designed when each actor had its own SQLite file. That
//! architecture is gone. All actor events now live in the shared `events`
//! table in `runtime.db` with an `actor_id` column for logical isolation
//! (see `persistence::actor_store`).
//!
//! Under shared-log architecture, `db_bytes` no longer has an obvious
//! meaning — there is no per-actor DB file to checkpoint. `event_stats`
//! correctly filters by `actor_id`, but the snapshot/restore round-trip is
//! not exercised in production and may not work as the docstrings below
//! describe.
//!
//! Before depending on `ActorSnapshot`:
//!
//!   1. Decide whether snapshots should now be event-slice exports
//!      (rows from `events WHERE actor_id = ?`) or be removed entirely.
//!   2. Update callers and this module consistently.
//!
//! Do not expand this module's surface in the meantime.

use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ship_types::ActorConfig;

/// A portable snapshot of an actor's event store and configuration.
///
/// Produced by `KernelRouter::snapshot` or `KernelRouter::suspend`. The
/// `db_bytes` field is the full SQLite DB file after a WAL checkpoint —
/// all actor state lives in this file. Pass to `KernelRouter::restore`
/// on any host to resume the actor with its full history intact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSnapshot {
    pub actor_id: String,
    pub namespace: String,
    pub config: ActorConfig,
    /// Raw SQLite DB bytes after WAL checkpoint.
    pub db_bytes: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub event_count: u64,
    pub last_event_id: Option<String>,
}

impl ActorSnapshot {
    /// Serialize to MessagePack bytes for transfer.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        rmp_serde::to_vec(self)
            .map_err(|e| anyhow::anyhow!("snapshot serialize failed: {e}"))
    }

    /// Deserialize from MessagePack bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        rmp_serde::from_slice(data)
            .map_err(|e| anyhow::anyhow!("snapshot deserialize failed: {e}"))
    }
}

/// Count events and return the last event ID for a specific actor.
///
/// Used by `KernelRouter::snapshot` to populate snapshot metadata. Filters
/// by `actor_id` so counts reflect this actor's log only — kernel-emitted
/// events ABOUT the actor (e.g. `kernel.actor.snapshot`) are not included.
pub(crate) fn event_stats(db_path: &Path, actor_id: &str) -> Result<(u64, Option<String>)> {
    use crate::db::{block_on, open_db_at};

    let mut conn = open_db_at(db_path)?;
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE actor_id = ?")
            .bind(actor_id)
            .fetch_one(&mut conn)
            .await
    })?;
    let last_id: Option<String> = block_on(async {
        sqlx::query_scalar(
            "SELECT id FROM events WHERE actor_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(actor_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok((count as u64, last_id))
}
