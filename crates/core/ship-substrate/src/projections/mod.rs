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
// 2026-06-10). The COLLECTION domain owns its projections at L1; the composition
// root passes them via [`ProjectionFactory`].
//
// ProvisionedGrantsProjection moved to ship-auth::grants_projection (keystone-A
// Phase A bundle 2, 2026-06-10). The AUTH domain owns its projection at L1;
// the composition root passes it via [`ProjectionFactory`].
pub use process::{ProcessRecord, ProcessStatus, load_processes};
pub use registry::{EventBus, Projection};
pub use spawn::{ProjectionHandle, spawn_with_failure_counter};

use std::sync::Arc;

/// Factory that produces additional (L1+) projections to register on an event
/// bus alongside the L0 `ActorProjection` default.
///
/// Constructed by the composition root (shipd / platform startup / tests) and
/// passed into `SqliteEventStore` / `PartitionedSink` / `KernelRouter` /
/// rebuild. Each store open invokes the factory so every scope db gets its
/// own projection instances. There is no process-global registry.
pub type ProjectionFactory = Arc<dyn Fn() -> Vec<Box<dyn Projection>> + Send + Sync>;

/// Empty factory — bus carries only the L0 `ActorProjection`.
pub fn no_extra_projections() -> ProjectionFactory {
    Arc::new(Vec::new)
}
