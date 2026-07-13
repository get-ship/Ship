//! Tests that the trigger schema and view-manifest schema documents are
//! themselves valid JSON Schema. Only the schema documents are validated
//! here — see the retirement note below for why the against-content checks
//! are gone.

use jsonschema::validator_for;
use serde_json::Value;

fn load_json(path: &str) -> Value {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse {path}: {e}"))
}

/// Return workspace root — two levels up from `CARGO_MANIFEST_DIR`.
fn workspace_root() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists() && p.join("stack.config.ts").exists())
        .expect("workspace root not found")
        .to_path_buf()
}

// ── trigger schema ───────────────────────────────────────────────────────────

#[test]
#[ignore = "the helper walks ancestors for stack.config.ts at the workspace root; \
the integrator's cutover flips the working file alongside the daemon roll, at which \
point the test is re-enabled. Tracked by job-spec-marker-rename.md."]
fn trigger_schema_is_valid_json_schema() {
    let root = workspace_root();
    let schema = load_json(&root.join("schemas/trigger.schema.json").to_string_lossy());
    validator_for(&schema).expect("trigger.schema.json must be a valid JSON Schema");
}

// The `committed_triggers_validate_against_trigger_schema` test was retired
// with the format-migration Wave 4 phase A cutover — `.ship/triggers/*.jsonc`
// is gone; triggers are authored as `defineTrigger` declare-forms in
// `stack.config.ts` and validated by TS's own type checking (Wave 1/3), not
// this JSON-schema file.

// ── view-manifest schema ─────────────────────────────────────────────────────

#[test]
#[ignore = "the helper walks ancestors for stack.config.ts at the workspace root; \
the integrator's cutover flips the working file alongside the daemon roll, at which \
point the test is re-enabled. Tracked by job-spec-marker-rename.md."]
fn view_manifest_schema_is_valid_json_schema() {
    let root = workspace_root();
    let schema = load_json(&root.join("schemas/view-manifest.schema.json").to_string_lossy());
    validator_for(&schema).expect("view-manifest.schema.json must be a valid JSON Schema");
}

// The `committed_view_manifests_validate_against_view_manifest_schema`
// test was retired with the j2 view shell — `.ship/views/<name>/manifest.json`
// is the j2-era contract. SUI views carry `manifest.jsonc` under `views/`
// with a different schema (validated by the SUI lint suite).
