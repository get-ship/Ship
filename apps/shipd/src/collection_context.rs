//! `CollectionContext` — builds the `SpaceContext` for `CollectionWire` in shipd.
//!
//! Collections are event-sourced (ADR D6/D7): the `DataStore` binds to the
//! space's per-scope event db (`spaces/<id>/space.db`), the SAME file the
//! `CollectionProjection` materializes the `collection_<id>` tables into, so a
//! query reads exactly what the projection wrote. The context is constructed
//! once at startup and shared per-request via `Arc` cloning.

use std::sync::Arc;

use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

use crate::ship_runtime::ApiState;

/// Build a shared `SpaceContext` for `CollectionWire`.
///
/// Opens a DataService connection to the space's per-scope `space.db` (SQLite
/// WAL supports multiple concurrent readers). The `KernelLog` shares the
/// daemon's kernel.
pub fn make_collection_space_context(state: &ApiState) -> Arc<SpaceContext> {
    let space_db = ship_substrate::db::space_db_path(state.ship_dir(), "global");
    let data_svc = ship_substrate::data::DataService::open_at(&space_db)
        .expect("collection_context: failed to open space.db");
    let store = Arc::new(SingleBackendDataStore::new(data_svc, "global"));
    let kernel_log = Arc::new(KernelLog::new(state.kernel().clone()));
    Arc::new(SpaceContext::new(SpaceId::new("global"), store, kernel_log))
}
