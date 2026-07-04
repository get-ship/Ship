//! `gen-agent-sdk` — emit the TypeScript `AgentConfig` type from the Rust SSOT.
//!
//! The agent descriptor type has ONE source of truth:
//! `compiler::types::config::AgentConfig`. This task exports that struct (and
//! its transitive named dependencies — `Capability`, `BlastRadius`,
//! `HookConfig`, `HookTrigger`, `PermissionConfig`) to a `.ts` file via specta,
//! so `@ship/sdk`'s `defineAgent` authors against the exact same shape the
//! compiler ingests. The TS is generated, never hand-written — regenerate on
//! any `AgentConfig` change.
//!
//! Outputs — ONE rendered string, written byte-identically to two locations so
//! there is a single source of truth and no hand-sync drift:
//!   1. `packages/sdk/src/generated/agent-config.ts` (ship-owned generated
//!      surface, build-materialized).
//!   2. `ship-exec/crates/ship-config/vendor/ship-sdk/agent-config.ts` (the
//!      config resolver's vendored `@ship/sdk` type stub, in the sibling
//!      `ship-exec` repo). `--check` gates both against the Rust SSOT.
//!
//! Usage:
//!   cargo xtask gen-agent-sdk          — regenerate the TS type (both copies)
//!   cargo xtask gen-agent-sdk --check  — verify both copies match (CI)

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use compiler::AgentConfig;
use specta::TypeCollection;
use specta_typescript::Typescript;

/// Ship-owned generated copy, relative to the ship repo root.
const OUT: &str = "packages/sdk/src/generated/agent-config.ts";

/// Vendored copy inside the sibling `ship-exec` repo, relative to its root.
/// Consumed by `ship-config` (the resolver's `@ship/sdk` type stub). Same
/// bytes as `OUT` — regenerating writes both, `--check` verifies both.
const VENDOR_REL: &str = "crates/ship-config/vendor/ship-sdk/agent-config.ts";

const HEADER: &str = "// AUTO-GENERATED from `compiler::types::config::AgentConfig` \
via `cargo xtask gen-agent-sdk`. DO NOT EDIT.";

/// Render the `AgentConfig` type bundle to a TypeScript string.
pub fn render() -> Result<String> {
    let mut types = TypeCollection::default();
    // `register` pulls in AgentConfig plus every transitively referenced named
    // type (Capability, BlastRadius, HookConfig, HookTrigger, PermissionConfig).
    types.register::<AgentConfig>();
    let body = Typescript::new()
        .header(HEADER)
        .export(&types)
        .context("specta export of AgentConfig")?;
    Ok(normalize(&body))
}

/// Strip per-line trailing whitespace and collapse to a single trailing
/// newline. specta emits trailing spaces; an editor that strips them on save
/// is exactly what split the ship-side and vendored copies. Normalizing here
/// makes the output byte-stable regardless of downstream formatters.
fn normalize(body: &str) -> String {
    let mut lines: Vec<&str> = body.lines().map(str::trim_end).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Resolve the sibling `ship-exec` repo root: `$SHIP_EXEC_PATH` if set, else
/// `../ship-exec` of the ship repo (the one-space `~/dev/Ship/` layout, and the
/// mirror of xtask-contracts' `$SHIP_PATH` → `../ship` convention).
fn ship_exec_root(repo_root: &Path) -> Option<PathBuf> {
    if let Ok(p) = std::env::var("SHIP_EXEC_PATH") {
        return Some(PathBuf::from(p));
    }
    repo_root.parent().map(|p| p.join("ship-exec"))
}

/// The paths to write/verify: the ship-side copy always, plus the ship-exec
/// vendored copy when that sibling repo is present. When it is absent (e.g. a
/// ship-only CI checkout) the vendored copy is skipped with a warning rather
/// than failing — set `$SHIP_EXEC_PATH` to enforce the cross-repo invariant.
fn targets(repo_root: &Path) -> Vec<PathBuf> {
    let mut out = vec![repo_root.join(OUT)];
    match ship_exec_root(repo_root) {
        Some(root) if root.is_dir() => out.push(root.join(VENDOR_REL)),
        Some(root) => eprintln!(
            "gen-agent-sdk: ship-exec not found at {} — skipping vendored copy \
             (set SHIP_EXEC_PATH to gate the cross-repo copy).",
            root.display()
        ),
        None => {}
    }
    out
}

pub fn run(repo_root: &Path, check: bool) -> Result<()> {
    let body = render()?;
    let targets = targets(repo_root);

    if check {
        let mut stale = false;
        for path in &targets {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing == body {
                eprintln!("gen-agent-sdk: {} up to date.", path.display());
            } else {
                eprintln!("gen-agent-sdk: STALE {}", path.display());
                stale = true;
            }
        }
        if stale {
            eprintln!("Run `cargo xtask gen-agent-sdk` and commit.");
            std::process::exit(1);
        }
        return Ok(());
    }

    for path in &targets {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        std::fs::write(path, &body)
            .with_context(|| format!("write {}", path.display()))?;
        eprintln!("gen-agent-sdk: wrote {}", path.display());
    }
    Ok(())
}
