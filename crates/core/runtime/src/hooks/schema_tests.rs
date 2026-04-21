//! Tests that committed `.ship/hooks/*.jsonc` validate against the trigger schema
//! and that `.ship/views/*/manifest.json` validate against the view-manifest schema.

use jsonschema::validator_for;
use serde_json::Value;

fn load_json(path: &str) -> Value {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse {path}: {e}"))
}

/// Strip JSONC comments so the json parser can handle `.jsonc` files.
fn load_jsonc(path: &str) -> Value {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    compiler::jsonc::from_jsonc_str::<Value>(&content)
        .unwrap_or_else(|e| panic!("failed to parse JSONC {path}: {e}"))
}

/// Return workspace root — two levels up from `CARGO_MANIFEST_DIR`.
fn workspace_root() -> std::path::PathBuf {
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists() && p.join(".ship").exists())
        .expect("workspace root not found")
        .to_path_buf()
}

// ── trigger schema ───────────────────────────────────────────────────────────

#[test]
fn trigger_schema_is_valid_json_schema() {
    let root = workspace_root();
    let schema = load_json(&root.join("schemas/trigger.schema.json").to_string_lossy());
    validator_for(&schema).expect("trigger.schema.json must be a valid JSON Schema");
}

#[test]
fn committed_hooks_validate_against_trigger_schema() {
    let root = workspace_root();
    let schema_path = root.join("schemas/trigger.schema.json").to_string_lossy().into_owned();
    let schema = load_json(&schema_path);
    let validator = validator_for(&schema).expect("trigger schema must compile");

    let hooks_dir = root.join(".ship/hooks");
    let entries = std::fs::read_dir(&hooks_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", hooks_dir.display()));

    let mut validated = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "jsonc" && ext != "json" {
            continue;
        }
        let doc = load_jsonc(&path.to_string_lossy());
        let errors: Vec<_> = validator.iter_errors(&doc).collect();
        assert!(
            errors.is_empty(),
            "{} failed trigger schema validation:\n{}",
            path.display(),
            errors.iter().map(|e| format!("  - {e}")).collect::<Vec<_>>().join("\n")
        );
        validated += 1;
    }
    assert!(validated > 0, "expected at least one .jsonc hook file to validate");
}

// ── view-manifest schema ─────────────────────────────────────────────────────

#[test]
fn view_manifest_schema_is_valid_json_schema() {
    let root = workspace_root();
    let schema = load_json(&root.join("schemas/view-manifest.schema.json").to_string_lossy());
    validator_for(&schema).expect("view-manifest.schema.json must be a valid JSON Schema");
}

#[test]
fn committed_view_manifests_validate_against_view_manifest_schema() {
    let root = workspace_root();
    let schema_path = root.join("schemas/view-manifest.schema.json").to_string_lossy().into_owned();
    let schema = load_json(&schema_path);
    let validator = validator_for(&schema).expect("view-manifest schema must compile");

    let views_dir = root.join(".ship/views");
    let entries = std::fs::read_dir(&views_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", views_dir.display()));

    let mut validated = 0usize;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let doc = load_json(&manifest_path.to_string_lossy());
        let errors: Vec<_> = validator.iter_errors(&doc).collect();
        assert!(
            errors.is_empty(),
            "{} failed view-manifest schema validation:\n{}",
            manifest_path.display(),
            errors.iter().map(|e| format!("  - {e}")).collect::<Vec<_>>().join("\n")
        );
        validated += 1;
    }
    assert!(validated > 0, "expected at least one view manifest.json to validate");
}
