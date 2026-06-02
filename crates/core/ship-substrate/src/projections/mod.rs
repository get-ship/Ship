//! Projection system — derived state from the event log.
//!
//! A [`Projection`] declares which event types it handles and how to apply them
//! to a read model (SQL table). The [`EventBus`] dispatches events to registered
//! projections after every append.
//!
//! Projections are the ONLY mutable state in the system. They can be truncated
//! and rebuilt from the event log at any time via [`rebuild`].

pub mod actor;
pub mod artifact;
pub mod async_projection;
pub mod collection;
pub mod process;
pub mod registry;
mod session;
pub mod spawn;

#[cfg(test)]
mod tests_actor;
#[cfg(test)]
mod tests_daemon_events;
#[cfg(test)]
mod tests_session;

pub use actor::ActorProjection;
pub use artifact::ArtifactProjection;
pub use async_projection::{AsyncProjection, spawn_projection};
pub use collection::CollectionProjection;
pub use process::{ProcessRecord, ProcessStatus, load_processes};
pub use registry::{EventBus, Projection};
pub use session::SessionProjection;
pub use spawn::{ProjectionHandle, spawn_with_failure_counter};

use std::sync::OnceLock;

/// Factory that produces additional projections to register on the default
/// event bus. Set at application startup by higher-layer crates (typically
/// `ship_platform::install_default_projections`); read by
/// `SqliteEventStore::default_bus`.
type ExtraProjectionsFactory = Box<dyn Fn() -> Vec<Box<dyn Projection>> + Send + Sync>;
static EXTRA_PROJECTIONS: OnceLock<ExtraProjectionsFactory> = OnceLock::new();

/// Install a factory for additional projections beyond the substrate L0
/// defaults. Idempotent — second and later calls are silently ignored.
/// Call this once at app startup before any `SqliteEventStore` is constructed.
pub fn install_extra_projections_factory(factory: ExtraProjectionsFactory) {
    let _ = EXTRA_PROJECTIONS.set(factory);
}

pub fn extra_projections() -> Vec<Box<dyn Projection>> {
    EXTRA_PROJECTIONS.get().map(|f| f()).unwrap_or_default()
}
