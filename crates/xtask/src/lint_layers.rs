//! Lint the L0/L1/L2 crate-layer boundaries.
//!
//! Rules enforced (each emits a clear failure message and returns a non-zero
//! exit on the first detected violations batch):
//!
//! **R0 — Layer boundary (original):** no L0 crate may depend on an L1 crate.
//!
//! **R1 — No orphan crates:** every workspace member must be classifiable as
//!   L0, L1, or L2. An unclassifiable crate forces the taxonomy decision.
//!
//! **R2 — L0 has no RPC/transport:** no L0 crate may declare a direct Cargo
//!   dependency on `axum`, `connectrpc`, `http`, `tonic`, or `hyper`.
//!   Allowlist: `ship-ops-rpc`, `ship-sidecar-proto` (generated proto crates
//!   that legitimately exist to wrap these transport deps).
//!
//! **R3 — Runtime seam:** `floki` must not depend on `ship-code`, and
//!   `ship-code` must not depend on `floki`. Both may depend on `agent-def`;
//!   neither may depend on the other. If either crate is absent the rule
//!   passes vacuously (handles future branch states).
//!
//! Layer taxonomy (all derived from filesystem, never drifts):
//!   L0 = the `ship-substrate` tree (crate + every crate under its `crates/`)
//!        plus the sibling L0 primitives under `crates/core/` (ship-git,
//!        ship-isolation, ship-cloud, ship-ops-rpc, ship-gateway-client,
//!        ship-sidecar-proto, compiler) and the isolation sidecar binary.
//!   L1 = `ship-platform`, every crate under `ship-platform/crates/` (plus
//!        nested sub-crates one level deeper, e.g. `agent-def/crates/
//!        ship-skill-vars`), and `ship-pipeline-runtime`.
//!   L2 = application / tooling layer — crates under `apps/`, `crates/deck/`,
//!        `crates/sui/`, `packages/`, `tests/`, plus `crates/xtask` and
//!        `crates/build-info`.
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

/// Crates in the RPC/transport allowlist — they wrap generated proto bindings
/// and legitimately need axum/connectrpc/http as part of their public API.
const RPC_ALLOWLIST: &[&str] = &["ship-ops-rpc", "ship-sidecar-proto"];

/// Forbidden dep-key tokens for the L0-no-RPC/transport rule.
const RPC_FORBIDDEN_DEPS: &[&str] = &["axum", "connectrpc", "http", "tonic", "hyper"];

// ── Workspace member classification helpers ───────────────────────────────

/// Return `true` if the workspace-relative member path belongs to L0.
fn is_l0_path(member: &str) -> bool {
    // ship-substrate tree (root crate + all nested crates)
    if member == "crates/core/ship-substrate"
        || member.starts_with("crates/core/ship-substrate/")
    {
        return true;
    }
    // L0 sibling crates
    for sib in L0_SIBLINGS {
        let prefix = format!("crates/core/{sib}");
        if member == prefix || member.starts_with(&format!("{prefix}/")) {
            return true;
        }
    }
    false
}

/// Return `true` if the workspace-relative member path belongs to L1.
fn is_l1_path(member: &str) -> bool {
    member == "crates/core/ship-platform"
        || member.starts_with("crates/core/ship-platform/")
        || member == "crates/ship-pipeline-runtime"
}

/// Return `true` if the workspace-relative member path belongs to L2.
fn is_l2_path(member: &str) -> bool {
    member.starts_with("apps/")
        || member.starts_with("crates/deck/")
        || member.starts_with("crates/sui/")
        || member.starts_with("packages/")
        || member.starts_with("tests/")
        || member == "crates/xtask"
        || member == "crates/build-info"
}

/// Parse the `[workspace] members = [...]` list from the root Cargo.toml.
/// Returns workspace-relative path strings (the quoted values verbatim).
fn workspace_members(root_manifest: &Path) -> Result<Vec<String>, String> {
    let toml = fs::read_to_string(root_manifest)
        .map_err(|e| format!("read {}: {e}", root_manifest.display()))?;
    let mut in_workspace = false;
    let mut in_members = false;
    let mut members = Vec::new();
    for line in toml.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_workspace = t == "[workspace]";
            // Once we leave [workspace] stop scanning for members
            if !in_workspace {
                in_members = false;
            }
            continue;
        }
        if !in_workspace {
            continue;
        }
        if t.starts_with("members") && t.contains('=') {
            in_members = true;
        }
        if in_members {
            // Collect quoted strings
            let mut rest = t;
            while let Some(start) = rest.find('"') {
                rest = &rest[start + 1..];
                if let Some(end) = rest.find('"') {
                    members.push(rest[..end].to_string());
                    rest = &rest[end + 1..];
                }
            }
            // The list ends at the closing bracket
            if t.contains(']') {
                in_members = false;
            }
        }
    }
    Ok(members)
}

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

    let mut errors: Vec<String> = Vec::new();

    // ════════════════════════════════════════════════════════════════════════
    // R0 — Layer boundary: no L0 crate may depend on an L1 crate.
    // ════════════════════════════════════════════════════════════════════════
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
                    "[R0] {} (line {}): L0 crate `{}` depends on L1 crate `{}` — forbidden upward dep",
                    rel.display(),
                    lineno + 1,
                    self_name.as_deref().unwrap_or("?"),
                    key,
                ));
            } else if self_name.as_deref() == Some(key) {
                errors.push(format!(
                    "[R0] {} (line {}): declares itself as a dependency",
                    rel.display(),
                    lineno + 1,
                ));
            }
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // R1 — No orphan crates: every workspace member must be L0, L1, or L2.
    // ════════════════════════════════════════════════════════════════════════
    let root_manifest = repo_root.join("Cargo.toml");
    let members = workspace_members(&root_manifest)?;
    for member in &members {
        if !is_l0_path(member) && !is_l1_path(member) && !is_l2_path(member) {
            let name = crate_name(&repo_root.join(member).join("Cargo.toml"))
                .unwrap_or_else(|| "?".to_string());
            errors.push(format!(
                "[R1] workspace member `{member}` (crate `{name}`) is not classifiable as L0, L1, \
                 or L2 — add it to an existing tier or define a new tier in lint_layers.rs"
            ));
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // R2 — L0 has no RPC/transport deps (axum/connectrpc/http/tonic/hyper).
    //      Allowlist: ship-ops-rpc, ship-sidecar-proto.
    // ════════════════════════════════════════════════════════════════════════
    for manifest in &l0 {
        let self_name = crate_name(manifest).unwrap_or_default();
        // Allowlisted proto-codegen crates may carry these deps.
        if RPC_ALLOWLIST.contains(&self_name.as_str()) {
            continue;
        }
        let toml = fs::read_to_string(manifest)
            .map_err(|e| format!("read {}: {e}", manifest.display()))?;
        let rel = manifest.strip_prefix(repo_root).unwrap_or(manifest);
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
            // Exact-match the dep key against the forbidden set.
            if RPC_FORBIDDEN_DEPS.contains(&key) {
                errors.push(format!(
                    "[R2] {} (line {}): L0 crate `{}` depends on RPC/transport crate `{}` — \
                     L0 must not pull in axum/connectrpc/http/tonic/hyper; allowlist or move \
                     this crate to L1",
                    rel.display(),
                    lineno + 1,
                    self_name,
                    key,
                ));
            }
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // R3 — Runtime seam: floki ↔ ship-code must never cross-depend.
    //      Passes vacuously if either crate is absent (branch state).
    // ════════════════════════════════════════════════════════════════════════
    let floki_manifest = core.join("ship-platform/crates/floki/Cargo.toml");
    let ship_code_manifest = core.join("ship-platform/crates/ship-code/Cargo.toml");

    if floki_manifest.is_file() {
        if let Ok(toml) = fs::read_to_string(&floki_manifest) {
            let rel = floki_manifest.strip_prefix(repo_root).unwrap_or(&floki_manifest);
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
                if key == "ship-code" {
                    errors.push(format!(
                        "[R3] {} (line {}): runtime seam violated — `floki` depends on `ship-code`; \
                         the two runtimes must never depend on each other (both depend on agent-def)",
                        rel.display(),
                        lineno + 1,
                    ));
                }
            }
        }
    }

    if ship_code_manifest.is_file() {
        if let Ok(toml) = fs::read_to_string(&ship_code_manifest) {
            let rel = ship_code_manifest.strip_prefix(repo_root).unwrap_or(&ship_code_manifest);
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
                if key == "floki" {
                    errors.push(format!(
                        "[R3] {} (line {}): runtime seam violated — `ship-code` depends on `floki`; \
                         the two runtimes must never depend on each other (both depend on agent-def)",
                        rel.display(),
                        lineno + 1,
                    ));
                }
            }
        }
    }

    // ── Report ────────────────────────────────────────────────────────────
    if errors.is_empty() {
        println!(
            "layer boundaries clean ({} L0 manifests checked against {} L1 crates; {} workspace members classified)",
            l0.len(),
            l1.len(),
            members.len(),
        );
        return Ok(());
    }
    for e in &errors {
        eprintln!("  {e}");
    }
    Err(format!(
        "layer boundary violations: {} (R0={}, R1={}, R2={}, R3={})",
        errors.len(),
        errors.iter().filter(|e| e.starts_with("[R0]")).count(),
        errors.iter().filter(|e| e.starts_with("[R1]")).count(),
        errors.iter().filter(|e| e.starts_with("[R2]")).count(),
        errors.iter().filter(|e| e.starts_with("[R3]")).count(),
    ))
}
