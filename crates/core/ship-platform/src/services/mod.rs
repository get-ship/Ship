// L0 service infrastructure re-exported from ship-substrate.
pub use ship_substrate::services::{
    HealthStatus, ServiceHandle, ServiceHandler, spawn_service, run_service,
};
pub use ship_substrate::services::mesh::{self, AgentStatus, MeshEntry, SharedMeshRegistry};
pub use ship_substrate::services::dispatch_ports::{self};
pub use ship_substrate::services::human_gateway::{self};
pub use ship_substrate::services::process_tree_probe::{self};
pub use ship_substrate::services::server_session::{self};

// L1 service actors.
pub mod orchestrator;
pub mod process_registry;
pub mod provider_adapter;
pub mod providers;
pub mod watcher;

#[cfg(feature = "unstable")]
pub mod sync;

// L1 domain services (method-oriented; mr-02 surface). Each is a trait
// + Impl pair that file-backed domain operations (rules, permissions,
// mcp, skills, agents, ...) consume. FilesService is the foundational
// path-scoped JSONC read/write helper; the rest follow as their HTTP
// shims stay in shipd (apps/shipd/src/services/<domain>/mod.rs).
pub mod data;
pub mod files;
pub mod git;
pub mod jsonc;
pub mod mcp;
pub mod agents;
pub mod artifacts;
pub mod collections;
pub mod permissions;
pub mod pipelines;
pub mod skills;
pub mod path_tolerant;
pub mod rules;
pub mod sessions;

// Cross-component integration tests spanning substrate + platform service
// actors. They live here (rather than in a separate crate) so neither
// substrate nor platform needs a dev-dependency on the other.
#[cfg(test)]
mod cross_component_tests;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_drain;
#[cfg(test)]
mod tests_provider_adapter;
