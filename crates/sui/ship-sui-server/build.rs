//! Cargo validates SUI bundle production.
//!
//! `include_str!` in `src/assets.rs` bakes bundle outputs into the
//! framework binary at compile time. Every asset under `assets/` is
//! gitignored — the build emits them on demand and `dispatch.sh`
//! bootstraps fresh worktrees by copying from the launching repo (or
//! rebuilding when that's empty). Producers:
//!
//!   * `pnpm -F @ship/sui-runtime bundle` — sui-runtime.js
//!   * `pnpm -F @ship/sui-dom bundle`     — sui-dom.js
//!   * `pnpm -F @ship/sdk-client bundle`  — sui-sdk-client.js
//!   * `pnpm -F @ship/ui-components build` — sui-components.js,
//!     sui-islands/*.js, sui-custom-elements.json (via sui-build)
//!   * the in-daemon Bun worker at `~/.ship/runtime/bundler.ts` —
//!     refreshes everything above plus views/*/view.js while shipd
//!     runs in dev mode
//!   * `pnpm -F @ship/set icons` — lucide-icons.json
//!
//! What this build script DOES is verify the assets are populated.
//! Pre-existing failure mode: `cargo build` on a fresh clone wrote
//! empty placeholders, `include_str!` baked empty bytes, daemon shipped
//! with empty bundles, top nav vanished at runtime with no warning. We
//! now panic with a clear message so the failure is immediate and
//! actionable.
//!
//! On a fresh clone without dispatch.sh, run the producers above (the
//! convenience aggregate `pnpm -F @ship/sui-runtime bundle && pnpm -F
//! @ship/sui-dom bundle && pnpm -F @ship/sdk-client bundle && pnpm -F
//! @ship/ui-components build && pnpm -F @ship/set build && pnpm -F
//! @ship/set icons`) before `cargo build`.

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
    "packages/sui-runtime/src",
    "packages/sui-dom/src",
    "packages/ui-components/src",
    "packages/ship-sdk-client/src",
    "runtime-assets/bundler.ts",
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
        .and_then(|p| p.parent())
        .expect("repo root (three levels above crates/sui/ship-sui-server)")
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
             Every asset under crates/sui/ship-sui-server/assets/ is gitignored.\n\
             On fresh worktrees, dispatch.sh bootstraps them automatically.\n\
             For a standalone cargo build, regenerate by running (in order):\n\n  \
             pnpm install --frozen-lockfile\n  \
             pnpm -F @ship/sui-runtime bundle\n  \
             pnpm -F @ship/sui-dom bundle\n  \
             pnpm -F @ship/sdk-client bundle\n  \
             pnpm -F @ship/ui-components build\n  \
             pnpm -F @ship/set build\n  \
             pnpm -F @ship/set icons\n\n\
             then re-run `cargo build`."
        );
    }

    for rel in ASSETS {
        println!("cargo:rerun-if-changed=assets/{rel}");
    }
    println!("cargo:rerun-if-changed=assets/{CUSTOM_ELEMENTS_JSON}");

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
