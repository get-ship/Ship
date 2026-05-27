//! Build orchestration — the worker's `build`/`build_all`, in Rust (#163).
//!
//! Loops `build_target` over the resolved targets and runs catalog aggregation,
//! using located sui-compiler + esbuild binaries. Synchronous; the async
//! `Bundler` handle wraps these in `spawn_blocking` when it replaces the bun
//! worker.

use std::path::{Path, PathBuf};

use ship_substrate::ShipHome;

use super::build::{BuildError, build_target};
use super::catalog::{build_catalogs, stringify_catalog};
use super::compile::locate_sui_compiler;
use super::esbuild::locate_esbuild_binary;
use super::proto::BundleManifest;
use super::targets::BundleLayout;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("sui-compiler not found (runtime dir or target/)")]
    NoCompiler,
    #[error("esbuild not found (runtime dir or node_modules)")]
    NoEsbuild,
    #[error("unknown target: {0}")]
    UnknownTarget(String),
    #[error(transparent)]
    Build(#[from] BuildError),
}

/// The two located binaries the build needs.
pub struct BuildTools {
    pub compiler: PathBuf,
    pub esbuild: PathBuf,
}

impl BuildTools {
    /// Locate both: installed runtime copies first, dev fallbacks otherwise.
    pub fn locate(home: &ShipHome) -> Result<Self, ServiceError> {
        Ok(Self {
            compiler: locate_sui_compiler(home).ok_or(ServiceError::NoCompiler)?,
            esbuild: locate_esbuild_binary(home).ok_or(ServiceError::NoEsbuild)?,
        })
    }
}

/// Build a single named target into a fresh staging subdir.
pub fn build_one(
    layout: &BundleLayout,
    tools: &BuildTools,
    name: &str,
    staging: &Path,
    production: bool,
) -> Result<BundleManifest, ServiceError> {
    let targets = layout.all_targets();
    let target = targets
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| ServiceError::UnknownTarget(name.to_string()))?;
    Ok(build_target(
        layout,
        target,
        &tools.compiler,
        &tools.esbuild,
        &staging.join(staging_key(name)),
        production,
    )?)
}

/// Result of a `build_all` pass. Best-effort: a failing target records an error
/// and the loop continues, matching the worker's per-target try/catch.
#[derive(Debug, Default)]
pub struct BuildAllReport {
    pub built: Vec<BundleManifest>,
    pub errors: Vec<(String, String)>,
    /// `(workspace_count, published_count)` if catalog aggregation ran.
    pub catalog: Option<(usize, usize)>,
}

/// Build every target, then aggregate the catalog into `.deck/catalogs/`.
pub fn build_all(
    layout: &BundleLayout,
    tools: &BuildTools,
    staging: &Path,
    production: bool,
) -> BuildAllReport {
    let mut report = BuildAllReport::default();
    for target in layout.all_targets() {
        let dir = staging.join(staging_key(&target.name));
        match build_target(layout, &target, &tools.compiler, &tools.esbuild, &dir, production) {
            Ok(m) => report.built.push(m),
            Err(e) => report.errors.push((target.name.clone(), e.to_string())),
        }
    }
    match build_catalogs(&layout.repo_root) {
        Ok((ws, published)) => {
            let dir = layout.out_root.join(".deck/catalogs");
            if let Err(e) = std::fs::create_dir_all(&dir) {
                report.errors.push(("catalog".into(), e.to_string()));
            } else {
                let w = std::fs::write(dir.join("workspace.json"), stringify_catalog(&ws));
                let p = std::fs::write(dir.join("published.json"), stringify_catalog(&published));
                if let Err(e) = w.and(p) {
                    report.errors.push(("catalog".into(), e.to_string()));
                }
                report.catalog = Some((ws.count, published.count));
            }
        }
        Err(e) => report.errors.push(("catalog".into(), e)),
    }
    report
}

/// Filesystem-safe staging subdir name for a target (e.g. `views/git/view` →
/// `views_git_view`).
fn staging_key(name: &str) -> String {
    name.replace('/', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        // crates/core/ship-platform → walk up to the dir with package.json + Cargo.toml.
        let mut dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        loop {
            if dir.join("package.json").exists() && dir.join("packages").is_dir() {
                return dir;
            }
            if !dir.pop() {
                panic!("could not find repo root");
            }
        }
    }

    fn tools() -> BuildTools {
        BuildTools {
            compiler: crate::toolchain::install::walk_for_target_binary("sui-compiler")
                .expect("sui-compiler built"),
            esbuild: crate::toolchain::install::discover_esbuild().expect("esbuild in node_modules"),
        }
    }

    /// Build REAL framework + aggregate targets from the actual repo into a
    /// scratch out_root (never clobbers committed assets). Proves the real
    /// `sui-runtime` .ts entry and a real virtual aggregate (`sui-marketing`,
    /// which compiles real `.sui` primitives) bundle through the Rust pipeline.
    #[test]
    fn builds_real_framework_targets_to_scratch() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::new(repo, None).with_out_root(scratch.path().to_path_buf());
        let tools = tools();

        // sui-runtime: large real .ts entry; it IS the runtime so it is NOT
        // externalized — the bundle must inline its own code.
        let rt = build_one(&layout, &tools, "sui-runtime", staging.path(), true)
            .expect("sui-runtime should build");
        assert!(rt.bytes > 5_000, "runtime bundle should be substantial, got {}", rt.bytes);
        let rt_out = scratch.path().join("crates/deck/deck-core/assets/sui-runtime.js");
        assert!(rt_out.exists(), "runtime written to deck-core/assets");

        // sui-marketing: virtual aggregate of real primitive .sui sources —
        // exercises compile (multiple .sui) + generated entry + esbuild, with
        // the runtime kept external.
        let mk = build_one(&layout, &tools, "sui-marketing", staging.path(), true)
            .expect("sui-marketing should build");
        assert!(mk.bytes > 1_000, "marketing aggregate should be non-trivial");
        let mk_body = std::fs::read_to_string(scratch.path().join(".deck/sui-marketing.js")).unwrap();
        assert!(
            mk_body.contains("@ship/sui-runtime"),
            "aggregate keeps the runtime singleton external"
        );
    }

    /// Exploratory: build EVERY real target (umbrella, all islands, all views)
    /// + catalog into a scratch out_root, and print the report. Ignored by
    /// default (slow, whole-repo, source-state-dependent). Run:
    /// `cargo test -p ship-platform full_build_all -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn full_build_all_smoke() {
        let repo = repo_root();
        let scratch = tempfile::tempdir().unwrap();
        let staging = tempfile::tempdir().unwrap();
        let layout = BundleLayout::new(repo, None).with_out_root(scratch.path().to_path_buf());
        let report = build_all(&layout, &tools(), staging.path(), true);

        eprintln!("built {} targets", report.built.len());
        eprintln!("catalog: {:?}", report.catalog);
        eprintln!("errors ({}):", report.errors.len());
        for (name, err) in &report.errors {
            eprintln!("  ✘ {name}: {}", err.lines().next().unwrap_or(""));
        }
        // Leaf framework + views/islands/marketing build under the CLI model.
        // KNOWN-PENDING: the umbrella aggregates (sui-components, sui-chrome)
        // side-effect-import 119 transitive `./X.compiled.js` siblings that
        // need a RESOLVER (compile-on-resolve) — esbuild CLI has no plugin, so
        // they fail here. This is the A-vs-B decision point (see
        // .artifacts/bundler-seam.md): resolve via esbuild service protocol
        // (Rust onResolve/onLoad) or re-enumerate umbrellas as virtual aggregates.
        for core in ["sui-runtime", "sui-dom", "sui-sdk-client"] {
            assert!(
                report.built.iter().any(|m| m.target == core),
                "core target {core} must build"
            );
        }
    }
}
