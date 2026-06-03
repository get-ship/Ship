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

use crate::db::envelope::{EVENT_COLS as COLS, envelope_from_row};
use crate::db::{block_on, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::EventFilter;

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
    pub fn new(
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

    /// Open an ActorStore for the given actor, backed by the Runtime-scope
    /// `runtime.db` (actors are Runtime-owned).
    ///
    /// `base_dir` is the global Ship directory (e.g. `~/.ship`) — NOT a
    /// pre-resolved db path. The runtime.db schema is provisioned by the kernel
    /// sink on first persist; replay reads here resolve the same file.
    pub fn open(
        actor_id: &str,
        base_dir: &Path,
        write_namespaces: Vec<String>,
        read_namespaces: Vec<String>,
    ) -> Result<Self> {
        let db_path = crate::db::runtime_db_path(base_dir);
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
    ///
    /// This is the persist primitive (namespace check only); live emit sites
    /// must use [`ActorStore::append_enforced`] so scope is enforced too.
    pub fn append(&self, event: &EventEnvelope) -> Result<()> {
        self.check_write_namespace(&event.event_type)?;
        let mut stamped = event.clone();
        stamped.actor_id = Some(self.actor_id.clone());
        raw_append(&self.db_path, &stamped)
    }

    /// The enforced actor-write chokepoint.
    ///
    /// Off-kernel service handlers (e.g. the sync service) that hold an
    /// `ActorStore` write through here, not the raw `append`, so scope is
    /// enforced as well as the actor namespace. Runs the write-namespace check,
    /// then the SAME stamp-or-reject rule the emit/append chokepoints run —
    /// `ship_types::stamp_author` then `ship_types::enforce_scope` against the
    /// authoritative `binding` — BEFORE persisting. The actor's own `actor_id`
    /// is stamped (logical isolation) and the stamped envelope returned.
    pub fn append_enforced(
        &self,
        event: &EventEnvelope,
        binding: &ship_types::ScopeBinding,
    ) -> Result<EventEnvelope> {
        self.check_write_namespace(&event.event_type)?;
        let mut stamped = event.clone();
        stamped.actor_id = Some(self.actor_id.clone());
        ship_types::stamp_author(&mut stamped, binding);
        ship_types::enforce_scope(&mut stamped, binding)
            .map_err(|e| anyhow!("{e}"))?;
        raw_append(&self.db_path, &stamped)?;
        Ok(stamped)
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
    crate::db::envelope::insert_event_row(event, &mut conn)
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
    if let Some(ref v) = filter.scope {
        qb.push(" AND scope = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.scope_key {
        qb.push(" AND scope_key = ").push_bind(v.clone());
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

