use std::path::Path;
use std::sync::Arc;

use super::data_store::DataStore;
use super::kernel_log::KernelLog;
use super::single_backend::SingleBackendDataStore;

/// Opaque per-space identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceId(pub String);

impl SpaceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SpaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Per-request handle bundling space identity, data access, and event log.
///
/// Constructed once per request (or once per mounted space) by shipd's
/// routing layer. Domain code receives `&SpaceContext` or `Arc<SpaceContext>`;
/// it never touches `DataService` or `KernelRouter` directly.
pub struct SpaceContext {
    pub space_id: SpaceId,
    pub data_store: Arc<dyn DataStore>,
    pub kernel: Arc<KernelLog>,
}

impl SpaceContext {
    pub fn new(
        space_id: SpaceId,
        data_store: Arc<dyn DataStore>,
        kernel: Arc<KernelLog>,
    ) -> Self {
        Self { space_id, data_store, kernel }
    }

    /// Mount a `SpaceContext` whose `DataStore` is backed by this space's OWN
    /// physical db file (`<ship_dir>/spaces/<space_id>/space.db`).
    ///
    /// This is the substrate keystone for physical per-space partition: each
    /// space's relational read-model lives in a SEPARATE SQLite file, not a
    /// shared `data.db` with a logical `space_id` column. The collection /
    /// artifact projections write their tables into that same per-scope db
    /// inside the kernel's append transaction (`collection.*` / `artifact.*`
    /// are Space-scoped), so a read through `data()` queries exactly the file
    /// the projection wrote (ADR D6/D7).
    ///
    /// The schema self-ensures on demand: `ensure_events_db_at_path` provisions
    /// the events log so a read that races ahead of the first per-scope write
    /// still finds a valid db, and `DataStore::ensure_table` self-creates the
    /// relational tables. No migrations.
    ///
    /// The kernel is the process-global router (one router per daemon, shared
    /// across all mounted spaces — only the DATA db is per-space); it is
    /// initialized lazily here rooted at `ship_dir` if startup has not already
    /// done so.
    pub fn for_space(ship_dir: &Path, space_id: SpaceId) -> anyhow::Result<Self> {
        let db_path = crate::db::space_db_path(ship_dir, space_id.as_str());
        // Self-ensure the per-scope events schema (mirrors `open_space_db`), so
        // the file + its events log exist before either the projection writes or
        // a read races ahead of the first write.
        crate::db::ensure_events_db_at_path(&db_path)?;
        let data_svc = crate::data::DataService::open_at(&db_path)?;
        let data_store: Arc<dyn DataStore> =
            Arc::new(SingleBackendDataStore::new(data_svc, space_id.as_str()));
        let router = crate::events::init_kernel_router(ship_dir.to_path_buf())?;
        let kernel = Arc::new(KernelLog::new(router));
        Ok(Self { space_id, data_store, kernel })
    }

    pub fn space_id(&self) -> &SpaceId {
        &self.space_id
    }

    pub fn data(&self) -> &dyn DataStore {
        &*self.data_store
    }

    pub fn kernel(&self) -> &KernelLog {
        &self.kernel
    }
}
