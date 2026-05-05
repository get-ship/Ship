use super::project::AgentProfile;
use super::types::{HookConfig, PermissionConfig};
use super::runtime_settings::project_ns;
use ship_substrate::db::kv;
use anyhow::Result;
use std::path::Path;

const KEY: &str = "agents";

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
struct AgentRow {
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

pub(super) fn get_agents_config(_ship_dir: &Path) -> Result<Vec<AgentProfile>> {
    let rows: Vec<AgentRow> = match kv::get(project_ns(_ship_dir), KEY)? {
        Some(v) => serde_json::from_value(v).unwrap_or_default(),
        None => return Ok(Vec::new()),
    };

    let mut agents = Vec::new();
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

        agents.push(AgentProfile {
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
    agents.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(agents)
}

pub(super) fn save_agents_config(ship_dir: &Path, agents: &[AgentProfile]) -> Result<()> {
    let mut rows = Vec::with_capacity(agents.len());
    for agent in agents {
        let normalized_rules: Vec<String> = agent
            .rules
            .iter()
            .map(|id| normalize_rule_external_id(id))
            .collect();
        rows.push(AgentRow {
            id: agent.id.clone(),
            name: agent.name.clone(),
            description: agent.description.clone(),
            role: agent.role.clone(),
            tools_preset: agent.tools_preset.clone(),
            active_tools_json: serde_json::to_string(&agent.active_tools)?,
            mcp_refs_json: serde_json::to_string(&agent.mcp_servers)?,
            skill_refs_json: serde_json::to_string(&agent.skills)?,
            rule_refs_json: serde_json::to_string(&normalized_rules)?,
            prompt_id: agent.prompt_id.clone(),
            hooks_json: serde_json::to_string(&agent.hooks)?,
            permissions_json: serde_json::to_string(&agent.permissions)?,
            target_agents_json: serde_json::to_string(&agent.target_agents)?,
        });
    }

    kv::set(project_ns(ship_dir), KEY, &serde_json::to_value(&rows)?)?;
    Ok(())
}
