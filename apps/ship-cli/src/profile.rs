//! Agent activation — `ship use [<agent-id>]`.
//!
//! Compile state (active_agent, compiled_at, plugins_installed) is recorded
//! by the `compile.completed` event the compiler emits at the end of every
//! successful run, materialized into the `project_compile_state` projection.
//! `ship status` and the daemon read from there. No filesystem state file
//! and no per-space KV namespace.
use anyhow::Result;
use std::path::{Path, PathBuf};

use stack_model::AgentProfile;
use crate::compile::{CompileOptions, run_compile};

// ── SpaceState ────────────────────────────────────────────────────────────────

/// Read view of `project_compile_state` for a given space. Fields are the
/// shape the CLI wants to display (`ship status`); the projection layer
/// stores whatever the latest `compile.completed` event carried.
#[derive(Debug, Clone, Default)]
pub struct SpaceState {
    pub active_agent: Option<String>,
    pub compiled_at: Option<String>,
    pub plugins_installed: Vec<String>,
}

impl SpaceState {
    /// Read the latest compile state for the space at `ship_dir`.
    /// Returns the default (all-None) when the space has never been
    /// compiled.
    pub fn load(ship_dir: &Path) -> Self {
        // The read is `compile_state::load` → `open_space_db`, which opens (and
        // self-provisions) the bounded `spaces/<id>/space.db` where the
        // Space-scoped `project_compile_state` projection lives. The space id
        // comes from the `projects` pointer; an unregistered space has no
        // compile state (space-less → default).
        let probe = ship_dir.parent().unwrap_or(ship_dir);
        let Some(space_id) = ship_substrate::project::resolve_active_space(probe) else {
            return SpaceState::default();
        };
        match agent_def::compile::projection::load(&space_id) {
            Ok(Some(row)) => SpaceState {
                active_agent: row.active_agent,
                compiled_at: Some(row.compiled_at),
                plugins_installed: row.plugins_installed,
            },
            Ok(None) => SpaceState::default(),
            Err(e) => {
                eprintln!("warning: could not read compile state: {}", e);
                SpaceState::default()
            }
        }
    }
}

// ── Activation ────────────────────────────────────────────────────────────────

/// Activate an agent: compile + install plugins. Compile state (active_agent,
/// plugins) is recorded by the `compile.completed` event into the Space-scoped
/// `project_compile_state` projection — no state file involved.
pub fn activate_agent(
    agent_id: Option<&str>,
    space_root: &Path,
    output_root: Option<&Path>,
    extra_skills: Vec<String>,
    scoped: bool,
) -> Result<()> {
    let state = SpaceState::load(space_root); // stack root anchors state
    let id = agent_id
        .map(str::to_string)
        .or_else(|| state.active_agent.clone())
        .ok_or_else(|| anyhow::anyhow!("No agent specified and no active agent set. Run: ship use <agent-id>"))?;

    let agent_path = match find_agent_file(&id, space_root) {
        Some(p) => p,
        None => {
            let available = list_available_agents(space_root);
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
        space_root,
        output_root,
        harness: None,
        dry_run: false,
        active_agent: Some(&id),
        extra_skills: extra_skills.clone(),
        plugins_installed: now_plugins,
        scoped_launch: scoped,
        default_effort: None,
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

/// Resolve `<agent_id>.jsonc` under `.ship/agents/` (space first, then
/// user home). JSONC is the only accepted format — the TOML and
/// `profiles/`, `presets/`, `modes/` compat paths were retired with the
/// agent-schema unification.
pub fn find_agent_file(agent_id: &str, space_root: &Path) -> Option<PathBuf> {
    let file = format!("{agent_id}.jsonc");

    // agents/ is a domain dir at the STACK ROOT (marker cutover).
    let space_path = space_root.join("agents").join(&file);
    if space_path.exists() {
        return Some(space_path);
    }

    let home = home::home_dir()?;
    // User-global agents live under the legacy home dir shape.
    let global_path = home.join(".ship")
        .join("agents")
        .join(&file);
    if global_path.exists() {
        return Some(global_path);
    }

    None
}

/// Load an agent profile from a single file. JSONC only — TOML has been
/// retired. The schema matches `schemas/agent.schema.json` and is
/// expressed by [`stack_model::AgentProfile`]. Reads through THE agent-def reader
/// (FsBroker, ship-config via ship-platform) — never raw `std::fs`.
/// Load an agent profile from a single file. JSONC only — TOML has been
/// retired. The schema matches `schemas/agent.schema.json` and is
/// expressed by [`stack_model::AgentProfile`]. Reads through THE agent-def reader
/// (FsBroker, ship-config via ship-platform) — never raw `std::fs`.
///
/// On parse/schema failure (the file is present but the JSONC doesn't fit the
/// agent schema), the error names the file, says it isn't valid agent config,
/// and points the user at JSONC syntax + the top-level `agent` block — never
/// a raw serde/jsonc trace.
pub(crate) fn load_agent_profile(path: &Path) -> Result<AgentProfile> {
    let fs = ship_substrate::ports::LocalFsBroker::new();
    match ship_platform::config::read_agent_profile_at(&fs, path) {
        Ok(Some(profile)) => Ok(profile),
        Ok(None) => Err(anyhow::anyhow!(
            "agent config at {} is not valid agent config. \
             Check JSONC syntax (comments, trailing commas, unmatched braces) \
             and that the top-level `agent` block matches schemas/agent.schema.json.",
            path.display()
        )),
        Err(e) => Err(anyhow::anyhow!("read agent file {}: {}", path.display(), e)),
    }
}

/// List agent IDs available in .ship/agents/ (for error messages). Sourced
/// through THE agent-def reader (FsBroker) — no raw `std::fs` walk.
fn list_available_agents(space_root: &Path) -> Vec<String> {
    let agents_dir = space_root.join("agents");
    let fs = ship_substrate::ports::LocalFsBroker::new();
    let mut ids: Vec<String> = ship_platform::config::read_agent_docs_in(&fs, &agents_dir)
        .unwrap_or_default()
        .into_iter()
        .map(|(stem, _)| stem)
        .collect();
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
        let ship_dir = tmp.path().to_path_buf();
        std::fs::create_dir_all(&ship_dir).unwrap();
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            r#"{ "id": "test-proj-id", "name": "test" }"#,
        )
        .unwrap();
        ship_dir
    }

    #[test]
    fn space_state_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let ship_dir = setup_ship_dir(&tmp);
        let state = SpaceState::load(&ship_dir);
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
            "agents/test.jsonc",
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
