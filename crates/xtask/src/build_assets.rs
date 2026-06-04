//! `cargo xtask build-assets` — thin shim to `cargo run -p deck-cli -- build-assets`.
//!
//! The bake logic (bundler invocation + honesty guards) now lives in
//! `deck-cli`'s `build-assets` subcommand. This file keeps the xtask entry
//! point alive for backward compat (Justfile / CI / dispatch.sh all use
//! `cargo xtask build-assets`) but delegates immediately so there is a single
//! implementation.

use std::path::Path;

use anyhow::{Context, Result};

pub fn run(repo_root: &Path) -> Result<()> {
    // Invoke `deck build-assets` in-process via the same build machinery
    // deck-cli uses. Using `std::process::Command` here would be clean but
    // would require the deck binary to be pre-built. Instead re-use the
    // deck_bundler + deck_config crates directly — same deps the CLI uses —
    // so `cargo xtask build-assets` doesn't need a separate `cargo build -p
    // deck-cli` step in the bootstrap.
    let home = ship_substrate::ShipHome::from_env();
    let cfg = deck::config::DeckConfig::load(repo_root);
    let bundler = deck::bundler::Bundler::start(&home, repo_root.to_path_buf(), None, cfg.clone());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    rt.block_on(bundler.build_all())
        .map_err(|e| anyhow::anyhow!("build_all failed: {e}"))?;

    // Framework-bundle honesty guard: non-empty after bake, or fail loud.
    const FRAMEWORK_BUNDLES: &[&str] = &[
        "sui-runtime.js",
        "sui-dom.js",
        "sui-sdk-client.js",
        "sui-components.js",
    ];
    let assets = repo_root.join("crates/deck/deck-core/assets");
    for name in FRAMEWORK_BUNDLES {
        let path = assets.join(name);
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            anyhow::bail!("framework bundle missing or empty after bake: {}", path.display());
        }
        println!("  {name}: {len} bytes");
    }

    // Consumer-bundle honesty guard: every target declared in deck.jsonc must
    // land non-empty. Uses BundleLayout for exact output-path routing, keeping
    // this guard in sync with the bundler's own routing logic.
    let layout = deck::bundler::targets::BundleLayout::with_config(
        repo_root.to_path_buf(),
        None,
        cfg,
    );
    for t in layout.consumer_targets() {
        let out_dir = layout.assets_dir_for(&t.name);
        let out_path = out_dir.join(&t.out_file);
        let len = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            anyhow::bail!(
                "consumer bundle missing or empty after bake: {} \
                 (target `{}` failed — check bundler logs)",
                out_path.display(),
                t.name
            );
        }
        println!("  {}: {len} bytes", t.name);
    }

    println!("build-assets: framework bundles + catalog baked via in-process Rolldown");
    println!("note: lucide-icons.json is committed source; regen via `cargo xtask gen-icons`");
    Ok(())
}
