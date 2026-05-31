use std::sync::Arc;

use super::data_store::DataStore;
use super::kernel_log::KernelLog;

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
