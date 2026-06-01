//! `ArtifactContext` — builds the `SpaceContext` for `ArtifactsWire` in shipd.
//!
//! Opens a DataService connection to the daemon's data.db and wraps it with
//! `SingleBackendDataStore`. The global ship dir is passed through for blob
//! backend construction.

use std::sync::Arc;

use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

use crate::ship_runtime::ApiState;

/// Build a shared `SpaceContext` for `ArtifactsWire`.
pub fn make_artifact_space_context(state: &ApiState) -> Arc<SpaceContext> {
    let data_svc = ship_substrate::data::DataService::open(state.ship_dir())
        .expect("artifact_context: failed to open data.db");
    let store = Arc::new(SingleBackendDataStore::new(data_svc, "global"));
    let kernel_log = Arc::new(KernelLog::new(state.kernel().clone()));
    Arc::new(SpaceContext::new(SpaceId::new("global"), store, kernel_log))
}
