// All service infrastructure and L1 service actors now live in ship-primitives.
// Re-export the complete surface so existing callers don't need path changes.
pub use ship_primitives::services::*;
pub use ship_primitives::services::{
    dispatch_ports, human_gateway, mesh, orchestrator, process_tree_probe, provider_adapter,
    providers, server_session,
};

#[cfg(feature = "unstable")]
pub use ship_primitives::services::sync;

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
