// L0 service infrastructure re-exported from ship-substrate.
// NOTE: callers must import from ship_substrate::services directly.
// These re-exports remain only for crate-internal use in install_defaults().

pub mod agent_sessions;
pub mod process_registry;
pub mod git;

/// Aggregated `ship.<domain>.<op>` → example-JSON pairs across every domain
/// migrated to the typed-op single source. Drives `cargo xtask gen-fixtures`,
/// which writes one committed JSON file per entry. Behind the `fixtures`
/// feature (the `example()` constructors compile only then or under test).
#[cfg(feature = "fixtures")]
pub fn data_op_examples() -> Vec<(&'static str, serde_json::Value)> {
    let mut out: Vec<(&'static str, serde_json::Value)> = Vec::new();
    out.extend(ship_agents::services::agents::fixtures::examples());
    out.extend(ship_agents::services::rules::fixtures::examples());
    out.extend(ship_agents::services::mcp::fixtures::examples());
    out.extend(ship_agents::services::permissions::fixtures::examples());
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
