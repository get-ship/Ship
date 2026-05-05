//! Cargo validates SUI bundle production.
//!
//! `include_str!` in `src/view.rs` bakes bundle outputs into the
//! framework binary at compile time. The bundles are produced by Node
//! scripts:
//!
//!   * `scripts/build-bundles.mjs` — esbuild for sui-runtime.js,
//!     sui-dom.js, sui-sdk-client.js, sui-components.js,
//!     sui-islands/*.js, and views/{agents,counter,git}/view.js
//!   * `packages/ship-set/scripts/generate-lucide-registry.mjs` —
//!     lucide-icons.json
//!
//! Those scripts run nested cargo (`cargo run -p sui-compiler`) so we
//! can NOT invoke them from build.rs — the inner cargo would deadlock
//! waiting on the parent's build lock. They run from `scripts/dev/
//! start.sh` (and the dev-mode hot-reload watcher in `dev_assets.rs`)
//! instead.
//!
//! What this build script DOES is verify the assets are populated.
//! Pre-existing failure mode: `cargo build` on a fresh clone wrote
//! empty placeholders, `include_str!` baked empty bytes, daemon shipped
//! with empty bundles, top nav vanished at runtime with no warning. We
//! now panic with a clear message so the failure is immediate and
//! actionable.
//!
//! On a fresh clone, run `pnpm install && node scripts/build-bundles.mjs
//! && node packages/ship-set/scripts/generate-lucide-registry.mjs`
//! before `cargo build`. `just dev-restart` does this for you.
//!
//! Cargo's `rerun-if-changed=` covers the asset paths plus every input
//! recorded in `apps/shipd/assets/.cache/*.inputs.json` so source
//! edits invalidate the cached binary correctly.
//!
//! Phase 1 of the move toward Rust-side bundling. Phase 2 lifts script
//! invocation into a `crates/sui-bundler/` crate; Phase 3 replaces
//! esbuild + nested-cargo sui-compiler with rolldown + an in-process
//! Rust compiler. See `.artifacts/job-spec-rolldown-migration.md`.

use std::fs;
use std::path::{Path, PathBuf};

const ASSETS: &[&str] = &[
    "sui-runtime.js",
    "sui-dom.js",
    "sui-sdk-client.js",
    "sui-components.js",
    "sui-islands/sui-terminal.js",
    "sui-islands/sui-command-bar.js",
    "sui-islands/sui-devtools.js",
    "sui-islands/sui-launcher.js",
    "sui-islands/sui-panel-dock.js",
    "sui-islands/sui-nav-top.js",
    "views/agents/view.js",
    "views/counter/view.js",
    "views/git/view.js",
    "lucide-icons.json",
];

const CUSTOM_ELEMENTS_JSON: &str = "sui-custom-elements.json";

const COARSE_SOURCE_ROOTS: &[&str] = &[
    "packages/ui-runtime/src",
    "packages/ui-dom/src",
    "packages/ui-components/src",
    "packages/ui-sdk-client/src",
    "scripts/build-bundles.mjs",
    "packages/ship-set/scripts/generate-lucide-registry.mjs",
];

fn main() {
    let manifest_dir: PathBuf = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR")
        .into();
    let assets = manifest_dir.join("assets");
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root (two levels above crates/ship-ui-framework)")
        .to_path_buf();

    fs::create_dir_all(assets.join("sui-islands"))
        .unwrap_or_else(|e| panic!("build.rs: mkdir sui-islands: {e}"));
    fs::create_dir_all(assets.join("views"))
        .unwrap_or_else(|e| panic!("build.rs: mkdir views: {e}"));

    // sui-custom-elements.json: empty-object is a valid CEM document
    // and a fine bootstrap value. include_str! always reads parseable
    // JSON; the bundler overwrites it with the real table.
    let cem_path = assets.join(CUSTOM_ELEMENTS_JSON);
    if !cem_path.exists() || file_is_empty(&cem_path) {
        fs::write(&cem_path, "{}")
            .unwrap_or_else(|e| panic!("build.rs: write {cem_path:?}: {e}"));
    }

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
             Bundles are produced by Node scripts that invoke nested cargo,\n\
             so cargo cannot orchestrate them from build.rs. Run:\n\n  \
             pnpm install\n  \
             node scripts/build-bundles.mjs\n  \
             node packages/ship-set/scripts/generate-lucide-registry.mjs\n\n\
             then re-run `cargo build`. `just dev-restart` does this for you."
        );
    }

    for rel in ASSETS {
        println!("cargo:rerun-if-changed=assets/{rel}");
    }
    println!("cargo:rerun-if-changed=assets/{CUSTOM_ELEMENTS_JSON}");

    let cache_dir = repo_root.join("apps/shipd/assets/.cache");
    if cache_dir.exists() {
        if let Ok(entries) = fs::read_dir(&cache_dir) {
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
