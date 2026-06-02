//! Agent activation — `ship use [<agent-id>]`.
//!
//! Compile state (active_agent, compiled_at, plugins_installed) is recorded
//! by the `compile.completed` event the compiler emits at the end of every
//! successful run, materialized into the `project_compile_state` projection.
//! `ship status` and the daemon read from there. No filesystem state file
//! and no per-project KV namespace.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use compiler::AgentProfile;
use crate::compile::{CompileOptions, run_compile};

// ── ProjectState ─────────────────────────────────────────────────────────────

/// Read view of `project_compile_state` for a given project. Fields are the
/// shape the CLI wants to display (`ship status`); the projection layer
/// stores whatever the latest `compile.completed` event carried.
#[derive(Debug, Clone, Default)]
pub struct ProjectState {
    pub active_agent: Option<String>,
    pub compiled_at: Option<String>,
    pub plugins_installed: Vec<String>,
}

impl ProjectState {
    /// Read the latest compile state for the project at `ship_dir`.
    /// Returns the default (all-None) when the project has never been
    /// compiled.
    pub fn load(ship_dir: &Path) -> Self {
        // The read is `compile_state::load` → `open_space_db`, which opens (and
        // self-provisions) the bounded `spaces/<id>/space.db` where the
        // Space-scoped `project_compile_state` projection lives. No platform.db
        // is touched here.
        let space_id = match ship_substrate::project::ensure_space_id(ship_dir) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("warning: could not resolve project id: {}", e);
                return ProjectState::default();
            }
        };
        match ship_platform::projections::compile_state::load(&space_id) {
            Ok(Some(row)) => ProjectState {
                active_agent: row.active_agent,
                compiled_at: Some(row.compiled_at),
                plugins_installed: row.plugins_installed,
            },
            Ok(None) => ProjectState::default(),
            Err(e) => {
                eprintln!("warning: could not read compile state: {}", e);
                ProjectState::default()
            }
        }
    }
}

// ── Activation ────────────────────────────────────────────────────────────────

/// Activate an agent: compile + install plugins. Compile state (active_agent,
/// plugins) is recorded by the `compile.completed` event into the Space-scoped
/// `project_compile_state` projection — no state file, no platform.db write.
/// If `agent_id` is None, re-runs the active agent read from that projection.
/// `extra_skills` are injected at compile time on top of the agent's declared skill refs.
pub fn activate_agent(
    agent_id: Option<&str>,
    project_root: &Path,
    output_root: Option<&Path>,
    extra_skills: Vec<String>,
    scoped: bool,
) -> Result<()> {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(project_root);
    let state = ProjectState::load(&ship_dir);
    let id = agent_id
        .map(str::to_string)
        .or_else(|| state.active_agent.clone())
        .ok_or_else(|| anyhow::anyhow!("No agent specified and no active agent set. Run: ship use <agent-id>"))?;

    let agent_path = match find_agent_file(&id, project_root) {
        Some(p) => p,
        None => {
            let available = list_available_agents(project_root);
            if available.is_empty() {
                anyhow::bail!(
                    "Agent '{}' not found in .ship/agents/. No agents found.\n\
                     Create one at .ship/agents/{}.jsonc or see: ship docs agents",
                    id,
                    id,
                );
            } else {
                anyhow::bail!(
                    "Agent '{}' not found in .ship/agents/. Available agents: {}\n\
                     Check the agent id or create .ship/agents/{}.jsonc",
                    id,
                    available.join(", "),
                    id,
                );
            }
        }
    };
    let agent = load_agent_profile(&agent_path)?;

    let now_plugins: Vec<String> = agent.plugins.install.clone();
    let prev_plugins = state.plugins_installed.clone();
    let scope = agent.plugins.scope.as_deref().unwrap_or("project");
    run_plugin_lifecycle(&now_plugins, &prev_plugins, scope);

    run_compile(CompileOptions {
        project_root,
        output_root,
        harness: None,
        dry_run: false,
        active_agent: Some(&id),
        extra_skills: extra_skills.clone(),
        plugins_installed: now_plugins,
        scoped_launch: scoped,
    })?;
    if extra_skills.is_empty() {
        println!("✓ activated agent '{}'", id);
    } else {
        let injected: Vec<String> = extra_skills.iter().map(|s| format!("+{}", s)).collect();
        println!("✓ activated agent '{}' ({})", id, injected.join(", "));
    }
    Ok(())
}

fn run_plugin_lifecycle(now: &[String], prev: &[String], scope: &str) {
    for plugin in now {
        if !prev.contains(plugin) {
            let status = std::process::Command::new("claude")
                .args(["plugin", "install", plugin, "--scope", scope])
                .status();
            match status {
                Ok(s) if s.success() => println!("  + plugin {}", plugin),
                Ok(_) => eprintln!("  warning: plugin install failed for {}", plugin),
                Err(_) => eprintln!(
                    "  warning: claude CLI not found — skipping plugin install for {}",
                    plugin
                ),
            }
        }
    }
    for plugin in prev {
        if !now.contains(plugin) {
            let status = std::process::Command::new("claude")
                .args(["plugin", "uninstall", plugin])
                .status();
            match status {
                Ok(s) if s.success() => println!("  - plugin {}", plugin),
                _ => eprintln!("  warning: plugin uninstall failed for {}", plugin),
            }
        }
    }
}

/// Resolve `<agent_id>.jsonc` under `.ship/agents/` (project first, then
/// user home). JSONC is the only accepted format — the TOML and
/// `profiles/`, `presets/`, `modes/` compat paths were retired with the
/// agent-schema unification.
pub fn find_agent_file(agent_id: &str, project_root: &Path) -> Option<PathBuf> {
    let ship = ship_substrate::ShipHome::project_ship_dir(project_root);
    let file = format!("{agent_id}.jsonc");

    let project_path = ship.join("agents").join(&file);
    if project_path.exists() {
        return Some(project_path);
    }

    let home = home::home_dir()?;
    let global_path = ship_substrate::ShipHome::project_ship_dir(home)
        .join("agents")
        .join(&file);
    if global_path.exists() {
        return Some(global_path);
    }

    None
}

/// Load an agent profile from a single file. JSONC only — TOML has been
/// retired. The schema matches `schemas/agent.schema.json` and is
/// expressed by [`compiler::AgentProfile`].
pub(crate) fn load_agent_profile(path: &Path) -> Result<AgentProfile> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("read agent file: {}", path.display()))?;
    compiler::jsonc::from_jsonc_str::<AgentProfile>(&content)
        .map_err(|e| anyhow::anyhow!("invalid agent JSONC at {}: {}", path.display(), e))
}

/// List agent IDs available in .ship/agents/ (for error messages).
fn list_available_agents(project_root: &Path) -> Vec<String> {
    let agents_dir = ship_substrate::ShipHome::project_ship_dir(project_root).join("agents");
    let mut ids = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && crate::paths::is_config_ext(&path) && crate::paths::is_agent_config(&path)
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            {
                ids.push(stem.to_string());
            }
        }
    }
    ids.sort();
    ids
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_file(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn setup_ship_dir(tmp: &TempDir) -> PathBuf {
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{ "id": "test-proj-id", "name": "test" }"#,
        )
        .unwrap();
        ship_dir
    }

    #[test]
    fn project_state_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        let state = ProjectState::load(&ship_dir);
        assert!(state.active_agent.is_none());
        assert!(state.plugins_installed.is_empty());
    }

    // The roundtrip test went away with WorkspaceState::save() — compile state
    // now flows through `compile.completed` events into the
    // `project_compile_state` projection. The projection's own tests cover
    // the round trip (see `ship-substrate::projections::compile_state::tests`).

    #[test]
    fn find_agent_file_finds_canonical_location() {
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            ".ship/agents/test.jsonc",
            r#"{ "agent": { "id": "test", "name": "Test" } }"#,
        );
        let found = find_agent_file("test", tmp.path());
        assert!(found.is_some());
        assert!(found.unwrap().to_string_lossy().ends_with("/test.jsonc"));
    }

    #[test]
    fn find_agent_file_ignores_toml() {
        // TOML is retired — a `.toml` file alone must not resolve.
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            ".ship/agents/test.toml",
            "[agent]\nname=\"Test\"\nid=\"test\"\n",
        );
        assert!(find_agent_file("test", tmp.path()).is_none());
    }

    #[test]
    fn find_agent_file_ignores_legacy_profiles_subdir() {
        // Legacy `agents/profiles/` is no longer searched.
        let tmp = TempDir::new().unwrap();
        write_file(
            tmp.path(),
            ".ship/agents/profiles/test.jsonc",
            r#"{ "agent": { "id": "test", "name": "Test" } }"#,
        );
        assert!(find_agent_file("test", tmp.path()).is_none());
    }

    #[test]
    fn find_agent_file_returns_none_for_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(find_agent_file("nonexistent", tmp.path()).is_none());
    }
}
