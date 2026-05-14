use crate::config::{McpServerConfig, get_config};
#[derive(Serialize, Deserialize, Debug, Clone, Default, Type)]
pub struct WorkspaceAgentSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<String>,
}

use crate::permissions::{Permissions, get_permissions};
use crate::rule::{Rule, list_rules};
use crate::skill::{Skill, list_effective_skills};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
use std::path::Path;

// ─── Resolved config ──────────────────────────────────────────────────────────

/// Fully resolved, in-memory provider settings for the current branch/feature.
///
/// Not stored as a file. Computed by [`resolve_provider_settings`] from:
/// 1. Project defaults (`ship.jsonc` + `agents/` directory)
/// 2. Active agent profile overrides
/// 3. Feature `[agent]` block (thin overrides)
///
/// A snapshot of key IDs is stored in the Workspace SQLite record for the UI.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ProviderSettings {
    /// Active provider IDs (e.g. `["claude"]`). Inherited from project or overridden by feature.
    pub providers: Vec<String>,
    /// Model override — `None` means use the provider's default.
    pub model: Option<String>,
    /// Resolved MCP servers (project list, filtered by agent profile + feature).
    pub mcp_servers: Vec<McpServerConfig>,
    /// Resolved skills (project + user effective list, filtered by feature).
    pub skills: Vec<Skill>,
    /// Rules from `agents/rules/` — always active, no filtering.
    pub rules: Vec<Rule>,
    /// Resolved permissions from `agents/permissions.toml`.
    pub permissions: Permissions,
    /// Active agent ID if one is set in `ship.jsonc`.
    pub active_agent: Option<String>,
}

fn normalize_provider_ids(ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for raw in ids {
        let id = raw.trim().to_ascii_lowercase();
        if id.is_empty() {
            continue;
        }
        // Keep provider IDs aligned with registered provider descriptors.
        if !matches!(id.as_str(), "claude" | "gemini" | "codex") {
            continue;
        }
        if seen.insert(id.clone()) {
            normalized.push(id);
        }
    }

    normalized
}

fn normalize_rule_id(id: &str) -> String {
    let normalized = id.trim().trim_end_matches(".md");
    if let Some((prefix, rest)) = normalized.split_once('-')
        && !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_digit())
    {
        return rest.to_string();
    }
    normalized.to_string()
}

// ─── Resolution ───────────────────────────────────────────────────────────────

/// Resolve the effective [`ProviderSettings`] for the current project state.
///
/// Resolution order (highest wins):
/// 1. Project defaults (`ship.jsonc` + `agents/` directory)
/// 2. Active agent profile overrides (profile's `mcp_servers` filter, skill refs, rules)
/// 3. Feature `[agent]` block — thin overrides: `model`, `providers`, filtered server/skill IDs
///
/// Pass `None` for `workspace_agent` when not on a feature branch.
pub fn resolve_provider_settings(
    ship_dir: &Path,
    workspace_agent: Option<&WorkspaceAgentSettings>,
) -> Result<ProviderSettings> {
    resolve_provider_settings_with_agent_override(ship_dir, workspace_agent, None)
}

/// Resolve effective provider settings with an optional agent profile override.
///
/// `active_agent_override` is intended for workspace-level agent selection.
/// When provided and valid, it takes precedence over project `active_agent`.
pub fn resolve_provider_settings_with_agent_override(
    ship_dir: &Path,
    workspace_agent: Option<&WorkspaceAgentSettings>,
    active_agent_override: Option<&str>,
) -> Result<ProviderSettings> {
    let config = get_config(Some(ship_dir.to_path_buf()))?;

    // ── Active agent ──────────────────────────────────────────────────────────
    let active_agent = active_agent_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| config.agents.iter().any(|agent| agent.id == *value))
        .map(str::to_string);
    let agent = config
        .agents
        .iter()
        .find(|a| active_agent.as_deref() == Some(a.id.as_str()));

    // ── Providers ─────────────────────────────────────────────────────────────
    // Resolution priority (highest wins):
    // 1. Workspace explicit providers — if they specified any (even if all unknown),
    //    use the normalized result rather than falling back. An empty result from all-invalid
    //    providers propagates as a no-valid-providers error rather than silently using "claude".
    // 2. Agent profile target_agents — when an agent is active and specifies target agents.
    // 3. Project-level providers from ship.jsonc.
    // 4. Default: ["claude"].
    let workspace_has_explicit_providers =
        workspace_agent.is_some_and(|wa| !wa.providers.is_empty());
    let workspace_override_providers = workspace_agent
        .filter(|wa| !wa.providers.is_empty())
        .map(|wa| normalize_provider_ids(&wa.providers))
        .unwrap_or_default();
    let providers = if workspace_has_explicit_providers {
        workspace_override_providers
    } else {
        let agent_providers = agent
            .filter(|a| !a.target_agents.is_empty())
            .map(|a| normalize_provider_ids(&a.target_agents))
            .unwrap_or_default();
        if !agent_providers.is_empty() {
            agent_providers
        } else {
            let project_providers = normalize_provider_ids(&config.providers);
            if project_providers.is_empty() {
                vec!["claude".to_string()]
            } else {
                project_providers
            }
        }
    };

    // ── Model ─────────────────────────────────────────────────────────────────
    // Model comes from the workspace override only — there's no project-level
    // default. The provider's CLI picks a default when none is specified.
    let model = workspace_agent.and_then(|wa| wa.model.clone());

    // ── MCP servers ───────────────────────────────────────────────────────────
    let mut mcp_servers = config.mcp_servers.clone();

    // Agent profile filter: if profile restricts servers, retain only allowed IDs.
    if let Some(m) = agent
        && !m.mcp_servers.is_empty()
    {
        mcp_servers.retain(|s| m.mcp_servers.contains(&s.id));
    }

    // Workspace filter: if workspace specifies server IDs, retain only those.
    if let Some(wa) = workspace_agent
        && !wa.mcp_servers.is_empty()
    {
        let ids: Vec<&str> = wa.mcp_servers.iter().map(|r| r.as_str()).collect();
        mcp_servers.retain(|s| ids.contains(&s.id.as_str()));
    }

    // ── Skills ────────────────────────────────────────────────────────────────
    let mut skills = list_effective_skills(ship_dir)?;

    // Agent profile filter: if profile restricts skills, retain only allowed IDs.
    if let Some(m) = agent
        && !m.skills.is_empty()
    {
        skills.retain(|s| m.skills.contains(&s.id));
    }

    // Workspace filter: if workspace specifies skill IDs, retain only those.
    if let Some(wa) = workspace_agent
        && !wa.skills.is_empty()
    {
        let ids: Vec<&str> = wa.skills.iter().map(|r| r.as_str()).collect();
        skills.retain(|s| ids.contains(&s.id.as_str()));
    }

    // ── Rules ─────────────────────────────────────────────────────────────────
    let mut rules = list_rules(ship_dir.to_path_buf())?;
    if let Some(m) = agent
        && !m.rules.is_empty()
    {
        let allowed: HashSet<String> = m.rules.iter().map(|id| normalize_rule_id(id)).collect();
        rules.retain(|rule| allowed.contains(&normalize_rule_id(&rule.file_name)));
    }

    // ── Permissions ───────────────────────────────────────────────────────────
    let permissions = get_permissions(ship_dir.to_path_buf())?;

    Ok(ProviderSettings {
        providers,
        model,
        mcp_servers,
        skills,
        rules,
        permissions,
        active_agent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentProfile, McpServerType, ProjectConfig, save_config};
    use crate::testutil::init_project;
    use crate::rule::create_rule;
    use crate::skill::create_skill;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn stdio_server(id: &str, command: &str) -> McpServerConfig {
        McpServerConfig {
            id: id.to_string(),
            name: id.to_string(),
            command: command.to_string(),
            args: vec![],
            env: HashMap::new(),
            scope: "project".to_string(),
            server_type: McpServerType::Stdio,
            url: None,
            disabled: false,
            timeout_secs: None,
        }
    }

    #[test]
    fn resolve_agent_config_feature_overrides_providers_model_and_skill_filter() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        create_skill(&ship_dir, "rt-alpha-skill", "Alpha", "alpha body")?;
        create_skill(&ship_dir, "rt-beta-skill", "Beta", "beta body")?;

        let config = ProjectConfig {
            providers: vec!["claude".to_string(), "gemini".to_string()],
            mcp_servers: vec![stdio_server("github", "github-bin")],
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = WorkspaceAgentSettings {
            model: Some("feature-model".to_string()),
            mcp_servers: vec!["github".to_string()],
            skills: vec!["rt-beta-skill".to_string()],
            providers: vec!["codex".to_string()],
        };

        let resolved = resolve_provider_settings(&ship_dir, Some(&feature_agent))?;
        assert_eq!(resolved.providers, vec!["codex".to_string()]);
        assert_eq!(resolved.model.as_deref(), Some("feature-model"));
        assert_eq!(
            resolved
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["rt-beta-skill"]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_feature_skill_filter_ignores_missing_ids() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        create_skill(&ship_dir, "rt-only-skill", "Only", "only body")?;

        let feature_agent = WorkspaceAgentSettings {
            model: None,
            mcp_servers: vec![],
            skills: vec!["rt-missing-skill".to_string(), "rt-only-skill".to_string()],
            providers: vec![],
        };

        let resolved = resolve_provider_settings(&ship_dir, Some(&feature_agent))?;
        assert_eq!(
            resolved
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["rt-only-skill"]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_applies_mode_mcp_filter_without_feature_override() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        let config = ProjectConfig {
            mcp_servers: vec![
                stdio_server("github", "github-bin"),
                stdio_server("linear", "linear-bin"),
            ],
            agents: vec![AgentProfile {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                mcp_servers: vec!["github".to_string()],
                ..Default::default()
            }],
            // active_agent removed — pass via override
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let resolved =
            resolve_provider_settings_with_agent_override(&ship_dir, None, Some("planning"))?;
        assert_eq!(resolved.mcp_servers.len(), 1);
        assert_eq!(resolved.mcp_servers[0].id, "github");
        Ok(())
    }

    #[test]
    fn resolve_agent_config_merges_mcp_toml_and_overrides_duplicate_ids() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        let config = ProjectConfig {
            mcp_servers: vec![stdio_server("github", "ship-toml-command")],
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let mut servers = HashMap::new();
        servers.insert(
            "github".to_string(),
            McpServerConfig {
                id: String::new(),
                name: "github".to_string(),
                command: "mcp-toml-command".to_string(),
                args: vec![],
                env: HashMap::new(),
                scope: "project".to_string(),
                server_type: McpServerType::Stdio,
                url: None,
                disabled: false,
                timeout_secs: None,
            },
        );
        servers.insert(
            "figma".to_string(),
            McpServerConfig {
                id: String::new(),
                name: "figma".to_string(),
                command: "figma-command".to_string(),
                args: vec![],
                env: HashMap::new(),
                scope: "project".to_string(),
                server_type: McpServerType::Stdio,
                url: None,
                disabled: false,
                timeout_secs: None,
            },
        );
        let mcp_json = serde_json::to_string_pretty(&crate::config::McpConfig {
            mcp: crate::config::McpSection { servers },
        })?;
        std::fs::write(ship_substrate::project::mcp_config_path(&ship_dir), mcp_json)?;

        let resolved = resolve_provider_settings(&ship_dir, None)?;
        let github = resolved
            .mcp_servers
            .iter()
            .find(|server| server.id == "github")
            .expect("github server should exist");
        let figma = resolved
            .mcp_servers
            .iter()
            .find(|server| server.id == "figma")
            .expect("figma server should exist");

        assert_eq!(github.command, "mcp-toml-command");
        assert_eq!(figma.command, "figma-command");
        Ok(())
    }

    #[test]
    fn resolve_agent_config_mode_and_feature_mcp_filters_intersect() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        let config = ProjectConfig {
            mcp_servers: vec![
                stdio_server("github", "github-bin"),
                stdio_server("linear", "linear-bin"),
                stdio_server("figma", "figma-bin"),
            ],
            agents: vec![AgentProfile {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                mcp_servers: vec!["github".to_string(), "linear".to_string()],
                ..Default::default()
            }],
            // active_agent removed — pass via override
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = WorkspaceAgentSettings {
            model: None,
            mcp_servers: vec!["linear".to_string(), "figma".to_string()],
            skills: vec![],
            providers: vec![],
        };

        let resolved = resolve_provider_settings_with_agent_override(
            &ship_dir,
            Some(&feature_agent),
            Some("planning"),
        )?;
        let ids: Vec<&str> = resolved
            .mcp_servers
            .iter()
            .map(|server| server.id.as_str())
            .collect();
        assert_eq!(ids, vec!["linear"]);
        Ok(())
    }

    #[test]
    fn resolve_agent_config_feature_without_provider_override_uses_project_providers() -> Result<()>
    {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        // Providers are file-canonical in ship.jsonc — save_config doesn't
        // persist them (project identity only), so write them directly to
        // exercise the project-providers fallback path.
        std::fs::write(
            ship_dir.join("ship.jsonc"),
            "{\"providers\": [\"claude\", \"gemini\"]}\n",
        )?;

        let feature_agent = WorkspaceAgentSettings {
            model: Some("feature-model".to_string()),
            mcp_servers: vec![],
            skills: vec![],
            providers: vec![],
        };

        let resolved = resolve_provider_settings(&ship_dir, Some(&feature_agent))?;
        assert_eq!(
            resolved.providers,
            vec!["claude".to_string(), "gemini".to_string()]
        );
        assert_eq!(resolved.model.as_deref(), Some("feature-model"));
        Ok(())
    }

    #[test]
    fn resolve_agent_config_feature_providers_are_normalized_and_unknown_filtered() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        let config = ProjectConfig {
            providers: vec!["gemini".to_string()],
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = WorkspaceAgentSettings {
            model: None,
            mcp_servers: vec![],
            skills: vec![],
            providers: vec![
                " codex ".to_string(),
                "unknown-provider".to_string(),
                "CLAUDE".to_string(),
                "claude".to_string(),
            ],
        };

        let resolved = resolve_provider_settings(&ship_dir, Some(&feature_agent))?;
        assert_eq!(
            resolved.providers,
            vec!["codex".to_string(), "claude".to_string()]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_invalid_feature_provider_returns_empty() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        let config = ProjectConfig {
            providers: vec!["gemini".to_string(), "claude".to_string()],
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let feature_agent = WorkspaceAgentSettings {
            model: None,
            mcp_servers: vec![],
            skills: vec![],
            providers: vec!["unknown-provider".to_string()],
        };

        // Invalid feature-level providers are no longer silently ignored —
        // an empty provider list propagates so callers can surface the error.
        let resolved = resolve_provider_settings(&ship_dir, Some(&feature_agent))?;
        assert!(resolved.providers.is_empty());
        Ok(())
    }

    #[test]
    fn resolve_agent_config_invalid_project_providers_fall_back_to_claude() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        let config = ProjectConfig {
            providers: vec!["unknown-provider".to_string(), "   ".to_string()],
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let resolved = resolve_provider_settings(&ship_dir, None)?;
        assert_eq!(resolved.providers, vec!["claude".to_string()]);
        Ok(())
    }

    #[test]
    fn resolve_agent_config_mode_filters_skills_and_rules() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        create_skill(&ship_dir, "rt-alpha-skill", "Alpha", "alpha body")?;
        create_skill(&ship_dir, "rt-beta-skill", "Beta", "beta body")?;
        create_rule(
            ship_dir.clone(),
            "010-runtime-hardening.md",
            "# Runtime Hardening\n",
        )?;

        let config = ProjectConfig {
            // active_agent removed — pass via override
            agents: vec![AgentProfile {
                id: "planning".to_string(),
                name: "Planning".to_string(),
                skills: vec!["rt-alpha-skill".to_string()],
                rules: vec!["010-runtime-hardening".to_string()],
                ..Default::default()
            }],
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        let resolved =
            resolve_provider_settings_with_agent_override(&ship_dir, None, Some("planning"))?;
        assert_eq!(
            resolved
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["rt-alpha-skill"]
        );
        assert_eq!(
            resolved
                .rules
                .iter()
                .map(|rule| rule.file_name.as_str())
                .collect::<Vec<_>>(),
            vec!["010-runtime-hardening.md"]
        );
        Ok(())
    }

    #[test]
    fn resolve_agent_config_workspace_mode_override_takes_precedence() -> Result<()> {
        let tmp = tempdir()?;
        let ship_dir = init_project(tmp.path())?;

        create_skill(&ship_dir, "rt-plan-skill", "Plan Skill", "plan body")?;
        create_skill(&ship_dir, "rt-code-skill", "Code Skill", "code body")?;

        let config = ProjectConfig {
            // active_agent removed — pass via override
            agents: vec![
                AgentProfile {
                    id: "planning".to_string(),
                    name: "Planning".to_string(),
                    skills: vec!["rt-plan-skill".to_string()],
                    ..Default::default()
                },
                AgentProfile {
                    id: "code".to_string(),
                    name: "Code".to_string(),
                    skills: vec!["rt-code-skill".to_string()],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        save_config(&config, Some(ship_dir.clone()))?;

        // Baseline (no project active_agent — the field no longer exists in
        // ProjectConfig — and no override): no agent is selected, no skills
        // get filtered in.
        let baseline = resolve_provider_settings(&ship_dir, None)?;
        assert_eq!(baseline.active_agent, None);

        // Override picks the right agent and its skills.
        let overridden =
            resolve_provider_settings_with_agent_override(&ship_dir, None, Some("code"))?;
        assert_eq!(overridden.active_agent.as_deref(), Some("code"));
        assert_eq!(
            overridden
                .skills
                .iter()
                .map(|skill| skill.id.as_str())
                .collect::<Vec<_>>(),
            vec!["rt-code-skill"]
        );

        // Invalid override falls back to None (no project default exists).
        let invalid_override =
            resolve_provider_settings_with_agent_override(&ship_dir, None, Some("missing-agent"))?;
        assert_eq!(invalid_override.active_agent, None);

        Ok(())
    }
}
