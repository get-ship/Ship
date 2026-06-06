// L0 service infrastructure re-exported from ship-substrate.
pub use ship_substrate::services::{
    HealthStatus, ServiceHandle, ServiceHandler, spawn_service, run_service,
};
pub use ship_substrate::services::mesh::{self, AgentStatus, MeshEntry, SharedMeshRegistry};
pub use ship_substrate::services::dispatch_ports::{self};
pub use ship_substrate::services::human_gateway::{self};
pub use ship_substrate::services::process_tree_probe::{self};
pub use ship_substrate::services::server_session::{self};

// L1 service actors. The pipeline runtime (orchestrator + PipelinesService)
// carved out to the `ship-pipeline-runtime` crate (P3 carve); only the
// agents-actor / harnesses / registry live here.
pub mod agent_sessions;
pub mod process_registry;
// Harness adapter registry + concrete adapters carved into ship-harnesses (L1).
// Re-exported here so existing `crate::services::{harness_adapter,harnesses}::…`
// consumers in this crate and in shipd keep compiling without change.
pub use ship_harnesses::harness_adapter;
pub use ship_harnesses::harnesses;
// line_accumulator, files, jsonc, path_tolerant carved into ship-files (pre-C8d).
// Re-exported here so all existing `crate::services::{…}::*` consumers keep compiling.
pub use ship_files::line_accumulator;
// Cloud-sync service actor (uses `SyncClient` from `ship-cloud` via the
// `unstable` feature gate). The handler itself stays in this crate because
// it implements `ServiceHandler` from substrate; only the transport moves
// out.
#[cfg(feature = "unstable")]
pub mod sync;
// L1 domain services (method-oriented; mr-02 surface). Each is a trait
// + Impl pair that file-backed domain operations (rules, permissions,
// mcp, skills, agents, ...) consume. FilesService is the foundational
// path-scoped JSONC read/write helper; the rest follow as their HTTP
// shims stay in shipd (apps/shipd/src/services/<domain>/mod.rs).
pub mod compile;
pub use ship_files::files;
pub use ship_files::jsonc;
pub use ship_files::path_tolerant;
pub mod git;
pub mod mcp;
pub mod agents;
pub mod permissions;
// pipelines service carved out to ship-pipeline-runtime crate
pub mod skills;
pub mod rules;
pub mod sessions;

/// Aggregated `ship.<domain>.<op>` → example-JSON pairs across every domain
/// migrated to the typed-op single source. Drives `cargo xtask gen-fixtures`,
/// which writes one committed JSON file per entry. Behind the `fixtures`
/// feature (the `example()` constructors compile only then or under test).
#[cfg(any(test, feature = "fixtures"))]
pub fn data_op_examples() -> Vec<(&'static str, serde_json::Value)> {
    let mut out: Vec<(&'static str, serde_json::Value)> = Vec::new();
    out.extend(agents::fixtures::examples());
    out.extend(rules::fixtures::examples());
    out.extend(mcp::fixtures::examples());
    out.extend(permissions::fixtures::examples());
    // account.* — the AccountState enum lives in the `ship-account` crate; a
    // fresh daemon is logged out.
    out.push((
        "account.state",
        serde_json::to_value(ship_account::types::AccountState::LoggedOut)
            .expect("AccountState serializes"),
    ));
    out
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_drain;
#[cfg(test)]
mod tests_harness_adapter;
