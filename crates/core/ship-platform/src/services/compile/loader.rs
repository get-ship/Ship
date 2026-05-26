//! Load a [`ProjectLibrary`] from the `.ship/` directory tree.
//! No compilation or resolution occurs here — pure filesystem loading.

use anyhow::Result;
use compiler::{
    AgentProfile, HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions,
    ProjectLibrary, Rule,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::permissions::PermissionPreset;
use super::skill_parser::load_compile_skills;

// ── MCP file types ────────────────────────────────────────────────────────────

/// A single MCP server entry from `mcp.jsonc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEntry {
    pub id: String,
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_type: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
}

fn default_scope() -> String {
    "project".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawMcpFile {
    #[serde(default)]
    mcp: RawMcpSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RawMcpSection {
    #[serde(default)]
    servers: HashMap<String, McpEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct LegacyMcpFile {
    #[serde(default)]
    servers: Vec<McpEntry>,
}

/// The `.ship/mcp.jsonc` file. Supports both the canonical keyed format
/// (`{ "mcp": { "servers": { "<id>": {...} } } }`) and the legacy flat array
/// format (`{ "servers": [...] }`).
#[derive(Debug, Clone, Default)]
pub struct McpFile {
    pub servers: Vec<McpEntry>,
}

impl McpFile {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)?;

        if let Ok(raw) = compiler::jsonc::from_jsonc_str::<RawMcpFile>(&text)
            && !raw.mcp.servers.is_empty()
        {
            let servers = raw
                .mcp
                .servers
                .into_iter()
                .map(|(key, mut entry)| {
                    if entry.id.is_empty() {
                        entry.id = key;
                    }
                    entry
                })
                .collect();
            return Ok(Self { servers });
        }
        if let Ok(legacy) = compiler::jsonc::from_jsonc_str::<LegacyMcpFile>(&text) {
            return Ok(Self {
                servers: legacy.servers,
            });
        }
        Ok(Self::default())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let mut servers_map = serde_json::Map::new();
        for s in &self.servers {
            let val = serde_json::to_value(s)?;
            servers_map.insert(s.id.clone(), val);
        }
        let root = serde_json::json!({ "mcp": { "servers": servers_map } });
        std::fs::write(path, serde_json::to_string_pretty(&root)?)?;
        Ok(())
    }
}

// ── Top-level entry point ─────────────────────────────────────────────────────

/// Load a [`ProjectLibrary`] from a `.ship/` directory (flat layout).
///
/// Looks for mcp, permissions, hooks, rules, skills at the `.ship/` root,
/// and agent profiles under `.ship/agents/`.
/// Missing files and dirs are silently skipped — an empty library is valid.
pub fn load_library(ship_dir: &Path) -> Result<ProjectLibrary> {
    Ok(ProjectLibrary {
        mcp_servers: load_mcp_servers(ship_dir)?,
        permissions: load_permissions(ship_dir)?,
        hooks: load_hooks(ship_dir)?,
        rules: load_rules(ship_dir)?,
        skills: load_compile_skills(ship_dir)?,
        agent_profiles: load_agent_profiles(ship_dir)?,
        ..Default::default()
    })
}

// ── MCP servers ───────────────────────────────────────────────────────────────

fn load_mcp_servers(ship_dir: &Path) -> Result<Vec<McpServerConfig>> {
    let path = ship_dir.join("mcp.jsonc");
    let file = McpFile::load(&path)?;
    if path.exists() && file.servers.is_empty() {
        eprintln!(
            "warning: {} exists but no MCP servers were loaded. Check JSONC syntax.",
            path.display()
        );
    }
    Ok(file
        .servers
        .into_iter()
        .map(|e| {
            let server_type = match e.server_type.as_deref() {
                Some("http") => McpServerType::Http,
                Some("sse") => McpServerType::Sse,
                _ => {
                    if e.url.is_some() && e.command.is_none() {
                        McpServerType::Http
                    } else {
                        McpServerType::Stdio
                    }
                }
            };
            McpServerConfig {
                id: e.id.clone(),
                name: e.name.unwrap_or_else(|| e.id.clone()),
                command: e.command.unwrap_or_default(),
                args: e.args,
                env: e.env,
                scope: e.scope,
                server_type,
                url: e.url,
                disabled: e.disabled,
                timeout_secs: None,
                codex_enabled_tools: vec![],
                codex_disabled_tools: vec![],
                cursor_env_file: None,
            }
        })
        .collect())
}

// ── Permissions ───────────────────────────────────────────────────────────────

fn load_permissions(ship_dir: &Path) -> Result<Permissions> {
    let path = ship_dir.join("permissions.jsonc");
    if !path.exists() {
        return Ok(Permissions::default());
    }
    let s = std::fs::read_to_string(&path)?;
    Ok(compiler::jsonc::from_jsonc_str::<Permissions>(&s).unwrap_or_default())
}

/// Load a named permission preset section (e.g. `"ship-standard"`) from
/// `<ship_dir>/permissions.jsonc`. Returns `None` if the file or section
/// is absent.
pub fn load_permission_preset(ship_dir: &Path, preset_name: &str) -> Option<PermissionPreset> {
    let path = ship_dir.join("permissions.jsonc");
    if !path.exists() {
        return None;
    }
    let s = std::fs::read_to_string(&path).ok()?;
    let val: serde_json::Value = compiler::jsonc::from_jsonc_str(&s).ok()?;

    let section = val.get(preset_name)?.as_object()?;

    let get_str_list = |key: &str| -> Vec<String> {
        section
            .get(key)
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let default_mode = section
        .get("default_mode")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    Some(PermissionPreset {
        default_mode,
        tools_allow: get_str_list("tools_allow"),
        tools_deny: get_str_list("tools_deny"),
        tools_ask: get_str_list("tools_ask"),
    })
}

// ── Hooks ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct HooksFile {
    #[serde(default)]
    hooks: Vec<HookEntry>,
}

#[derive(Deserialize)]
struct HookEntry {
    id: String,
    trigger: String,
    command: String,
    #[serde(default)]
    matcher: Option<String>,
}

fn load_hooks(ship_dir: &Path) -> Result<Vec<HookConfig>> {
    let path = ship_dir.join("hooks.jsonc");
    if !path.exists() {
        return Ok(vec![]);
    }
    let s = std::fs::read_to_string(&path)?;
    let file: HooksFile = compiler::jsonc::from_jsonc_str(&s)?;
    Ok(file
        .hooks
        .into_iter()
        .filter_map(|e| {
            let trigger = match e.trigger.as_str() {
                "pre_tool_use" | "PreToolUse" => HookTrigger::PreToolUse,
                "post_tool_use" | "PostToolUse" => HookTrigger::PostToolUse,
                "notification" | "Notification" => HookTrigger::Notification,
                "stop" | "Stop" => HookTrigger::Stop,
                "subagent_stop" | "SubagentStop" => HookTrigger::SubagentStop,
                "pre_compact" | "PreCompact" => HookTrigger::PreCompact,
                _ => return None,
            };
            Some(HookConfig {
                id: e.id,
                trigger,
                command: e.command,
                matcher: e.matcher,
                cursor_event: None,
            })
        })
        .collect())
}

// ── Rules ─────────────────────────────────────────────────────────────────────

fn load_rules(ship_dir: &Path) -> Result<Vec<Rule>> {
    let rules_dir = ship_dir.join("rules");
    if !rules_dir.exists() {
        return Ok(vec![]);
    }
    let mut rules = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&rules_dir)?
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let file_name = e.file_name().to_string_lossy().to_string();
        let raw = std::fs::read_to_string(e.path())?;
        rules.push(parse_rule(&file_name, &raw));
    }
    Ok(rules)
}

/// Parse a rule `.md` file, stripping YAML frontmatter if present.
/// Frontmatter fields: `description`, `globs` (list), `alwaysApply` (bool).
pub fn parse_rule(file_name: &str, raw: &str) -> Rule {
    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let body = &rest[end + 5..];
        let mut always_apply = true;
        let mut globs = vec![];
        let mut description = None;
        for line in fm.lines() {
            if let Some(v) = line.strip_prefix("alwaysApply:") {
                always_apply = v.trim() != "false";
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().trim_matches('"').to_string());
            } else if line.trim_start().starts_with("- ") {
                globs.push(line.trim().trim_start_matches("- ").to_string());
            }
        }
        return Rule {
            file_name: file_name.to_string(),
            content: body.trim().to_string(),
            always_apply,
            globs,
            description,
        };
    }
    Rule {
        file_name: file_name.to_string(),
        content: raw.trim().to_string(),
        always_apply: true,
        globs: vec![],
        description: None,
    }
}

// ── Agent profiles ────────────────────────────────────────────────────────────

fn load_agent_profiles(ship_dir: &Path) -> Result<Vec<AgentProfile>> {
    let agents_dir = ship_dir.join("agents");
    if !agents_dir.exists() {
        return Ok(vec![]);
    }
    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(&agents_dir)?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("jsonc") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        match compiler::jsonc::from_jsonc_str::<AgentProfile>(&content) {
            Ok(profile) => profiles.push(profile),
            Err(e) => {
                eprintln!(
                    "warning: skipping {}: {}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    e,
                );
            }
        }
    }
    profiles.sort_by(|a, b| a.agent.id.cmp(&b.agent.id));
    Ok(profiles)
}
