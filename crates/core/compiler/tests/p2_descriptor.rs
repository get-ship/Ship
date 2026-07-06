//! Tests for the Phase-2 descriptor type growth on `ExtensionLibrary` and
//! `ResolvedConfig`. The job scope (per the J2 spec in
//! `census-phase2-resolver.md` Part 5) is additive only: every new field is
//! `#[serde(default)]` so existing descriptors round-trip byte-unchanged.
//!
//! Two contracts are pinned here:
//!
//! 1. The `load_library_fixture` round-trips byte-identically through serde
//!    JSON. This is the "old descriptors deserialize unchanged" acceptance
//!    criterion — it would FAIL if any new field used a non-`#[serde(default)]`
//!    attribute, leaked a non-serializing field, or got accidentally renamed.
//!
//! 2. Every new field round-trips through serde (set → serialize →
//!    deserialize → equal) so the Phase-2 fields the resolver will populate
//!    in J3+ survive a JSON boundary.
//!
//! Phase-3 placeholder fields (`rollup_inputs` and its `RollupInputs`
//! inner) are exercised the same way: type-only, never `Some` in production,
//! but the wire shape is the Phase-3 contract — it MUST round-trip.

use std::collections::BTreeMap;
use std::path::PathBuf;

use compiler::{
    load_library, ExtensionDescriptor, ExtSource, ModuleDescriptor,
    ExtensionLibrary, RollupInputs,
};
use serde_json::json;

fn fixture_ship_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("load_library_fixture")
        .join(".ship")
}

fn assert_byte_identical<T>(value: &T, label: &str)
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    // serde_json's `to_string` uses BTreeMap-backed key ordering via the
    // `preserve_order` feature — this is the same wire shape `compile`
    // emits. A byte-identical round-trip is the strictest form of "old
    // descriptors deserialize unchanged".
    let wire = serde_json::to_string(value).expect("serialize");
    let back: T = serde_json::from_str(&wire).expect("deserialize");
    let wire_again = serde_json::to_string(&back).expect("re-serialize");
    assert_eq!(
        wire, wire_again,
        "{label}: second serialization differs from first (round-trip not stable)"
    );
    // Compare via JSON value rather than `PartialEq` so this works for
    // types (like `ExtensionLibrary`) that don't derive it. JSON value
    // comparison is field-by-field after deserialization — a `null` field
    // going missing or a renamed key produces a different `Value` and
    // trips the assertion.
    let v1: serde_json::Value = serde_json::to_value(value).expect("to_value");
    let v2: serde_json::Value = serde_json::to_value(&back).expect("to_value(back)");
    assert_eq!(
        v1, v2,
        "{label}: deserialized value differs from original (field loss or rename)"
    );
}

// ─── Fixture round-trip ──────────────────────────────────────────────────────

#[test]
fn fixture_round_trips_byte_unchanged() {
    // The Phase-0 fixture ships with no Phase-2 fields populated. Loading
    // it through `load_library` then round-tripping through serde JSON must
    // produce identical bytes — this is the "old descriptors deserialize
    // unchanged" acceptance criterion for J2.
    let ship_dir = fixture_ship_dir();
    let lib = load_library(&ship_dir).expect("load_library must succeed");

    // Defaults for every new field must be the leaf's "absent" value:
    // empty strings/vecs, Value::Null for globals, None for Options.
    // `skill_paths` is the exception: the leaf's manifest read guarantees
    // the default project path is always present, even with no `ship.jsonc`
    // ("an empty manifest yields exactly the default").
    assert!(lib.skills_dir.is_none());
    assert!(lib.rules_dir.is_none());
    assert_eq!(lib.skill_paths, vec![compiler::loader::DEFAULT_SPACE_SKILL_PATH.to_string()]);
    assert!(lib.per_agent_skills.is_empty());
    assert!(lib.imported_modules.is_empty());
    assert!(lib.globals.is_null());
    assert!(lib.rollup_inputs.is_none());

    assert_byte_identical(&lib, "ExtensionLibrary");
}

#[test]
fn empty_library_round_trips_byte_unchanged() {
    // A `ExtensionLibrary::default()` (no fields set) must round-trip
    // identically — this is the "missing dir yields empty library"
    // extension of the round-trip contract.
    let lib = ExtensionLibrary::default();
    assert_byte_identical(&lib, "ExtensionLibrary::default");
}

// ─── New-field serde tests ───────────────────────────────────────────────────

#[test]
fn skills_dir_round_trips() {
    let mut lib = ExtensionLibrary::default();
    lib.skills_dir = Some("custom-skills/".to_string());
    assert_byte_identical(&lib, "ExtensionLibrary::skills_dir");
}

#[test]
fn rules_dir_round_trips() {
    let mut lib = ExtensionLibrary::default();
    lib.rules_dir = Some("custom-rules/".to_string());
    assert_byte_identical(&lib, "ExtensionLibrary::rules_dir");
}

#[test]
fn skill_paths_round_trips() {
    let mut lib = ExtensionLibrary::default();
    lib.skill_paths = vec![
        ".ship/skills/".to_string(),
        "vendor/sui/skills/".to_string(),
    ];
    assert_byte_identical(&lib, "ExtensionLibrary::skill_paths");
}

#[test]
fn per_agent_skills_round_trips() {
    let mut lib = ExtensionLibrary::default();
    let mut per_agent = BTreeMap::new();
    per_agent.insert(
        "shipwright".to_string(),
        vec![compiler::Skill {
            id: "shipwright-defaults".to_string(),
            name: "Shipwright defaults".to_string(),
            stable_id: None,
            description: Some("Per-agent skill for shipwright.".to_string()),
            license: None,
            compatibility: None,
            allowed_tools: vec![],
            metadata: Default::default(),
            content: "be precise".to_string(),
            source: Default::default(),
            vars: Default::default(),
            artifacts: vec![],
            grants: vec![],
            requires: vec![],
        }],
    );
    lib.per_agent_skills = per_agent;
    assert_byte_identical(&lib, "ExtensionLibrary::per_agent_skills");
}

#[test]
fn imported_modules_round_trips() {
    let mut lib = ExtensionLibrary::default();
    lib.imported_modules = vec![ModuleDescriptor {
        path: PathBuf::from("agents.ts"),
        source: "export const agents = [];\n".to_string(),
        content_hash: "blake3:deadbeef".to_string(),
        exports: vec!["agents".to_string()],
    }];
    assert_byte_identical(&lib, "ExtensionLibrary::imported_modules");
}

#[test]
fn globals_round_trips() {
    let mut lib = ExtensionLibrary::default();
    lib.globals = json!({
        "agentDefaults": { "model": "claude-sonnet-4-5" }
    });
    assert_byte_identical(&lib, "ExtensionLibrary::globals");
}

#[test]
fn rollup_inputs_placeholder_round_trips() {
    // The Phase-3 seam — type-only today, never Some in production. The
    // wire shape is the Phase-3 contract; the test pins that the placeholder
    // serializes the same way an empty RollupInputs will when Phase 3 lands.
    let mut lib = ExtensionLibrary::default();
    lib.rollup_inputs = Some(RollupInputs {
        extensions: vec![ExtensionDescriptor {
            source: ExtSource::Space,
            space_root: PathBuf::from("/projects/sui"),
            descriptor: ExtensionLibrary::default(),
            content_hash: "blake3:0000".to_string(),
        }],
        precedence: RollupInputs::default_precedence(),
    });
    assert_byte_identical(&lib, "ExtensionLibrary::rollup_inputs");
}

#[test]
fn missing_phase2_fields_use_defaults() {
    // A bare JSON object (no Phase-2 keys) must deserialize to a
    // ExtensionLibrary whose Phase-2 fields are the leaf defaults — this
    // proves #[serde(default)] is wired on every new field, not just
    // "defaults when struct-literal is constructed".
    let bare = json!({});
    let lib: ExtensionLibrary = serde_json::from_value(bare).expect("bare JSON must deserialize");
    assert!(lib.skills_dir.is_none());
    assert!(lib.rules_dir.is_none());
    assert!(lib.skill_paths.is_empty());
    assert!(lib.per_agent_skills.is_empty());
    assert!(lib.imported_modules.is_empty());
    assert!(lib.globals.is_null());
    assert!(lib.rollup_inputs.is_none());
}

#[test]
fn pre_phase2_json_payload_deserializes_unchanged() {
    // A JSON payload that contains ONLY pre-Phase-2 fields (the same shape
    // Phase-1 era callers would emit) must deserialize to a library whose
    // pre-Phase-2 fields match. Phase-2 fields default. This is the strict
    // read of the "old descriptors deserialize unchanged" contract.
    let payload = json!({
        "agents": [],
        "active_agent": null,
        "mcp_servers": [],
        "skills": [],
        "rules": [],
        "permissions": {
            "tools": { "allow": ["Read"], "deny": [], "ask": [] },
            "filesystem": { "allow": [], "deny": [] },
            "commands": { "allow": [], "deny": [] },
            "network": { "policy": "allow-list", "allow_hosts": [] },
            "agent": { "max_cost_per_session": null, "max_turns": null }
        },
        "hooks": [],
        "plugins": { "install": [], "scope": "project" },
        "claude_settings_extra": null,
        "agent_profiles": [],
        "claude_team_agents": [],
        "env": {},
        "available_models": [],
        "codex_sandbox": null,
        "model": null,
        "codex_approval_policy": null,
        "codex_reasoning_effort": null,
        "codex_max_threads": null,
        "codex_max_depth": null,
        "codex_job_max_runtime_seconds": null,
        "codex_shell_env_policy": null,
        "codex_notify": null,
        "codex_settings_extra": null,
        "opencode_settings_extra": null,
        "cursor_environment": null,
        "cursor_settings_extra": null,
        "claude_theme": null,
        "claude_auto_updates": null,
        "claude_include_co_authored_by": null,
        "studio_port": null,
    });
    let lib: ExtensionLibrary = serde_json::from_value(payload)
        .expect("Phase-1-shape JSON must deserialize to a Phase-2 ExtensionLibrary");
    assert!(lib.skills_dir.is_none());
    assert!(lib.imported_modules.is_empty());
    assert!(lib.rollup_inputs.is_none());
    // Pre-Phase-2 surface is unchanged:
    assert!(lib.agents.is_empty());
    assert!(lib.skills.is_empty());
    assert!(lib.mcp_servers.is_empty());
}
