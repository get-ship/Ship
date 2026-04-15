//! Load a [`ProjectLibrary`] from the `.ship/agents/` directory tree.
//! No compilation or resolution occurs here — pure filesystem loading.

use anyhow::Result;
use compiler::{
    AgentProfile, HookConfig, HookTrigger, McpServerConfig, McpServerType, Permissions,
    ProjectLibrary, Rule,
};
use serde::Deserialize;
use std::path::Path;

// ── Top-level entry point ─────────────────────────────────────────────────────

/// Load a [`ProjectLibrary`] from a `.ship/` directory (flat layout).
///
/// Looks for mcp, permissions, hooks, rules, skills at the `.ship/` root,
/// and agent profiles under `.ship/agents/profiles/`.
/// Missing files and dirs are silently skipped — an empty library is valid.
pub fn load_library(ship_dir: &Path) -> Result<ProjectLibrary> {
    Ok(ProjectLibrary {
        mcp_servers: load_mcp_servers(ship_dir)?,
        permissions: load_permissions(ship_dir)?,
        hooks: load_hooks(ship_dir)?,
        rules: load_rules(ship_dir)?,
        skills: load_skills(ship_dir)?,
        agent_profiles: load_agent_profiles(ship_dir)?,
        ..Default::default()
    })
}

// ── MCP servers ───────────────────────────────────────────────────────────────

fn load_mcp_servers(agents_dir: &Path) -> Result<Vec<McpServerConfig>> {
    // Prefer mcp.jsonc over mcp.toml
    let jsonc_path = agents_dir.join("mcp.jsonc");
    let path = if jsonc_path.exists() {
        jsonc_path
    } else {
        agents_dir.join("mcp.toml")
    };
    let file = crate::mcp::McpFile::load(&path)?;
    // Warn if the MCP file exists but produced no servers (likely a parse error)
    if path.exists() && file.servers.is_empty() {
        eprintln!(
            "warning: {} exists but no MCP servers were loaded. Check JSON/TOML syntax.",
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
                gemini_trust: None,
                gemini_include_tools: vec![],
                gemini_exclude_tools: vec![],
                gemini_timeout_ms: None,
                cursor_env_file: None,
            }
        })
        .collect())
}

// ── Permissions ───────────────────────────────────────────────────────────────

/// A named permission preset section from `agents/permissions.toml`.
/// Matches the `[ship-standard]`, `[ship-autonomous]`, etc. blocks.
#[derive(Deserialize, Default, Clone)]
pub struct PermissionPreset {
    #[serde(default)]
    pub default_mode: Option<String>,
    #[serde(default)]
    pub tools_allow: Vec<String>,
    #[serde(default)]
    pub tools_deny: Vec<String>,
    #[serde(default)]
    pub tools_ask: Vec<String>,
}

fn load_permissions(agents_dir: &Path) -> Result<Permissions> {
    // Prefer permissions.jsonc over permissions.toml
    let jsonc_path = agents_dir.join("permissions.jsonc");
    let path = if jsonc_path.exists() {
        jsonc_path
    } else {
        agents_dir.join("permissions.toml")
    };
    if !path.exists() {
        return Ok(Permissions::default());
    }
    let s = std::fs::read_to_string(&path)?;
    // Try flat Permissions first, fall back to default on parse error.
    // The named-preset sections are ignored here —
    // they are resolved via load_permission_preset() when a profile activates.
    if crate::paths::is_jsonc_ext(&path) {
        match compiler::jsonc::from_jsonc_str::<Permissions>(&s) {
            Ok(p) => Ok(p),
            Err(_) => Ok(Permissions::default()),
        }
    } else {
        match toml::from_str::<Permissions>(&s) {
            Ok(p) => Ok(p),
            Err(_) => Ok(Permissions::default()),
        }
    }
}

/// Load a named permission preset section (e.g. `[ship-standard]` / `"ship-standard"`)
/// from `agents/permissions.{jsonc,toml}`. Returns `None` if the file or section is absent.
pub fn load_permission_preset(agents_dir: &Path, preset_name: &str) -> Option<PermissionPreset> {
    // Prefer permissions.jsonc over permissions.toml
    let jsonc_path = agents_dir.join("permissions.jsonc");
    let path = if jsonc_path.exists() {
        jsonc_path
    } else {
        agents_dir.join("permissions.toml")
    };
    if !path.exists() {
        return None;
    }
    let s = std::fs::read_to_string(&path).ok()?;

    let val: serde_json::Value = if crate::paths::is_jsonc_ext(&path) {
        compiler::jsonc::from_jsonc_str(&s).ok()?
    } else {
        // Parse TOML then convert to serde_json::Value
        let tv: toml::Value = toml::from_str(&s).ok()?;
        serde_json::to_value(&tv).ok()?
    };

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

fn load_hooks(agents_dir: &Path) -> Result<Vec<HookConfig>> {
    // Prefer hooks.jsonc over hooks.toml
    let jsonc_path = agents_dir.join("hooks.jsonc");
    let path = if jsonc_path.exists() {
        jsonc_path
    } else {
        agents_dir.join("hooks.toml")
    };
    if !path.exists() {
        return Ok(vec![]);
    }
    let s = std::fs::read_to_string(&path)?;
    let file: HooksFile = if crate::paths::is_jsonc_ext(&path) {
        compiler::jsonc::from_jsonc_str(&s)?
    } else {
        toml::from_str(&s)?
    };
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
                gemini_event: None,
            })
        })
        .collect())
}

// ── Rules ─────────────────────────────────────────────────────────────────────

fn load_rules(agents_dir: &Path) -> Result<Vec<Rule>> {
    let rules_dir = agents_dir.join("rules");
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
fn parse_rule(file_name: &str, raw: &str) -> Rule {
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

// ── Skills ────────────────────────────────────────────────────────────────────

use crate::skill_loader::load_skills;

// ── Agent profiles ────────────────────────────────────────────────────────────

fn load_agent_profiles(ship_dir: &Path) -> Result<Vec<AgentProfile>> {
    // Profiles live under agents/profiles/ in the .ship/ directory
    let profiles_dir = ship_dir.join("agents").join("profiles");
    if !profiles_dir.exists() {
        return Ok(vec![]);
    }
    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(&profiles_dir)?.flatten() {
        let path = entry.path();
        if crate::paths::is_config_ext(&path) {
            let content = std::fs::read_to_string(&path)?;
            let result: Result<AgentProfile, String> = if crate::paths::is_jsonc_ext(&path) {
                compiler::jsonc::from_jsonc_str(&content).map_err(|e| e.to_string())
            } else {
                toml::from_str(&content).map_err(|e| e.to_string())
            };
            match result {
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
    }
    profiles.sort_by(|a, b| a.profile.id.cmp(&b.profile.id));
    Ok(profiles)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "loader_tests.rs"]
mod tests;
