//! `ship compile` — load the project library, resolve, compile, write.

mod agent;
mod mcp_allowlist;
pub(crate) mod output;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_agent;
#[cfg(test)]
mod tests_extra_skills;
#[cfg(test)]
mod tests_worktree;

use anyhow::{Context, Result};
use compiler::{compile, resolve_library};
use std::path::Path;

use crate::dep_skills::resolve_dep_skills;
use crate::loader::load_library;

use agent::{apply_agent_to_library, collect_all_skill_refs};
use mcp_allowlist::{
    ensure_ship_mcp_allowed_claude, ensure_ship_mcp_allowed_cursor, ensure_ship_mcp_allowed_gemini,
};
use output::{ensure_session_gitignored, print_dry_run};

// Re-export the public API so `use crate::compile::{CompileOptions, run_compile}` works.
pub use output::write_output;

// ── Entry point ──────────────────────────────────────────────────────────────

pub struct CompileOptions<'a> {
    /// Source root — where .ship/ lives (may be the main repo for worktrees).
    pub project_root: &'a Path,
    /// Output root — where compiled files (CLAUDE.md, .claude/, .mcp.json) go.
    /// Defaults to project_root. Set to cwd for worktree compilation.
    pub output_root: Option<&'a Path>,
    /// Optionally restrict to a single provider.
    pub provider: Option<&'a str>,
    /// Print what would be written without touching the filesystem.
    pub dry_run: bool,
    /// Active mode id (already resolved from PathContext / ship.toml).
    pub active_agent: Option<&'a str>,
    /// Extra skill IDs injected at compile time via `--with`. Additive on top of
    /// the agent's declared skill refs. Each ID must exist in the loaded library.
    pub extra_skills: Vec<String>,
    /// Plugin IDs the active agent declared and that this compile installed.
    /// Recorded on the emitted `compile.completed` event so the
    /// `project_compile_state` projection knows which plugins are live.
    pub plugins_installed: Vec<String>,
}

pub fn run_compile(opts: CompileOptions<'_>) -> Result<()> {
    let ship_dir = runtime::ShipHome::project_ship_dir(&opts.project_root);
    let output_root = opts.output_root.unwrap_or(opts.project_root);

    // 1. Load raw library from .ship/ (flat layout)
    let mut library = load_library(&ship_dir).context("failed to load .ship/")?;

    // 2. Apply mode overrides (permissions, inline rules, provider list)
    if let Some(mode_id) = opts.active_agent {
        apply_agent_to_library(&mut library, mode_id, opts.project_root)?;
    }
    library.active_agent = opts.active_agent.map(str::to_string);

    // 2b. Resolve dep skill refs from cached packages.
    //     Collect all skill refs declared across agent profiles and mode configs,
    //     resolve any github.com/ refs from ship.lock + cache, and merge into
    //     library.skills before passing to the compiler. Local refs are skipped.
    {
        let all_skill_refs = collect_all_skill_refs(&library);
        if !all_skill_refs.is_empty() {
            let lock_path = runtime::ShipHome::project_ship_dir(&opts.project_root).join("ship.lock");
            let dep_skills = resolve_dep_skills(
                &all_skill_refs,
                &library.skills,
                &lock_path,
                None, // use default ~/.ship/cache/
            )
            .context("resolving dep skills from cache")?;
            library.skills.extend(dep_skills);
        }
    }

    // 2c. Inject --with extra skills into the active mode's skill filter.
    //     Each extra ID must exist in the loaded library (validates at inject time).
    //     If the active mode has a non-empty skill filter, append the extra IDs so
    //     they pass resolve's retain filter. If the filter is empty (include-all),
    //     the skills are already present — no mode mutation needed.
    if !opts.extra_skills.is_empty() {
        for skill_id in &opts.extra_skills {
            anyhow::ensure!(
                library.skills.iter().any(|s| &s.id == skill_id),
                "skill not found: {} — check skill_paths in ship.jsonc",
                skill_id
            );
        }
        // Find the active agent (if any) and extend its skill filter.
        if let Some(agent_id) = opts.active_agent
            && let Some(agent) = library.agents.iter_mut().find(|a| a.id == agent_id)
            && !agent.skills.is_empty()
        {
            // If agent.skills is empty, all skills are included — nothing to extend.
            for skill_id in &opts.extra_skills {
                if !agent.skills.contains(skill_id) {
                    agent.skills.push(skill_id.clone());
                }
            }
        }
    }

    // 3. Resolve (mode filtering, provider selection)
    let mut resolved = resolve_library(&library, None, opts.active_agent);
    // Inject the resolved ship home so the compiler can emit MCP entries
    // with `SHIP_HOME` without reading env directly (it is WASM-safe).
    let ship_home: runtime::ShipHome = runtime::ShipHome::from_env();
    resolved.ship_home_dir = Some(ship_home.root().to_string_lossy().into_owned());

    // 4. Determine providers to compile for
    let providers: Vec<String> = match opts.provider {
        Some(p) => vec![p.to_string()],
        None => resolved.providers.clone(),
    };

    if providers.is_empty() {
        println!("No providers configured. Add providers to .ship/ship.toml or mode file.");
        return Ok(());
    }

    // 5. Compile + write (output goes to output_root, not project_root)
    for provider_id in &providers {
        let Some(output) = compile(&resolved, provider_id) else {
            eprintln!("warning: unknown provider '{}', skipping", provider_id);
            continue;
        };
        if opts.dry_run {
            print_dry_run(provider_id, &output);
        } else {
            write_output(output_root, provider_id, &output)
                .with_context(|| format!("failed to write {} output", provider_id))?;
        }
    }

    // 6. Pre-approve ship MCP tools globally for each provider.
    if !opts.dry_run {
        for provider in &providers {
            let result = match provider.as_str() {
                "claude" => ensure_ship_mcp_allowed_claude(),
                "gemini" => ensure_ship_mcp_allowed_gemini(),
                "cursor" => ensure_ship_mcp_allowed_cursor(output_root),
                _ => Ok(()),
            };
            if let Err(e) = result {
                eprintln!("warning: could not update {provider} MCP permissions: {e}");
            }
        }
    }

    if !opts.dry_run {
        ensure_session_gitignored(output_root)?;
        emit_compile_completed(&ship_dir, opts.active_agent, &providers, &opts.plugins_installed);
        if opts.extra_skills.is_empty() {
            println!("✓ compiled for: {}", providers.join(", "));
        } else {
            let injected: Vec<String> = opts.extra_skills.iter().map(|s| format!("+{}", s)).collect();
            println!("✓ compiled for: {} ({})", providers.join(", "), injected.join(", "));
        }
    }
    Ok(())
}

/// Emit a `compile.completed` event recording what this compile produced.
/// The latest event for each project_id materializes into the
/// `project_compile_state` projection. Best-effort — any failure is logged
/// but doesn't fail the compile (the user has working output regardless).
fn emit_compile_completed(
    ship_dir: &Path,
    active_agent: Option<&str>,
    providers: &[String],
    plugins_installed: &[String],
) {
    use runtime::events::EventEnvelope;
    use runtime::events::types::{CompileCompleted, event_types::COMPILE_COMPLETED};
    use runtime::persistence::event_store::{EventStore, SqliteEventStore};

    let project_id = match runtime::project::ensure_project_id(ship_dir) {
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
        provider: providers.first().cloned(),
        plugins_installed: plugins_installed.to_vec(),
    };
    let envelope = match EventEnvelope::new(COMPILE_COMPLETED, &project_id, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("compile: cannot build envelope: {e}");
            return;
        }
    };
    let store = match SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("compile: cannot open event store: {e}");
            return;
        }
    };
    if let Err(e) = store.append(&envelope) {
        tracing::warn!("compile: cannot append compile.completed: {e}");
    }
}
