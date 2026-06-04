//! Cargo validates SUI bundle production.
//!
//! `include_str!` in `src/assets.rs` bakes bundle outputs into the
//! framework binary at compile time. Every asset under `assets/` is
//! gitignored — the build emits them on demand and `dispatch.sh`
//! bootstraps fresh worktrees by copying from the launching repo (or
//! rebuilding when that's empty). The framework bundles (sui-runtime,
//! sui-dom, sui-sdk-client, sui-components) are produced by
//! `cargo xtask build-assets` — the in-process Rolldown bundler (#163),
//! the same engine shipd runs in dev mode. The lucide icon set is
//! committed source (`assets/lucide-icons.json`) — the daemon embeds it
//! and serves per-icon at `/_sui/icons/{name}.svg`; regenerate after a
//! lucide-static version bump via `cargo run -p xtask -- gen-icons`.
//! The workspace catalog is emitted to `.deck/catalogs/workspace.json`
//! by `build-assets` and read at runtime by `src/packs.rs`.
//!
//! What this build script DOES is verify the assets are populated.
//! Pre-existing failure mode: `cargo build` on a fresh clone wrote
//! empty placeholders, `include_str!` baked empty bytes, daemon shipped
//! with empty bundles, top nav vanished at runtime with no warning. We
//! now panic with a clear message so the failure is immediate and
//! actionable.
//!
//! On a fresh clone without dispatch.sh, run `cargo xtask build-assets`
//! before `cargo build`. `lucide-icons.json` is committed source — it is
//! already present in a fresh clone.

use std::fs;
use std::path::{Path, PathBuf};

// deck's OWN baked product only. App/user bundles (views, chrome islands) are
// no longer here — they build to the project's `.deck/` and are read at runtime
// (see src/assets.rs::read_deck_asset). deck bakes only the runtime it ships.
const ASSETS: &[&str] = &[
    "sui-runtime.js",
    "sui-dom.js",
    "sui-sdk-client.js",
    "sui-components.js",
    "lucide-icons.json",
];

const COARSE_SOURCE_ROOTS: &[&str] = &[
    "packages/sui/runtime/src",
    "packages/sui/dom/src",
    "packages/components/ui-components/src",
    "apps/shell/src/components",
    "packages/deck/sdk-client/src",
];

fn main() {
    let manifest_dir: PathBuf = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR")
        .into();
    let assets = manifest_dir.join("assets");
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root (three levels above crates/deck/deck-core)")
        .to_path_buf();

    fs::create_dir_all(&assets)
        .unwrap_or_else(|e| panic!("build.rs: mkdir assets: {e}"));

    // Verify every required asset exists and is non-empty.
    let mut missing: Vec<&str> = Vec::new();
    let mut empty: Vec<&str> = Vec::new();
    for rel in ASSETS {
        let p = assets.join(rel);
        if !p.exists() {
            missing.push(rel);
        } else if file_is_empty(&p) {
            empty.push(rel);
        }
    }

    if !missing.is_empty() || !empty.is_empty() {
        let lines = missing
            .iter()
            .map(|r| format!("    - missing: {r}"))
            .chain(empty.iter().map(|r| format!("    - empty:   {r}")))
            .collect::<Vec<_>>()
            .join("\n");
        panic!(
            "build.rs: SUI bundle assets are missing or empty.\n{lines}\n\n\
             Most assets under crates/deck/deck-core/assets/ are gitignored\n\
             (lucide-icons.json is committed source — present in a fresh clone).\n\
             On fresh worktrees, dispatch.sh bootstraps the gitignored set;\n\
             for a standalone cargo build, regenerate by running (in order):\n\n  \
             cargo build -p sui-compiler   # the binary the bundler spawns\n  \
             cargo xtask build-assets   # framework bundles via in-process Rolldown\n\n\
             then re-run `cargo build`."
        );
    }

    for rel in ASSETS {
        println!("cargo:rerun-if-changed=assets/{rel}");
    }

    let cache_dir = repo_root.join("apps/shipd/assets/.cache");
    if cache_dir.exists()
        && let Ok(entries) = fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(inputs) = read_inputs(&p) {
                    for input in inputs {
                        let abs = repo_root.join(&input);
                        if abs.exists() {
                            println!("cargo:rerun-if-changed={}", abs.display());
                        }
                    }
                }
            }
        }

    for root in COARSE_SOURCE_ROOTS {
        let abs = repo_root.join(root);
        if abs.exists() {
            println!("cargo:rerun-if-changed={}", abs.display());
        }
    }
}

fn file_is_empty(p: &Path) -> bool {
    fs::metadata(p).map(|m| m.len() == 0).unwrap_or(true)
}

fn read_inputs(path: &Path) -> Result<Vec<String>, String> {
    // Manifest shape: { "inputs": [ "rel/path", ... ] }. Avoid pulling
    // serde_json into the build-script dep graph — parse the array
    // crudely. Build script doesn't run in adversarial contexts.
    let s = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let key = "\"inputs\"";
    let after_key = s.find(key).ok_or("no inputs key")?;
    let arr_start = s[after_key..].find('[').ok_or("no [ after inputs")? + after_key;
    let arr_end = s[arr_start..].find(']').ok_or("no ] for inputs")? + arr_start;
    let arr = &s[arr_start + 1..arr_end];
    let mut out = Vec::new();
    for raw in arr.split(',') {
        let trimmed = raw.trim();
        if trimmed.len() < 2 {
            continue;
        }
        let unquoted = trimmed
            .trim_start_matches('"')
            .trim_end_matches('"')
            .to_string();
        if !unquoted.is_empty() {
            out.push(unquoted);
        }
    }
    Ok(out)
}
