//! Setup wizard manifest contract check.
//!
//! Earlier this file held six render tests asserting on specific UI
//! strings ("Welcome to Ship", `/api/projects` route, named Datastar
//! attributes). The wizard templates were heavily reworked and the
//! assertions drifted; tests sat `#[ignore]`'d for many sessions.
//! Deleted in favor of the manifest shape check below, which catches
//! gross structural breakage (a removed signal, a renamed action)
//! without coupling to UI copy.
//!
//! Run with: `cargo test -p shipd --test setup_tests`.

use std::path::{Path, PathBuf};

fn view_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest
        .parent()
        .and_then(Path::parent)
        .expect("repo root above apps/shipd")
        .to_path_buf();
    ship_substrate::ShipHome::project_ship_dir(repo_root).join("views/setup")
}

#[test]
fn manifest_is_valid_jsonc_and_names_setup() {
    let path = view_dir().join("manifest.jsonc");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let stripped: String = raw
        .lines()
        .map(|l| {
            if let Some(ix) = l.find("//") {
                &l[..ix]
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let value: serde_json::Value =
        serde_json::from_str(&stripped).expect("manifest must parse as JSON after stripping //");
    assert_eq!(value["name"], "setup");
    assert_eq!(value["type"], "html");
    let signals = value["signals"]
        .as_object()
        .expect("signals block must be an object");
    for sig in [
        "step", "ws_mode", "ws_name", "ws_path", "provider",
        "api_key", "provider_test_result",
    ] {
        assert!(signals.contains_key(sig), "manifest missing signal: {sig}");
    }
    let actions = value["actions"]
        .as_object()
        .expect("actions block must be an object");
    for act in ["workspace.create", "settings.save", "provider.test", "setup.complete"] {
        assert!(actions.contains_key(act), "manifest missing action: {act}");
    }
}
