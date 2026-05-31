//! `CollectionContext` — builds the `SpaceContext` for `CollectionWire` in shipd.
//!
//! Opens a DataService connection to the daemon's data.db and wraps it with
//! `SingleBackendDataStore`. The context is constructed once at startup and
//! shared per-request via `Arc` cloning.

use std::sync::Arc;

use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

use crate::ship_runtime::ApiState;

/// Build a shared `SpaceContext` for `CollectionWire`.
///
/// Opens a separate DataService connection to data.db (SQLite WAL supports
/// multiple concurrent readers). The `KernelLog` shares the daemon's kernel.
pub fn make_collection_space_context(state: &ApiState) -> Arc<SpaceContext> {
    let data_svc = ship_substrate::data::DataService::open(state.ship_dir())
        .expect("collection_context: failed to open data.db");
    let store = Arc::new(SingleBackendDataStore::new(data_svc, "global"));
    let kernel_log = Arc::new(KernelLog::new(state.kernel().clone()));
    Arc::new(SpaceContext::new(SpaceId::new("global"), store, kernel_log))
}
