//! Boundary lint — the ship-standard / ship-code architecture invariant.
//!
//! The code world lives under `crates/code/` on disk (ship-code,
//! harness-config-compiler, and whatever joins them). Two graphs must never
//! carry it:
//!
//! 1. **Ship standard** — shipd with default features OFF. The standard
//!    dependency graph is the contract that lets the coding capability be
//!    sold, mounted, and audited as a separate surface.
//! 2. **Floki** — ship's native agent runtime. Floki and the
//!    third-party-harness world are two runtimes that must never
//!    cross-depend (lint-layers R3 holds the direct edges; this check
//!    holds the TRANSITIVE closure — the gap that let
//!    floki → agent-def → harness-config-compiler ride undetected while the
//!    emission crate was named `compiler`).
//!
//! Mechanism: the deny-set is derived from the WORKSPACE PATH — every
//! member whose manifest lives under `crates/code/` — not from a
//! hand-kept name list, so a new crate added to the code world is
//! covered the moment it exists. `ship-git` remains name-denied in the
//! standard graph (it lives outside `crates/code/` but is coding-class).

use std::path::Path;
use std::process::Command;

/// Workspace members whose manifests live under `crates/code/`.
fn code_world_members(repo_root: &Path) -> Result<Vec<String>, String> {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("cargo metadata failed to spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo metadata exited non-zero:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("cargo metadata parse: {e}"))?;
    let prefix = repo_root.join("crates/code/");
    let prefix = prefix.to_string_lossy();
    Ok(meta["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| {
            p["manifest_path"]
                .as_str()
                .is_some_and(|m| m.starts_with(prefix.as_ref()))
        })
        .filter_map(|p| p["name"].as_str().map(str::to_string))
        .collect())
}

fn tree_violations(
    repo_root: &Path,
    tree_args: &[&str],
    deny: &[String],
) -> Result<Vec<String>, String> {
    let out = Command::new("cargo")
        .args(tree_args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("cargo tree failed to spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo tree exited non-zero:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let tree = String::from_utf8_lossy(&out.stdout);
    Ok(tree
        .lines()
        .filter(|l| {
            let name = l.split_whitespace().next().unwrap_or("");
            deny.iter().any(|d| d == name)
        })
        .map(str::to_string)
        .collect())
}

pub fn run(repo_root: &Path) -> Result<(), String> {
    let mut deny = code_world_members(repo_root)?;
    if deny.is_empty() {
        return Err("no workspace members found under crates/code/ — the code world \
                    moved without updating this lint's path assumption"
            .to_string());
    }
    deny.push("ship-git".to_string());

    // 1. Ship standard: shipd, default features OFF.
    let standard = tree_violations(
        repo_root,
        &[
            "tree",
            "-p",
            "shipd",
            "--no-default-features",
            "--prefix",
            "none",
        ],
        &deny,
    )?;
    // 2. Floki's transitive closure.
    let floki = tree_violations(repo_root, &["tree", "-p", "floki", "--prefix", "none"], &deny)?;

    if standard.is_empty() && floki.is_empty() {
        println!(
            "boundary clean: standard + floki graphs carry none of crates/code/ \
             ({} denied crates) or ship-git",
            deny.len() - 1
        );
        return Ok(());
    }
    let mut msg = String::from(
        "the standard/code seam is an architecture rule — crates/code/ mounts \
         only behind the `ship-code` capability, and floki (the native runtime) \
         never crosses into the third-party-harness world:\n",
    );
    if !standard.is_empty() {
        msg.push_str(&format!(
            "\nship standard (shipd --no-default-features) carries:\n{}\n",
            standard.join("\n")
        ));
    }
    if !floki.is_empty() {
        msg.push_str(&format!(
            "\nfloki's closure carries:\n{}\n",
            floki.join("\n")
        ));
    }
    Err(msg)
}
