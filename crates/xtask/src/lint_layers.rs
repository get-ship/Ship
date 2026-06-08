//! Lint the L0/L1 crate-layer boundary.
//!
//! Rule: **no L0 crate may depend on an L1 crate.**
//!   - L0 = the `ship-substrate` tree (the crate + every crate under its
//!     `crates/`) plus the sibling L0 primitives under `crates/core/`
//!     (ship-git, ship-isolation, ship-cloud, ship-ops-rpc,
//!     ship-gateway-client, ship-sidecar-proto, compiler).
//!   - L1 = `ship-platform`, every crate under `ship-platform/crates/` (plus
//!     nested domain sub-crates one level deeper, e.g.
//!     `ship-agents/crates/ship-skill-vars`), and `ship-pipeline-runtime`.
//!
//! Both sets are derived from the filesystem, so the check never drifts as
//! crates are added or moved. Keying violations on the dependency *name* (the
//! token left of `=` in a dep table) avoids false positives from prose that
//! merely mentions an L1 crate (e.g. a `description = "…ship-agents…"`).
//!
//! Catches the most common layer regression — a one-line dep addition that
//! wires the wrong direction. Run via `cargo xtask lint-layers`; CI fails on
//! any violation.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The `[package] name` declared in a Cargo.toml.
fn crate_name(manifest: &Path) -> Option<String> {
    let toml = fs::read_to_string(manifest).ok()?;
    let mut in_pkg = false;
    for line in toml.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_pkg = t == "[package]";
            continue;
        }
        if in_pkg && let Some(rest) = t.strip_prefix("name") {
            let v = rest.trim_start().strip_prefix('=')?.trim();
            return Some(v.trim_matches('"').to_string());
        }
    }
    None
}

/// Every `dir/*/Cargo.toml`.
fn child_manifests(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let m = e.path().join("Cargo.toml");
            if m.is_file() {
                out.push(m);
            }
        }
    }
    out.sort();
    out
}

const L0_SIBLINGS: &[&str] = &[
    "ship-git",
    "ship-isolation",
    "ship-cloud",
    "ship-ops-rpc",
    "ship-gateway-client",
    "ship-sidecar-proto",
    "compiler",
];

pub fn run(repo_root: &Path) -> Result<(), String> {
    let core = repo_root.join("crates/core");
    let substrate = core.join("ship-substrate");

    // ── L0 manifest set ─────────────────────────────────────────────────────
    let mut l0: Vec<PathBuf> = vec![substrate.join("Cargo.toml")];
    l0.extend(child_manifests(&substrate.join("crates")));
    for sib in L0_SIBLINGS {
        let m = core.join(sib).join("Cargo.toml");
        if m.is_file() {
            l0.push(m);
        }
    }
    l0.extend(child_manifests(&core.join("ship-isolation/bin")));

    // ── L1 forbidden set (derived) ────────────────────────────────────────────
    let mut l1: BTreeSet<String> = BTreeSet::new();
    l1.insert("ship-platform".to_string());
    l1.insert("ship-pipeline-runtime".to_string());
    for m in child_manifests(&core.join("ship-platform/crates")) {
        if let Some(n) = crate_name(&m) {
            l1.insert(n);
        }
        // Nested domain sub-crates one level deeper, e.g.
        // ship-agents/crates/ship-skill-vars.
        if let Some(crate_dir) = m.parent() {
            for sub in child_manifests(&crate_dir.join("crates")) {
                if let Some(n) = crate_name(&sub) {
                    l1.insert(n);
                }
            }
        }
    }

    // ── Check each L0 manifest's dep keys ──────────────────────────────────────
    let mut errors = Vec::new();
    for manifest in &l0 {
        let toml = fs::read_to_string(manifest)
            .map_err(|e| format!("read {}: {e}", manifest.display()))?;
        let rel = manifest.strip_prefix(repo_root).unwrap_or(manifest);
        let self_name = crate_name(manifest);
        let mut in_deps = false;
        for (lineno, line) in toml.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with('[') {
                in_deps = t.contains("dependencies");
                continue;
            }
            if !in_deps || t.starts_with('#') {
                continue;
            }
            let Some(eq) = t.find('=') else { continue };
            let key = t[..eq].trim();
            if key.is_empty() || key.contains(char::is_whitespace) {
                continue;
            }
            if l1.contains(key) {
                errors.push(format!(
                    "{} (line {}): L0 crate `{}` depends on L1 crate `{}` — forbidden upward dep",
                    rel.display(),
                    lineno + 1,
                    self_name.as_deref().unwrap_or("?"),
                    key,
                ));
            } else if self_name.as_deref() == Some(key) {
                errors.push(format!(
                    "{} (line {}): declares itself as a dependency",
                    rel.display(),
                    lineno + 1,
                ));
            }
        }
    }

    if errors.is_empty() {
        println!(
            "layer boundaries clean ({} L0 manifests checked against {} L1 crates)",
            l0.len(),
            l1.len(),
        );
        return Ok(());
    }
    for e in &errors {
        eprintln!("  {e}");
    }
    Err(format!("layer boundary violations: {}", errors.len()))
}
