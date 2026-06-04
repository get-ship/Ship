//! Conformance gate: `schemas/agent.schema.json` ⟷ `compiler::AgentProfile`.
//!
//! Premise (agents-arc E2): agent config is file-truth; the SCHEMA is the
//! authority for shape; the Rust parse-target must conform to it. This test
//! pins "schema and struct must agree" as a permanent CI invariant — any
//! drift fails a key-set assertion with the symmetric difference printed.
//!
//! Layering: the schema⟷struct key parity could be compiler-local, but the
//! harness-enum and isolation-enum cross-checks need `harness_adapter`'s
//! REGISTRY (via `capabilities()`) and `ship_isolation`, which the compiler
//! crate (leaf / pure-transform) cannot import. So the full gate lives here
//! in `ship-platform`, which sees `compiler::AgentProfile`, the harness
//! REGISTRY, and `ship_isolation`.
//!
//! Two seams are deliberate and NOT diffed structurally:
//!   - `harness_settings` — opaque external-`$ref` pass-through. We assert its
//!     property key-set equals the authoritative harness id set pulled from the
//!     Rust REGISTRY (not hardcoded), nothing about inner harness shapes.
//!   - `isolation` — schema-only by design (resolved at platform runtime; it
//!     lives as `Option<Value>` on `config::AgentProfile`, deliberately absent
//!     from `compiler::AgentProfile`). We assert its tier/strategy enums equal
//!     the `ship_isolation` enum variant ids.

use std::collections::BTreeSet;

use compiler::types::agent_profile::{
    AgentProfile, McpRefs, PluginRefs, ProfileApps, ProfileHooks, ProfileMeta,
    ProfilePermissions, ProfileRules, SkillRefs,
};
use serde_json::Value;
use ship_isolation::{IsolationStrategy, IsolationTier};
use ship_platform::services::harness_adapter::capabilities;

/// Load + parse the canonical agent schema from the repo root. The test
/// crate's `CARGO_MANIFEST_DIR` is `crates/core/ship-platform`; the schema
/// is three levels up at `schemas/agent.schema.json`.
fn load_schema() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../schemas/agent.schema.json"
    );
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read agent schema at {path}: {e}"));
    serde_json::from_str(&raw).expect("agent schema is valid JSON")
}

/// The `properties` keys of a JSON-Schema object node.
fn property_keys(node: &Value) -> BTreeSet<String> {
    node.get("properties")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

/// The serde key set of a serialized struct value.
fn serialized_keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("struct serializes to a JSON object")
        .keys()
        .cloned()
        .collect()
}

/// Render the symmetric difference between two key sets for a failure message,
/// labelling which side owns each orphan key.
fn diff_report(struct_keys: &BTreeSet<String>, schema_keys: &BTreeSet<String>) -> String {
    let only_struct: Vec<_> = struct_keys.difference(schema_keys).cloned().collect();
    let only_schema: Vec<_> = schema_keys.difference(struct_keys).cloned().collect();
    format!(
        "struct-only (missing from schema): {only_struct:?}; \
         schema-only (missing from struct): {only_schema:?}"
    )
}

/// A maximally-populated profile: every optional field set, every collection
/// non-empty, so the serialized key set is the struct's full surface (no key
/// elided by `skip_serializing_if`). `ProfileMeta.icon` is the only
/// `skip_serializing_if = Option::is_none` field — it MUST be `Some` here.
fn full_profile() -> AgentProfile {
    AgentProfile {
        agent: ProfileMeta {
            id: "conformance".into(),
            name: "Conformance".into(),
            version: Some("1.0.0".into()),
            description: Some("Pins schema⟷struct parity.".into()),
            role: Some("worker".into()),
            tags: vec!["review".into()],
            tools_preset: Some("worker".into()),
            active_tools: vec!["Read".into()],
            icon: Some("test-tube".into()),
        },
        skills: SkillRefs { refs: vec!["tdd".into()] },
        mcp: McpRefs { servers: vec!["github".into()] },
        plugins: PluginRefs {
            install: vec!["superpowers@claude-plugins-official".into()],
            scope: Some("project".into()),
        },
        permissions: ProfilePermissions {
            preset: Some("ship-standard".into()),
            tools_allow: vec!["Read".into()],
            tools_ask: vec!["Edit".into()],
            tools_deny: vec!["WebFetch".into()],
            default_mode: Some("acceptEdits".into()),
        },
        rules: ProfileRules { inline: Some("Be precise.".into()) },
        hooks: ProfileHooks {
            stop: Some("ship session end".into()),
            subagent_stop: Some("ship subagent end".into()),
        },
        apps: ProfileApps { studio: true },
        harness_settings: [(
            "claude".to_string(),
            serde_json::json!({ "model": "opus" }),
        )]
        .into_iter()
        .collect(),
    }
}

#[test]
fn top_level_key_parity() {
    let schema = load_schema();
    let profile = serde_json::to_value(full_profile()).expect("serialize profile");

    let struct_keys = serialized_keys(&profile);
    // Schema-side-only keys with no `compiler::AgentProfile` counterpart by
    // design: `$schema`/`schema_version` are editor/migration metadata;
    // `isolation` is resolved at platform runtime (lives on
    // `config::AgentProfile`, not the compiler struct) and is cross-checked
    // separately by `isolation_enums_match_ship_isolation`.
    let metadata_allowlist: BTreeSet<String> =
        ["$schema", "schema_version", "isolation"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    let schema_keys: BTreeSet<String> = property_keys(&schema)
        .difference(&metadata_allowlist)
        .cloned()
        .collect();

    assert_eq!(
        struct_keys, schema_keys,
        "top-level key drift: {}",
        diff_report(&struct_keys, &schema_keys)
    );
}

#[test]
fn per_section_key_parity() {
    let schema = load_schema();
    let schema_props = schema
        .get("properties")
        .and_then(Value::as_object)
        .expect("schema has properties");
    let profile = serde_json::to_value(full_profile()).expect("serialize profile");
    let profile_obj = profile.as_object().expect("profile object");

    // Each Ship-owned nested object: its serialized struct keys must equal the
    // schema section's `properties` keys. Pins default_mode, hooks.*,
    // apps.studio, plugins.scope, and every other nested field exactly.
    // `harness_settings` is excluded — it is the opaque seam (covered by
    // `harness_settings_seam_matches_registry`).
    for section in [
        "agent", "skills", "mcp", "plugins", "permissions", "rules", "hooks", "apps",
    ] {
        let struct_keys = serialized_keys(
            profile_obj
                .get(section)
                .unwrap_or_else(|| panic!("profile has section `{section}`")),
        );
        let schema_keys = property_keys(
            schema_props
                .get(section)
                .unwrap_or_else(|| panic!("schema has section `{section}`")),
        );
        assert_eq!(
            struct_keys, schema_keys,
            "section `{section}` key drift: {}",
            diff_report(&struct_keys, &schema_keys)
        );
    }
}

#[test]
fn harness_settings_seam_matches_registry() {
    let schema = load_schema();
    let harness_node = schema
        .get("properties")
        .and_then(|p| p.get("harness_settings"))
        .expect("schema has harness_settings property");

    // The opaque seam exists on the struct too: `AgentProfile.harness_settings`
    // serializes as an object keyed by harness id.
    let profile = serde_json::to_value(full_profile()).expect("serialize profile");
    assert!(
        profile
            .get("harness_settings")
            .map(Value::is_object)
            .unwrap_or(false),
        "struct must carry harness_settings as an object"
    );

    // Authoritative harness id set — derived from the Rust REGISTRY via
    // `capabilities()` (single source of truth), NOT hardcoded. Future
    // gemini/pi-class drift between the schema and the registry fails here.
    let registry_ids: BTreeSet<String> =
        capabilities().into_iter().map(|(id, _)| id.to_string()).collect();
    let schema_harness_keys = property_keys(harness_node);

    assert_eq!(
        schema_harness_keys, registry_ids,
        "harness_settings key drift vs REGISTRY: {}",
        diff_report(&schema_harness_keys, &registry_ids)
    );
    // `additionalProperties: false` keeps the set closed — pin it so an
    // open seam can't silently re-admit a dropped harness.
    assert_eq!(
        harness_node.get("additionalProperties"),
        Some(&Value::Bool(false)),
        "harness_settings must stay a closed set"
    );
}

#[test]
fn isolation_enums_match_ship_isolation() {
    let schema = load_schema();
    let isolation = schema
        .get("properties")
        .and_then(|p| p.get("isolation"))
        .expect("schema has isolation property");

    let enum_ids = |node: &Value, field: &str| -> BTreeSet<String> {
        node.get("properties")
            .and_then(|p| p.get(field))
            .and_then(|f| f.get("enum"))
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("isolation.{field}.enum is an array"))
            .iter()
            .map(|v| v.as_str().expect("enum value is a string").to_string())
            .collect()
    };

    let schema_tiers = enum_ids(isolation, "tier");
    let rust_tiers: BTreeSet<String> =
        IsolationTier::ALL.iter().map(|t| t.as_id().to_string()).collect();
    assert_eq!(
        schema_tiers, rust_tiers,
        "isolation.tier.enum drift vs ship_isolation::IsolationTier: {}",
        diff_report(&schema_tiers, &rust_tiers)
    );

    let schema_strategies = enum_ids(isolation, "strategy");
    let rust_strategies: BTreeSet<String> = IsolationStrategy::ALL
        .iter()
        .map(|s| s.as_id().to_string())
        .collect();
    assert_eq!(
        schema_strategies, rust_strategies,
        "isolation.strategy.enum drift vs ship_isolation::IsolationStrategy: {}",
        diff_report(&schema_strategies, &rust_strategies)
    );
}
