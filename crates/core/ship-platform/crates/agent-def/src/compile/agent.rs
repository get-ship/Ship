//! Agent profile application — merging an active agent's overrides into the
//! project library. The library has already been populated with every
//! `.ship/agents/*.jsonc` by [`super::loader::load_library`]; this step
//! pulls the active one out and merges its permissions / hooks / plugins /
//! harness settings, then registers the active profile in `library.agents` so
//! `resolve()` can filter MCP / skill / rule refs against it.

use agent_schema::{PluginEntry, PluginsManifest};
use anyhow::Result;
use stack_model::ExtensionLibrary;
use std::path::Path;

use super::permissions::apply_agent_permissions;

// ── Agent → library ──────────────────────────────────────────────────────────

pub fn apply_agent_to_library(
    library: &mut ExtensionLibrary,
    agent_id: &str,
    space_root: &Path,
) -> Result<()> {
    let Some(profile) = library
        .agent_profiles
        .iter()
        .find(|p| p.agent.id == agent_id)
        .cloned()
    else {
        return Ok(());
    };

    // Slice 3 close-out: the permission preset layer is now an
    // in-memory lookup against `library.permissions_presets`
    // (populated by the resolver from
    // `defineConfig.agents.permissions.presets` — the
    // `ship_dir`-keyed disk read died with `.ship/permissions.jsonc`).
    // `space_root` stays in the signature because the sibling
    // team-agent load below still walks a sub-dir of it
    // (claude/teams/); we don't need it for the permission
    // resolution.
    library.permissions = apply_agent_permissions(
        library.permissions.clone(),
        &profile,
        &library.permissions_presets,
    );

    if let Some(inline) = &profile.rules.inline {
        let trimmed = inline.trim();
        if !trimmed.is_empty() {
            // The agent's inline rule body is already in `library.rules`
            // under `file_name = {agent_id}.md` — the resolver's TS eval
            // harvested it from the `defineAgent` call's `rules: { inline }`
            // block and `merge_ts_declarations` merged it in. Only push
            // when the harvest didn't (the path where someone constructs
            // an `ExtensionLibrary` outside the resolver — e.g. an
            // in-process test stub).
            let file_name = format!("{agent_id}.md");
            if !library.rules.iter().any(|r| r.file_name == file_name) {
                library.rules.push(agent_schema::Rule {
                    file_name,
                    content: trimmed.to_string(),
                    always_apply: true,
                    globs: vec![],
                    description: None,
                });
            }
        }
    }

    // Register the active profile in `library.agents` so resolve() can apply
    // its MCP / skill / rule filtering.
    let mut active = profile.clone();
    active.agent.id = agent_id.to_string();
    library.agents.push(active);

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
            scope: profile
                .plugins
                .scope
                .clone()
                .unwrap_or_else(|| "project".to_string()),
        };
    }

    // Merge the profile's lifecycle hooks into the library (dedup by id) so the
    // active agent's Stop / SubagentStop / etc. hooks reach the harness emitters.
    for hook in &profile.hooks {
        if library.hooks.iter().any(|h| h.id == hook.id) {
            continue;
        }
        library.hooks.push(hook.clone());
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

    library.claude_team_agents = load_team_agents(space_root, "claude");

    Ok(())
}

// ── Team agents ──────────────────────────────────────────────────────────────

fn load_team_agents(space_root: &Path, harness_id: &str) -> Vec<(String, String)> {
    // agents/ is a DOMAIN DIR at the stack root (marker cutover).
    let teams_dir = space_root.join("agents").join("teams").join(harness_id);
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
/// [`resolve_dep_skills`](ship_store::dep_skills::resolve_dep_skills).
pub fn collect_all_skill_refs(library: &ExtensionLibrary) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();

    for profile in &library.agent_profiles {
        for r in &profile.skills.refs {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    for agent in &library.agents {
        for r in &agent.skills.refs {
            if !refs.contains(r) {
                refs.push(r.clone());
            }
        }
    }

    refs
}
