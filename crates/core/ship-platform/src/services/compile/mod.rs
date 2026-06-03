//! Compile service — the single shared compile path for the CLI and daemon.
//!
//! The daemon is the compile authority; the CLI is ambient. This service gives
//! native compile one shared home so the daemon can run it in-process.
//!
//! The compiler crate (transform) stays WASM-safe and has no platform deps.
//! This module owns the native-only loader (disk → `compiler::ProjectLibrary`)
//! and the full compile pipeline (library → written harness files).

pub mod agent;
pub mod loader;
pub mod mcp_allowlist;
pub mod permissions;
pub mod skill_parser;

#[cfg(test)]
mod tests;

pub use loader::{load_library, load_permission_preset, McpEntry, McpFile};
pub use permissions::PermissionPreset;
pub use skill_parser::load_skills;

use anyhow::{Context, Result};
use compiler::CompileOutput;
use std::path::{Path, PathBuf};

/// Deliver each included sdk-skill's `references/docs/*.mdoc` pages beside the
/// `SKILL.md` the compiler emitted. The compiler writes the (templated)
/// `SKILL.md`; its TOC links to per-op reference pages that live in the embedded
/// sdk-skill tree — copy them into the same harness directory so the links
/// resolve in the compiled output. No-op for authored skills (no embedded refs).
fn augment_with_sdk_references(output: &mut CompileOutput) {
    let dirs: Vec<String> = output
        .skill_files
        .keys()
        .filter_map(|k| k.strip_suffix("/SKILL.md").map(str::to_string))
        .collect();
    for dir in dirs {
        let id = dir.rsplit('/').next().unwrap_or("");
        for (rel, content) in skill_parser::embedded_skill_reference_files(id) {
            output.skill_files.insert(format!("{dir}/{rel}"), content);
        }
    }
}

use crate::registry::dep_skills::{resolve_dep_agents, resolve_dep_skills};

use agent::{apply_agent_to_library, collect_all_skill_refs};
use mcp_allowlist::{ensure_ship_mcp_allowed_claude, ensure_ship_mcp_allowed_cursor};

// ── Public types ─────────────────────────────────────────────────────────────

pub struct CompileOptions {
    /// Source root — where `.ship/` lives.
    pub project_root: PathBuf,
    /// Output root — where compiled files go. Defaults to `project_root`.
    pub output_root: Option<PathBuf>,
    /// Optionally restrict to a single harness.
    pub harness: Option<String>,
    /// Active agent id (already resolved from PathContext / ship.jsonc).
    pub active_agent: Option<String>,
    /// Extra skill IDs injected at compile time. Each must exist in the library.
    pub extra_skills: Vec<String>,
    /// Plugin IDs the active agent declared and that this compile installed.
    pub plugins_installed: Vec<String>,
    /// Compile for a daemon-spawned scoped launch — emit the authed-HTTP `ship`
    /// MCP entry instead of the trusted-local stdio shim.
    pub scoped_launch: bool,
    /// When true: resolve and compile but do not write files, pre-approve MCP
    /// tools, or emit `compile.completed`. Outputs are included in the returned
    /// summary so callers can dry-run-print them.
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct CompileSummary {
    /// Capability grants the resolved skill set requests.
    pub requested_grants: Vec<String>,
    /// Harnesses that were compiled (or would be compiled in dry-run mode).
    pub harnesses: Vec<String>,
    /// Per-harness compile outputs. Populated even for dry runs so callers can
    /// print what would be written.
    pub outputs: Vec<(String, CompileOutput)>,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn compile_project(opts: &CompileOptions) -> Result<CompileSummary> {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(&opts.project_root);
    let output_root = opts.output_root.as_deref().unwrap_or(&opts.project_root);

    // 1. Load raw library from .ship/ (flat layout)
    let mut library = load_library(&ship_dir).context("failed to load .ship/")?;

    // 2. Apply agent overrides (permissions, inline rules, harness list, plugins)
    if let Some(agent_id) = opts.active_agent.as_deref() {
        apply_agent_to_library(&mut library, agent_id, &opts.project_root)?;
    }
    library.active_agent = opts.active_agent.clone();

    // 3. Resolve dep skills from cache (github.com/ refs in ship.lock + cache)
    let all_skill_refs = collect_all_skill_refs(&library);
    if !all_skill_refs.is_empty() {
        let lock_path = ship_dir.join("ship.lock");
        let dep_skills = resolve_dep_skills(
            &all_skill_refs,
            &library.skills,
            &lock_path,
            None,
        )
        .context("resolving dep skills from cache")?;
        library.skills.extend(dep_skills);
    }

    // 3b. Compose dep agents from cache (each locked package's exports.agents),
    // namespaced by package path. Auto-compose: an installed dep's agents become
    // available in the space. (Composition engine #122; mirrors dep skills.)
    let dep_agents = resolve_dep_agents(&ship_dir.join("ship.lock"), None)
        .context("composing dep agents from cache")?;
    library.agent_profiles.extend(dep_agents);

    // 4. Inject extra skills into the active agent's skill filter
    if !opts.extra_skills.is_empty() {
        for skill_id in &opts.extra_skills {
            anyhow::ensure!(
                library.skills.iter().any(|s| &s.id == skill_id),
                "skill not found: {} — check skill_paths in ship.jsonc",
                skill_id
            );
        }
        if let Some(agent_id) = opts.active_agent.as_deref()
            && let Some(agent) = library.agents.iter_mut().find(|a| a.id == agent_id)
            && !agent.skills.is_empty()
        {
            for skill_id in &opts.extra_skills {
                if !agent.skills.contains(skill_id) {
                    agent.skills.push(skill_id.clone());
                }
            }
        }
    }

    // 5. Resolve (agent filtering, harness selection)
    let mut resolved =
        compiler::resolve_library(&library, None, opts.active_agent.as_deref());
    let ship_home = ship_substrate::ShipHome::from_env();
    resolved.ship_home_dir = Some(ship_home.root().to_string_lossy().into_owned());
    resolved.scoped_launch = opts.scoped_launch;

    let requested_grants = resolved.requested_grants.clone();

    // 6. Determine harnesses
    let harnesses: Vec<String> = match opts.harness.as_deref() {
        Some(h) => vec![h.to_string()],
        None => resolved.harnesses.clone(),
    };

    if harnesses.is_empty() {
        return Ok(CompileSummary {
            requested_grants,
            harnesses,
            outputs: vec![],
        });
    }

    // 7. Compile per harness
    let mut outputs = Vec::new();
    for harness_id in &harnesses {
        let Some(mut output) = compiler::compile(&resolved, harness_id) else {
            eprintln!("warning: unknown harness '{}', skipping", harness_id);
            continue;
        };
        augment_with_sdk_references(&mut output);
        outputs.push((harness_id.clone(), output));
    }

    // 8. Write + post-compile steps (skipped in dry-run mode)
    if !opts.dry_run {
        for (harness_id, output) in &outputs {
            compiler::write_output(output_root, harness_id, output)
                .with_context(|| format!("failed to write {} output", harness_id))?;
        }
        for harness_id in &harnesses {
            let result = match harness_id.as_str() {
                "claude" => ensure_ship_mcp_allowed_claude(),
                "cursor" => ensure_ship_mcp_allowed_cursor(output_root),
                _ => Ok(()),
            };
            if let Err(e) = result {
                eprintln!("warning: could not update {harness_id} MCP permissions: {e}");
            }
        }
        emit_compile_completed(
            &ship_dir,
            opts.active_agent.as_deref(),
            &harnesses,
            &opts.plugins_installed,
        );
    }

    Ok(CompileSummary {
        requested_grants,
        harnesses,
        outputs,
    })
}

// ── Event emission ────────────────────────────────────────────────────────────

fn emit_compile_completed(
    ship_dir: &Path,
    active_agent: Option<&str>,
    harnesses: &[String],
    plugins_installed: &[String],
) {
    use ship_substrate::events::EventEnvelope;
    use ship_substrate::events::types::{CompileCompleted, event_types::COMPILE_COMPLETED};
    use ship_substrate::persistence::event_store::SqliteEventStore;

    let space_id = match ship_substrate::project::ensure_space_id(ship_dir) {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("compile: cannot resolve project id: {e}");
            return;
        }
    };
    let payload = CompileCompleted {
        active_agent: active_agent.map(str::to_string),
        compiled_at: chrono::Utc::now().to_rfc3339(),
        ship_version: env!("CARGO_PKG_VERSION").to_string(),
        harness: harnesses.first().cloned(),
        plugins_installed: plugins_installed.to_vec(),
    };
    let envelope = match EventEnvelope::new(COMPILE_COMPLETED, &space_id, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("compile: cannot build envelope: {e}");
            return;
        }
    };
    // compile.* is Space-scoped → write the bound space's space.db under the
    // global ship-home (the db compile_state::load(space_id) reads). `space_id`
    // is the scope key.
    let store = match ship_substrate::project::get_global_dir().and_then(|root| {
        SqliteEventStore::new_for_scope(
            &root,
            ship_types::event_registry::EventScope::Space,
            Some(space_id.as_str()),
        )
    }) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("compile: cannot open event store: {e}");
            return;
        }
    };
    // compile.completed is Space-scoped: the binding must be bound to this
    // space so enforce_scope stamps/validates the space key. A daemon-internal
    // Space binding (the compile pass runs in the daemon, no session) carries it.
    let space = ship_types::identity::SpaceId::new(space_id.as_str());
    let binding = ship_types::ScopeBinding::daemon_internal(
        Some(space.clone()),
        ship_types::Authority::Space { space_id: space },
    );
    if let Err(e) = store.append_enforced(&envelope, &binding) {
        tracing::warn!("compile: cannot append compile.completed: {e}");
    }
}
