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
//! Output: `packages/sdk/src/generated/agent-config.ts` (ship-owned generated
//! surface). The `@ship/sdk` package vendors a copy for `defineAgent` typing.
//!
//! Usage:
//!   cargo xtask gen-agent-sdk          — regenerate the TS type
//!   cargo xtask gen-agent-sdk --check  — verify on-disk matches (CI)

use std::path::Path;

use anyhow::{Context, Result};
use compiler::AgentConfig;
use specta::TypeCollection;
use specta_typescript::Typescript;

const OUT: &str = "packages/sdk/src/generated/agent-config.ts";

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
    Ok(body)
}

pub fn run(repo_root: &Path, check: bool) -> Result<()> {
    let body = render()?;
    let out_path = repo_root.join(OUT);

    if check {
        let existing = std::fs::read_to_string(&out_path).unwrap_or_default();
        if existing != body {
            eprintln!("gen-agent-sdk: STALE {}", out_path.display());
            eprintln!("Run `cargo xtask gen-agent-sdk` and commit.");
            std::process::exit(1);
        }
        eprintln!("gen-agent-sdk: {} up to date.", out_path.display());
        return Ok(());
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(&out_path, &body)
        .with_context(|| format!("write {}", out_path.display()))?;
    eprintln!("gen-agent-sdk: wrote {}", out_path.display());
    Ok(())
}
