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
pub mod files;
pub mod git;
pub mod jsonc;
pub mod mcp;
pub mod agents;
pub mod permissions;
pub mod pipelines;
pub mod skills;
pub mod path_tolerant;
pub mod rules;
#[cfg(test)]
#[path = "files_tests.rs"]
mod files_tests;
