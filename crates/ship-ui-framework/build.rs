//! Ensure every gitignored asset that `include_str!` references in
//! `src/` exists at compile time. The Node bundle script
//! (`scripts/build-bundles.mjs`) overwrites these placeholders with
//! real bytes when it runs; until then, asset endpoints serve an empty
//! body — same outcome as the file being absent at runtime, but the
//! crate compiles on a fresh clone before bundles are built.
//!
//! Mirrors the placeholder pass in `apps/shipd/build.rs`. Kept local
//! to this crate so its `include_str!`s have authoritative bootstrap.

use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir: PathBuf = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR")
        .into();
    let assets = manifest_dir.join("assets");

    let placeholders: &[(&str, &str)] = &[
        ("sui-runtime.js", ""),
        ("sui-dom.js", ""),
        ("sui-sdk-client.js", ""),
        ("sui-components.js", ""),
        ("sui-islands/sui-terminal.js", ""),
        ("sui-custom-elements.json", "{}"),
    ];

    for (rel, initial) in placeholders {
        let path = assets.join(rel);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("build.rs: mkdir {parent:?}: {e}"));
        }
        fs::write(&path, initial)
            .unwrap_or_else(|e| panic!("build.rs: write placeholder {path:?}: {e}"));
    }

    // Tell cargo to rerun when assets change so a regenerated bundle
    // invalidates the embedded bytes.
    for (rel, _) in placeholders {
        println!("cargo:rerun-if-changed=assets/{rel}");
    }
}
