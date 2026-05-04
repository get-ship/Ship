//! Platform service actors — headless actors that provide infrastructure APIs.
//!
//! A service actor runs continuously, consuming events from its mailbox and
//! optionally waking on a timer. Spawned by the kernel at startup; communicates
//! with other actors via kernel-routed events.
//!
//! # Implementing a service
//!
//! ```rust,ignore
//! struct MySvc;
//! impl ServiceHandler for MySvc {
//!     fn name(&self) -> &str { "my-svc" }
//!     fn handle(&mut self, event: &EventEnvelope, _store: &ActorStore) -> Result<()> { Ok(()) }
//! }
//! ```
//!
//! Register with the kernel:
//!
//! ```rust,ignore
//! kernel.spawn_service("my-svc", config, Box::new(MySvc))?;
//! ```

// L0 service infrastructure re-exported from ship-substrate.
pub use ship_substrate::services::{
    HealthStatus, ServiceHandle, ServiceHandler, spawn_service, run_service,
};
pub use ship_substrate::services::mesh::{self, AgentStatus, MeshEntry, SharedMeshRegistry};
pub use ship_substrate::services::dispatch_ports::{self};
pub use ship_substrate::services::human_gateway::{self};
pub use ship_substrate::services::process_tree_probe::{self};
pub use ship_substrate::services::server_session::{self};

// L1 service actors (use L0 above + agent/collection primitives).
pub mod orchestrator;
pub mod provider_adapter;
pub mod providers;
#[cfg(feature = "unstable")]
pub mod sync;
#[cfg(test)]
#[path = "tests_provider_adapter.rs"]
mod tests_provider_adapter;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod cross_component_tests;
#[cfg(test)]
#[path = "tests_drain.rs"]
mod tests_drain;

