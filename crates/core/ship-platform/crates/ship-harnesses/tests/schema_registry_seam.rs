//! The agent schema's `harness_settings` key set must match the harness
//! REGISTRY (via `capabilities()`) — the single source of truth. Lives with
//! the registry: the platform schema conforms TO the code world's registry,
//! and core carries no ship-harnesses dependency.

use std::collections::BTreeSet;

use serde_json::Value;
use ship_harnesses::agent_driver::capabilities;

/// Load the canonical agent schema from the repo root (five levels up from
/// this crate's manifest at `crates/core/ship-platform/crates/ship-harnesses`).
fn load_schema() -> Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../../schemas/agent.schema.json");
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read agent schema at {path}: {e}"));
    serde_json::from_str(&raw).expect("agent schema is valid JSON")
}

#[test]
fn harness_settings_seam_matches_registry() {
    let schema = load_schema();
    let harness_node = schema
        .get("properties")
        .and_then(|p| p.get("harness_settings"))
        .expect("schema has harness_settings property");

    // Authoritative harness id set — derived from the Rust REGISTRY via
    // `capabilities()` (single source of truth), NOT hardcoded. Future
    // pi-class drift between the schema and the registry fails here.
    let registry_ids: BTreeSet<String> =
        capabilities().into_iter().map(|(id, _)| id.to_string()).collect();
    let schema_harness_keys: BTreeSet<String> = harness_node
        .get("properties")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();

    assert_eq!(
        schema_harness_keys, registry_ids,
        "harness_settings key drift vs REGISTRY: schema={schema_harness_keys:?} registry={registry_ids:?}"
    );
    // `additionalProperties: false` keeps the set closed — pin it so an
    // open seam can't silently re-admit a dropped harness.
    assert_eq!(
        harness_node.get("additionalProperties"),
        Some(&Value::Bool(false)),
        "harness_settings must stay a closed set"
    );
}
