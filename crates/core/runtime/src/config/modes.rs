use super::project::AgentProfile;
use super::types::{HookConfig, PermissionConfig};
use super::runtime_settings::project_ns;
use crate::db::kv;
use anyhow::Result;
use std::path::Path;

const KEY: &str = "modes";

/// Strip the optional `NNNN-` numeric prefix and `.md` suffix from a rule
/// reference so callers see canonical IDs (`rust` rather than `0001-rust.md`).
fn normalize_rule_external_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

/// Intermediate row stored in kv_state — mirrors the old agent_config columns.
#[derive(serde::Serialize, serde::Deserialize)]
struct ModeRow {
    id: String,
    name: String,
    description: Option<String>,
    role: Option<String>,
    #[serde(default)]
    tools_preset: Option<String>,
    active_tools_json: String,
    mcp_refs_json: String,
    skill_refs_json: String,
    rule_refs_json: String,
    prompt_id: Option<String>,
    hooks_json: String,
    permissions_json: String,
    target_agents_json: String,
}

pub(super) fn get_modes_config(_ship_dir: &Path) -> Result<Vec<AgentProfile>> {
    let ns = project_ns(_ship_dir);
    let rows: Vec<ModeRow> = match kv::get(&ns, KEY)? {
        Some(v) => serde_json::from_value(v).unwrap_or_default(),
        None => return Ok(Vec::new()),
    };

    let mut modes = Vec::new();
    for row in rows {
        let active_tools: Vec<String> =
            serde_json::from_str(&row.active_tools_json).unwrap_or_default();
        let mcp_servers: Vec<String> =
            serde_json::from_str(&row.mcp_refs_json).unwrap_or_default();
        let skills: Vec<String> =
            serde_json::from_str(&row.skill_refs_json).unwrap_or_default();
        let rules: Vec<String> = serde_json::from_str(&row.rule_refs_json).unwrap_or_default();
        let hooks: Vec<HookConfig> = serde_json::from_str(&row.hooks_json).unwrap_or_default();
        let permissions: PermissionConfig =
            serde_json::from_str(&row.permissions_json).unwrap_or_default();
        let target_agents: Vec<String> =
            serde_json::from_str(&row.target_agents_json).unwrap_or_default();

        modes.push(AgentProfile {
            id: row.id,
            name: row.name,
            description: row.description,
            role: row.role,
            tools_preset: row.tools_preset,
            active_tools,
            mcp_servers,
            skills,
            rules,
            prompt_id: row.prompt_id,
            hooks,
            permissions,
            target_agents,
        });
    }
    modes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(modes)
}

pub(super) fn save_modes_config(ship_dir: &Path, modes: &[AgentProfile]) -> Result<()> {
    let mut rows = Vec::with_capacity(modes.len());
    for mode in modes {
        let normalized_rules: Vec<String> = mode
            .rules
            .iter()
            .map(|id| normalize_rule_external_id(id))
            .collect();
        rows.push(ModeRow {
            id: mode.id.clone(),
            name: mode.name.clone(),
            description: mode.description.clone(),
            role: mode.role.clone(),
            tools_preset: mode.tools_preset.clone(),
            active_tools_json: serde_json::to_string(&mode.active_tools)?,
            mcp_refs_json: serde_json::to_string(&mode.mcp_servers)?,
            skill_refs_json: serde_json::to_string(&mode.skills)?,
            rule_refs_json: serde_json::to_string(&normalized_rules)?,
            prompt_id: mode.prompt_id.clone(),
            hooks_json: serde_json::to_string(&mode.hooks)?,
            permissions_json: serde_json::to_string(&mode.permissions)?,
            target_agents_json: serde_json::to_string(&mode.target_agents)?,
        });
    }

    let ns = project_ns(ship_dir);
    kv::set(&ns, KEY, &serde_json::to_value(&rows)?)?;
    Ok(())
}
