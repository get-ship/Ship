//! `PartitionedSink` — scope-routed `EventPersist`.
//!
//! A single `EventPersist` that fans every persisted event out to a per-scope
//! SQLite db, keyed by the event's `(scope, scope_key)`:
//!
//! - `EventScope::Space`   → `spaces/<scope_key>/space.db`
//! - `EventScope::Account` → `account.db`
//! - `EventScope::Runtime` → `runtime.db`
//!
//! Each backing db is a lazily-opened, cached [`SqliteEventStore`]. The cache
//! key is the resolved db identity (`"space:<id>"`, `"account"`, `"runtime"`)
//! so the same store is reused across persists. Each store also dispatches the
//! full projection bus into its OWN db on persist, so a scope db carries both
//! its `events` log and the projection tables those events feed.
//!
//! This IS the kernel's sink: `KernelRouter::new` constructs a `PartitionedSink`
//! and hands it to `ship_kernel::KernelRouter`, so every emitted event lands in
//! its scope's db. There is no `platform.db` on the emit path. The shutdown WAL
//! flush iterates [`PartitionedSink::opened_db_paths`] to checkpoint exactly the
//! scope dbs it has mounted.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use ship_kernel::EventPersist;
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;
use ship_types::Stamped;

use super::event_store::SqliteEventStore;
use crate::db;
use crate::projections::{ProjectionFactory, no_extra_projections};

/// Routes events to a per-scope SQLite db under a ship-home root.
pub struct PartitionedSink {
    /// Ship-home root that scope db paths resolve under.
    root: PathBuf,
    /// Lazily-opened, cached per-scope stores keyed by db identity.
    stores: Mutex<HashMap<String, Arc<SqliteEventStore>>>,
    /// Extra (L1+) projections registered on every lazily-opened store.
    extra: ProjectionFactory,
}

impl PartitionedSink {
    /// Construct a sink rooted at the given ship-home directory with only the
    /// L0 `ActorProjection` on each store bus.
    pub fn new(root: PathBuf) -> Self {
        Self::with_extra_projections(root, no_extra_projections())
    }

    /// Construct a sink that registers `extra` projections on every store it
    /// opens. Composition roots pass the platform projection factory here.
    pub fn with_extra_projections(root: PathBuf, extra: ProjectionFactory) -> Self {
        Self {
            root,
            stores: Mutex::new(HashMap::new()),
            extra,
        }
    }

    /// Resolve `(cache_key, db_path)` for an event's scope. Errs when a Space
    /// event lacks the `scope_key` that Phase 3's `enforce_scope` is
    /// responsible for stamping, OR when `scope_key` names a `space_id` with
    /// no row in the `spaces` Runtime-scoped registry (I-2: `spaces/<key>/`
    /// on disk must correspond 1:1 with the registry). The registry is
    /// guaranteed populated before any real space_id reaches here: `space.
    /// opened` is itself Runtime-scoped and Durable, so its registration
    /// commits (event row + `SpacesProjection` fold, same SQLite
    /// transaction) before `ensure_identity_for_root` ever returns a
    /// `space_id` to a caller — nothing can hand this fn a genuine space_id
    /// that isn't already registered. Fail loud rather than mint a
    /// `spaces/<any-string>/space.db` for an unregistered / fabricated key.
    fn route(&self, event: &EventEnvelope) -> Result<(String, PathBuf)> {
        match event.scope {
            EventScope::Space => {
                let key = event.scope_key.as_deref().ok_or_else(|| {
                    anyhow!(
                        "PartitionedSink: Space event {} has no scope_key; refusing to route",
                        event.id
                    )
                })?;
                match db::spaces::get_at(&self.root, key) {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        return Err(anyhow!(
                            "PartitionedSink: Space event {} has scope_key `{key}` with no \
                             registry row in `spaces`; refusing to mint an unregistered \
                             spaces/{key}/space.db",
                            event.id
                        ));
                    }
                    Err(e) => {
                        return Err(anyhow!(
                            "PartitionedSink: failed to read `spaces` registry for scope_key \
                             `{key}`: {e}"
                        ));
                    }
                }
                Ok((format!("space:{key}"), db::space_db_path(&self.root, key)))
            }
            EventScope::Account => Ok(("account".to_string(), db::account_db_path(&self.root))),
            EventScope::Runtime => Ok(("runtime".to_string(), db::runtime_db_path(&self.root))),
        }
    }

    /// Snapshot of the db paths the sink has opened so far — one per scope it
    /// has actually routed an event to (`runtime.db`, `account.db`, each
    /// touched `spaces/<id>/space.db`). The shutdown WAL flush iterates this so
    /// it checkpoints exactly the mounted scope dbs rather than a single dead
    /// `platform.db`.
    pub fn opened_db_paths(&self) -> Vec<PathBuf> {
        self.stores
            .lock()
            .map(|stores| stores.values().map(|s| s.db_path().to_path_buf()).collect())
            .unwrap_or_default()
    }

    /// Return the cached store for `key`, lazily opening + caching it at
    /// `db_path` on a miss.
    fn store_for(&self, key: String, db_path: PathBuf) -> Result<Arc<SqliteEventStore>> {
        let mut stores = self
            .stores
            .lock()
            .map_err(|_| anyhow!("PartitionedSink: store cache mutex poisoned"))?;
        if let Some(store) = stores.get(&key) {
            return Ok(store.clone());
        }
        let store = Arc::new(SqliteEventStore::with_projection_factory(
            &db_path,
            &self.extra,
        )?);
        stores.insert(key, store.clone());
        Ok(store)
    }
}

impl EventPersist for PartitionedSink {
    fn persist(&self, event: &Stamped) -> anyhow::Result<()> {
        let (key, db_path) = self.route(event.as_envelope())?;
        let store = self.store_for(key, db_path)?;
        store.persist(event)
    }
}
