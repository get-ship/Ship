//! ActorStore — scoped event handle for a single actor.
//!
//! Writes and queries delegate to the shared platform.db `events` table. The
//! `ActorStore` enforces namespace boundaries on writes and filters reads by
//! the owning actor's `actor_id`. No per-actor SQLite files are created —
//! the actor is a logical boundary over a shared log.
//!
//! The kernel creates `ActorStore` instances via `KernelRouter::spawn_actor`;
//! actors never construct their own.

use anyhow::{Result, anyhow};
use sqlx::QueryBuilder;
use std::path::{Path, PathBuf};

use crate::db::envelope::envelope_from_row;
use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::EventFilter;

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    causation_id, workspace_id, session_id, \
    actor_id, parent_actor_id, elevated, created_at, target_actor_id";

/// A scoped event handle bound to a single actor's SQLite DB.
///
/// Writes are restricted to `write_namespaces`. Typed-filter queries are
/// restricted to `read_namespaces`. Unfiltered queries always return only
/// this actor's own events.
#[derive(Debug)]
pub struct ActorStore {
    actor_id: String,
    db_path: PathBuf,
    write_namespaces: Vec<String>,
    read_namespaces: Vec<String>,
}

impl ActorStore {
    pub(crate) fn new(
        actor_id: impl Into<String>,
        db_path: PathBuf,
        write_namespaces: Vec<String>,
        read_namespaces: Vec<String>,
    ) -> Self {
        Self {
            actor_id: actor_id.into(),
            db_path,
            write_namespaces,
            read_namespaces,
        }
    }

    /// Open an ActorStore for the given actor, backed by platform.db.
    ///
    /// `base_dir` is the global Ship directory (e.g. `~/.ship`). Platform.db
    /// must have been initialised already (via `runtime::db::ensure_db_at`).
    pub fn open(
        actor_id: &str,
        base_dir: &Path,
        write_namespaces: Vec<String>,
        read_namespaces: Vec<String>,
    ) -> Result<Self> {
        let db_path = base_dir.join("platform.db");
        Ok(Self::new(actor_id, db_path, write_namespaces, read_namespaces))
    }

    pub fn actor_id(&self) -> &str {
        &self.actor_id
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Persist an event. Rejects `event_type` outside `write_namespaces`.
    ///
    /// The event is stamped with this actor's `actor_id` before writing —
    /// logical isolation requires every row in the shared events table know
    /// which actor wrote it. Any pre-existing `actor_id` on the envelope is
    /// overwritten so a store can't launder events for another actor.
    pub fn append(&self, event: &EventEnvelope) -> Result<()> {
        self.check_write_namespace(&event.event_type)?;
        let mut stamped = event.clone();
        stamped.actor_id = Some(self.actor_id.clone());
        raw_append(&self.db_path, &stamped)
    }

    /// Query events owned by this actor.
    ///
    /// All queries against platform.db are scoped to `actor_id = self.actor_id`
    /// — an ActorStore is a filter view over the shared log, not a private
    /// store. If `filter.event_type` is set it must be covered by
    /// `read_namespaces`.
    pub fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
        if let Some(ref et) = filter.event_type {
            self.check_read_namespace(et)?;
        }
        // Override any caller-supplied actor_id filter with our own — an
        // ActorStore is the actor's view, not a general query API.
        let mut scoped = filter.clone();
        scoped.actor_id = Some(self.actor_id.clone());
        query_db(&self.db_path, &scoped)
    }

    fn check_write_namespace(&self, event_type: &str) -> Result<()> {
        if self
            .write_namespaces
            .iter()
            .any(|ns| event_type.starts_with(ns.as_str()))
        {
            return Ok(());
        }
        Err(anyhow!(
            "write namespace violation: '{}' not permitted for actor '{}'",
            event_type,
            self.actor_id
        ))
    }

    fn check_read_namespace(&self, event_type: &str) -> Result<()> {
        if self
            .read_namespaces
            .iter()
            .any(|ns| event_type.starts_with(ns.as_str()))
        {
            return Ok(());
        }
        Err(anyhow!(
            "read namespace violation: '{}' not permitted for actor '{}'",
            event_type,
            self.actor_id
        ))
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Write an event to a DB path without namespace enforcement.
/// Used by `ActorStore::append` (after namespace check) and by
/// `KernelRouter::route` for kernel-namespace events. In the single-event-log
/// architecture the `db_path` is always platform.db; the parameter is kept
/// so tests can target a temp DB.
pub(crate) fn raw_append(db_path: &Path, event: &EventEnvelope) -> Result<()> {
    let mut conn = open_db_at(db_path)?;
    let created_at = event.created_at.to_rfc3339();
    block_on(async {
        // INSERT OR IGNORE makes this idempotent — multiple writers
        // can call append for the same event id without UNIQUE
        // constraint failures. Necessary because some emit paths go
        // through both `db::*_events::run_tx` (which already wrote
        // the row) AND `kernel.emit` (which calls this helper). The
        // first write wins; subsequent attempts are no-ops. Cheaper
        // than coordinating two parallel write paths.
        sqlx::query(
            "INSERT OR IGNORE INTO events \
             (id, event_type, entity_id, actor, payload_json, version, \
              causation_id, workspace_id, session_id, \
              actor_id, parent_actor_id, elevated, created_at, target_actor_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&event.event_type)
        .bind(&event.entity_id)
        .bind(&event.actor)
        .bind(&event.payload_json)
        .bind(event.version as i64)
        .bind(&event.causation_id)
        .bind(&event.workspace_id)
        .bind(&event.session_id)
        .bind(&event.actor_id)
        .bind(&event.parent_actor_id)
        .bind(event.elevated as i64)
        .bind(&created_at)
        .bind(&event.target_actor_id)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

fn query_db(db_path: &Path, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_db_at(db_path)?;
    let mut qb: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new(format!("SELECT {COLS} FROM events WHERE 1=1"));

    if let Some(ref v) = filter.entity_id {
        qb.push(" AND entity_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.event_type {
        qb.push(" AND event_type = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.workspace_id {
        qb.push(" AND workspace_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.session_id {
        qb.push(" AND session_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.actor_id {
        qb.push(" AND actor_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.parent_actor_id {
        qb.push(" AND parent_actor_id = ").push_bind(v.clone());
    }
    if filter.elevated_only {
        qb.push(" AND elevated = 1");
    }
    if let Some(ref since) = filter.since {
        qb.push(" AND created_at >= ").push_bind(since.to_rfc3339());
    }
    qb.push(" ORDER BY id ASC");
    if let Some(limit) = filter.limit {
        qb.push(" LIMIT ").push_bind(limit as i64);
    }

    let rows = block_on(async { qb.build().fetch_all(&mut conn).await })?;
    rows.iter().map(|r| envelope_from_row(r, true)).collect()
}

