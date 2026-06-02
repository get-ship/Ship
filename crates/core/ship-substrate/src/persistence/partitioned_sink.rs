//! `PartitionedSink` — scope-routed `EventSink` (Phase 4a).
//!
//! A single `EventSink` that fans every persisted event out to a per-scope
//! SQLite db, keyed by the event's `(scope, scope_key)`:
//!
//! - `EventScope::Space`   → `spaces/<scope_key>/space.db`
//! - `EventScope::Account` → `account.db`
//! - `EventScope::Runtime` → `runtime.db`
//!
//! Each backing db is a lazily-opened, cached [`SqliteEventStore`]. The cache
//! key is the resolved db identity (`"space:<id>"`, `"account"`, `"runtime"`)
//! so the same store is reused across persists.
//!
//! SCOPE BOUNDARY (Phase 4): this is a complete, isolated component. It is NOT
//! yet wired into `KernelRouter` (whose sink stays `platform.db`-backed) and it
//! touches no projection read path — that cutover is Phase 4d.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow};
use ship_kernel::EventSink;
use ship_types::envelope::EventEnvelope;
use ship_types::event_registry::EventScope;

use super::event_store::SqliteEventStore;
use crate::db;

/// Routes events to a per-scope SQLite db under a ship-home root.
pub struct PartitionedSink {
    /// Ship-home root that scope db paths resolve under.
    root: PathBuf,
    /// Lazily-opened, cached per-scope stores keyed by db identity.
    stores: Mutex<HashMap<String, Arc<SqliteEventStore>>>,
}

impl PartitionedSink {
    /// Construct a sink rooted at the given ship-home directory.
    pub fn new(root: PathBuf) -> Self {
        Self { root, stores: Mutex::new(HashMap::new()) }
    }

    /// Resolve `(cache_key, db_path)` for an event's scope. Errs only when a
    /// Space event lacks the `scope_key` that Phase 3's `enforce_scope` is
    /// responsible for stamping — fail loud rather than write to a wrong db.
    fn route(&self, event: &EventEnvelope) -> Result<(String, PathBuf)> {
        match event.scope {
            EventScope::Space => {
                let key = event.scope_key.as_deref().ok_or_else(|| {
                    anyhow!(
                        "PartitionedSink: Space event {} has no scope_key; refusing to route",
                        event.id
                    )
                })?;
                Ok((format!("space:{key}"), db::space_db_path(&self.root, key)))
            }
            EventScope::Account => {
                Ok(("account".to_string(), db::account_db_path(&self.root)))
            }
            EventScope::Runtime => {
                Ok(("runtime".to_string(), db::runtime_db_path(&self.root)))
            }
        }
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
        let store = Arc::new(SqliteEventStore::new_at_db_path(&db_path)?);
        stores.insert(key, store.clone());
        Ok(store)
    }
}

impl EventSink for PartitionedSink {
    fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()> {
        let (key, db_path) = self.route(event)?;
        let store = self.store_for(key, db_path)?;
        store.persist(event)
    }
}
