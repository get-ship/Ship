//! Projection system — derived state from the event log.
//!
//! A [`Projection`] declares which event types it handles and how to apply them
//! to a read model (SQL table). The [`EventBus`] dispatches events to registered
//! projections after every append.
//!
//! Projections are the only DERIVED state in the event system: they can be
//! truncated and rebuilt from the event log via [`EventBus::rebuild`] /
//! `db::rebuild`. (Non-derived mutable stores — daemon_settings, kv.db,
//! DataStore-port tables, the blob plane — are not event-sourced and are not
//! rebuildable from the log.)

pub mod actor;
pub mod async_projection;
pub mod process;
pub mod registry;
pub mod spawn;

#[cfg(test)]
mod tests_actor;
#[cfg(test)]
mod tests_daemon_events;

pub use actor::ActorProjection;
pub use async_projection::{AsyncProjection, spawn_projection};
// CollectionProjection moved to ship-collection::item_projection (keystone A,
// 2026-06-10). The COLLECTION domain owns its projections at L1; it is wired
// into the bus via ship-platform's install_extra_projections_factory call.
//
// ProvisionedGrantsProjection moved to ship-auth::grants_projection (keystone-A
// Phase A bundle 2, 2026-06-10). The AUTH domain owns its projection at L1;
// it is wired via ship-platform's install_extra_projections_factory call.
pub use process::{ProcessRecord, ProcessStatus, load_processes};
pub use registry::{EventBus, Projection};
pub use spawn::{ProjectionHandle, spawn_with_failure_counter};

use std::sync::Mutex;

/// Factory that produces additional projections to register on the default
/// event bus. Registered at application startup by each higher-layer domain
/// (platform, pipeline-runtime, …); read by `SqliteEventStore::default_bus`.
type ExtraProjectionsFactory = Box<dyn Fn() -> Vec<Box<dyn Projection>> + Send + Sync>;
static EXTRA_PROJECTIONS: Mutex<Vec<ExtraProjectionsFactory>> = Mutex::new(Vec::new());

/// Register a factory for additional projections beyond the substrate L0
/// defaults. Each domain contributes its own projections through this single
/// API; the substrate composes every registered factory at bus-construction
/// time. Call at app startup before any `SqliteEventStore` is constructed.
pub fn install_extra_projections_factory(factory: ExtraProjectionsFactory) {
    EXTRA_PROJECTIONS
        .lock()
        .expect("extra-projections registry poisoned")
        .push(factory);
}

pub fn extra_projections() -> Vec<Box<dyn Projection>> {
    EXTRA_PROJECTIONS
        .lock()
        .expect("extra-projections registry poisoned")
        .iter()
        .flat_map(|f| f())
        .collect()
}
