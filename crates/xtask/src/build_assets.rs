//! `cargo xtask build-assets` — bake the framework bundles via the in-process
//! Rolldown bundler, the same engine the daemon runs. Replaces the retired
//! `bun packages/deck/bundler/index.ts --all` bake (#163), so the contributor
//! build no longer needs bun for bundling.
//!
//! `Bundler::build_all` writes the framework singletons + component umbrella +
//! catalog to `crates/deck/deck-core/assets` (`assets_dir_for`) and views /
//! islands to `.deck/`. `deck-core/build.rs` validates the framework set at
//! cargo-build time. `lucide-icons.json` is committed source (not bundling)
//! and is regenerated separately via `cargo xtask gen-icons`.

use std::path::Path;

use anyhow::{Context, Result, bail};
use deck::bundler::Bundler;

/// The framework bundles `deck-core/build.rs` bakes via `include_str!`. Verified
/// non-empty after the bake so a silently-failed target can't ship empty bytes
/// (the failure mode build.rs guards — top nav vanishing with no warning).
const FRAMEWORK_BUNDLES: &[&str] = &[
    "sui-runtime.js",
    "sui-dom.js",
    "sui-sdk-client.js",
    "sui-components.js",
];

pub fn run(repo_root: &Path) -> Result<()> {
    let cfg = deck::config::DeckConfig::load(repo_root);
    let bundler = Bundler::start(repo_root.to_path_buf(), None, cfg);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    rt.block_on(bundler.build_all())
        .map_err(|e| anyhow::anyhow!("build_all failed: {e}"))?;

    // build_all logs per-target failures but returns Ok, so verify the framework
    // bundles actually landed non-empty before declaring the bake good.
    let assets = repo_root.join("crates/deck/deck-core/assets");
    for name in FRAMEWORK_BUNDLES {
        let path = assets.join(name);
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if len == 0 {
            bail!("framework bundle missing or empty after bake: {}", path.display());
        }
        println!("  {name}: {len} bytes");
    }

    // The lean `sui-marketing.js` (apps/site's mounted `<sui-*>` subset) is a
    // Virtual aggregate that lands in `.deck/`, not deck-core/assets. It has no
    // build.rs guard, so a swallowed per-target failure (build_all only logs)
    // would silently ship a stale bundle to the site. Guard it here, same as the
    // framework set — non-empty after bake, or fail loud.
    let marketing = repo_root.join(".deck/sui-marketing.js");
    let marketing_len = std::fs::metadata(&marketing).map(|m| m.len()).unwrap_or(0);
    if marketing_len == 0 {
        bail!(
            "sui-marketing bundle missing or empty after bake: {} \
             (the sui-marketing Virtual target failed — check `bundler: target failed` logs)",
            marketing.display()
        );
    }
    println!("  sui-marketing.js: {marketing_len} bytes");

    println!("build-assets: framework bundles + catalog baked via in-process Rolldown");
    println!("note: lucide-icons.json is committed source; regen via `cargo xtask gen-icons`");
    Ok(())
}
