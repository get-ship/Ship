//! Agent profile application — merging an active agent's overrides into the
//! project library. The library has already been populated with every
//! `.ship/agents/*.jsonc` by [`super::loader::load_library`]; this step
//! pulls the active one out and merges its permissions / hooks / plugins /
//! harness settings, then injects a flat `AgentConfig` so `resolve()` can
//! filter harnesses and MCP/skill refs against it.

use anyhow::Result;
use compiler::{HookConfig, HookTrigger, PluginEntry, PluginsManifest, ProjectLibrary};
use std::path::Path;

use super::permissions::apply_agent_permissions;

// ── Agent → library ──────────────────────────────────────────────────────────

pub fn apply_agent_to_library(
    library: &mut ProjectLibrary,
    agent_id: &str,
    project_root: &Path,
) -> Result<()> {
    let Some(profile) = library
        .agent_profiles
        .iter()
        .find(|p| p.agent.id == agent_id)
        .cloned()
    else {
        return Ok(());
    };

    let ship_dir = ship_substrate::ShipHome::project_ship_dir(project_root);

    library.permissions =
        apply_agent_permissions(library.permissions.clone(), &profile, Some(&ship_dir));

    if let Some(inline) = &profile.rules.inline {
        let trimmed = inline.trim();
        if !trimmed.is_empty() {
            library.rules.push(compiler::Rule {
                file_name: format!("{agent_id}.md"),
                content: trimmed.to_string(),
                always_apply: true,
                globs: vec![],
                description: None,
            });
        }
    }

    // Inject a flat AgentConfig so resolve() can apply MCP / skill / rule
    // filtering from the active profile.
    let mut cfg = profile.to_config();
    cfg.id = agent_id.to_string();
    library.agents.push(cfg);

    if !profile.plugins.install.is_empty() {
        library.plugins = PluginsManifest {
            install: profile
                .plugins
                .install
                .iter()
                .map(|id| PluginEntry {
                    id: id.clone(),
                    harness: "claude".to_string(),
                })
                .collect(),
            scope: profile.plugins.scope.clone().unwrap_or_else(|| "project".to_string()),
        };
    }

    if let Some(cmd) = &profile.hooks.stop {
        push_hook(library, format!("{agent_id}-stop"), HookTrigger::Stop, cmd);
    }
    if let Some(cmd) = &profile.hooks.subagent_stop {
        push_hook(
            library,
            format!("{agent_id}-subagent-stop"),
            HookTrigger::SubagentStop,
            cmd,
        );
    }

    if let Some(v) = profile.harness_settings.get("claude") {
        library.claude_settings_extra = Some(v.clone());
    }
    if let Some(v) = profile.harness_settings.get("codex") {
        library.codex_settings_extra = Some(v.clone());
    }
    if let Some(v) = profile.harness_settings.get("cursor") {
        library.cursor_settings_extra = Some(v.clone());
    }

    library.claude_team_agents = load_team_agents(project_root, "claude");

    Ok(())
}

fn push_hook(library: &mut ProjectLibrary, id: String, trigger: HookTrigger, command: &str) {
    if library.hooks.iter().any(|h| h.id == id) {
        return;
    }
    library.hooks.push(HookConfig {
        id,
        trigger,
        command: command.to_string(),
        matcher: None,
        cursor_event: None,
    });
}

// ── Team agents ──────────────────────────────────────────────────────────────

fn load_team_agents(project_root: &Path, harness_id: &str) -> Vec<(String, String)> {
    let teams_dir = ship_substrate::ShipHome::project_ship_dir(project_root)
        .join("agents")
        .join("teams")
        .join(harness_id);
    if !teams_dir.exists() {
        return vec![];
    }
    let mut agents = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&teams_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md")
                && let (Some(name), Ok(content)) = (
                    path.file_name().map(|n| n.to_string_lossy().to_string()),
                    std::fs::read_to_string(&path),
                )
            {
                agents.push((name, content));
            }
        }
    }
    agents.sort_by(|a, b| a.0.cmp(&b.0));
    agents
}

// ── Dep skill ref collection ─────────────────────────────────────────────────

/// Collect all skill refs declared in agent profiles and mode configs within
/// `library`. Only dep refs (those starting with `github.com/`) will be resolved
/// by the caller; local refs are included in the list but filtered in
/// [`resolve_dep_skills`](crate::registry::dep_skills::resolve_dep_skills).
pub fn collect_all_skill_refs(library: &ProjectLibrary) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();

    for profile in &library.agent_profiles {
        for r in &profile.skills.refs {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    for agent in &library.agents {
        for r in &agent.skills {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    refs
}
