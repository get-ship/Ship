use crate::config::{McpServerConfig, McpServerType, get_config};
use crate::permissions::{
    AgentLimits, CommandPermissions, NetworkPolicy, Permissions, ToolPermissions, save_permissions,
};
use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::providers::{ConfigFormat, ProviderDescriptor, SkillsOutput, require_provider};

// ─── MCP server import ────────────────────────────────────────────────────────

pub fn import_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<usize> {
    let desc = require_provider(provider_id)?;
    let (managed, _) =
        ship_substrate::db::managed_state::get_managed_state_db(provider_id).unwrap_or_default();

    let mut config = get_config(Some(project_dir.clone()))?;
    let mut added = 0usize;
    let import_paths = provider_import_paths(desc, &project_dir)?;

    for path in import_paths {
        if !path.exists() {
            continue;
        }
        let imported_scope = import_scope_for_path(desc, &project_dir, &path);
        let imported_servers = match desc.config_format {
            ConfigFormat::Json => import_mcp_servers_from_json(desc, &path)?,
            ConfigFormat::Toml => import_mcp_servers_from_toml(desc, &path)?,
        };

        for server in imported_servers {
            let Some(mut server) = normalize_imported_server(server) else {
                continue;
            };
            server.scope = imported_scope.clone();
            if managed.contains(&server.id) {
                continue;
            }
            if config
                .mcp_servers
                .iter()
                .any(|existing| existing.id == server.id)
            {
                continue;
            }
            config.mcp_servers.push(server);
            added += 1;
        }
    }

    if added > 0 {
        crate::config::save_config(&config, Some(project_dir))?;
    }
    Ok(added)
}

fn import_scope_for_path(desc: &ProviderDescriptor, project_dir: &Path, path: &Path) -> String {
    let project_scope_path = project_dir
        .parent()
        .map(|root| root.join(desc.project_config));
    if project_scope_path.as_ref().is_some_and(|p| p == path) {
        "project".to_string()
    } else {
        "global".to_string()
    }
}

fn normalize_imported_server(mut server: McpServerConfig) -> Option<McpServerConfig> {
    server.id = server.id.trim().to_string();
    if server.id.is_empty() {
        return None;
    }

    if server.id.eq_ignore_ascii_case("ship") {
        return None;
    }

    if server.name.trim().is_empty() {
        server.name = server.id.clone();
    }

    match server.server_type {
        McpServerType::Stdio => {
            server.command = server.command.trim().to_string();
            if server.command.is_empty() {
                return None;
            }
            server.url = None;
        }
        McpServerType::Sse | McpServerType::Http => {
            let url = server
                .url
                .as_deref()
                .map(str::trim)
                .filter(|u| !u.is_empty())?
                .to_string();
            server.url = Some(url);
            server.command.clear();
            server.args.clear();
        }
    }

    Some(server)
}

pub fn import_permissions_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<bool> {
    let imported = match provider_id {
        "claude" => import_permissions_from_claude(&project_dir)?,
        "gemini" => import_permissions_from_gemini(&project_dir)?,
        "codex" => import_permissions_from_codex(&project_dir)?,
        _ => return Err(anyhow!("Unsupported provider '{}'", provider_id)),
    };

    if let Some(permissions) = imported {
        save_permissions(project_dir, &permissions)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn provider_import_paths(desc: &ProviderDescriptor, project_dir: &Path) -> Result<Vec<PathBuf>> {
    let project_path_opt = project_dir
        .parent()
        .map(|project_root| project_root.join(desc.project_config));
    let global_path = home()?.join(desc.global_config);
    if let Some(project_path) = project_path_opt.as_ref()
        && project_path.exists()
    {
        return Ok(vec![project_path.clone()]);
    }

    if global_path.exists() {
        return Ok(vec![global_path.clone()]);
    }

    let mut candidates = Vec::new();
    if let Some(project_path) = project_path_opt {
        candidates.push(project_path);
    }
    if !candidates.iter().any(|path| path == &global_path) {
        candidates.push(global_path);
    }
    Ok(candidates)
}

fn import_mcp_servers_from_json(
    desc: &ProviderDescriptor,
    path: &Path,
) -> Result<Vec<McpServerConfig>> {
    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(path)?)?;
    let Some(mcp_obj) = root.get(desc.mcp_key).and_then(|v| v.as_object()) else {
        return Ok(Vec::new());
    };

    let mut servers = Vec::new();
    for (id, entry) in mcp_obj {
        let command = entry
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = entry
            .get(desc.http_url_field)
            .or_else(|| entry.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let server_type = match entry.get("type").and_then(|v| v.as_str()) {
            Some("sse") => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _ => {
                if command.is_empty() && url.is_some() {
                    McpServerType::Http
                } else {
                    McpServerType::Stdio
                }
            }
        };

        servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command,
            args: entry
                .get("args")
                .and_then(|v| v.as_array())
                .map(|args| {
                    args.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            env: entry
                .get("env")
                .and_then(|v| v.as_object())
                .map(|env| {
                    env.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: entry
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or_default(),
            timeout_secs: entry
                .get("startup_timeout_sec")
                .or_else(|| entry.get("timeout_secs"))
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok()),
        });
    }
    Ok(servers)
}

fn import_mcp_servers_from_toml(
    desc: &ProviderDescriptor,
    path: &Path,
) -> Result<Vec<McpServerConfig>> {
    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let Some(mcp_table) = root.get(desc.mcp_key).and_then(|v| v.as_table()) else {
        return Ok(Vec::new());
    };

    let mut servers = Vec::new();
    for (id, entry) in mcp_table {
        let Some(table) = entry.as_table() else {
            continue;
        };

        let command = table
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let url = table
            .get(desc.http_url_field)
            .or_else(|| table.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let server_type = match table.get("type").and_then(|v| v.as_str()) {
            Some("sse") => McpServerType::Sse,
            Some("http") => McpServerType::Http,
            _ => {
                if command.is_empty() && url.is_some() {
                    McpServerType::Http
                } else {
                    McpServerType::Stdio
                }
            }
        };

        servers.push(McpServerConfig {
            id: id.clone(),
            name: id.clone(),
            command,
            args: table
                .get("args")
                .and_then(|v| v.as_array())
                .map(|args| {
                    args.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            env: table
                .get("env")
                .and_then(|v| v.as_table())
                .map(|env| {
                    env.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
            scope: "global".to_string(),
            server_type,
            url,
            disabled: table
                .get("disabled")
                .and_then(|v| v.as_bool())
                .unwrap_or_default(),
            timeout_secs: table
                .get("startup_timeout_sec")
                .and_then(|v| v.as_integer())
                .and_then(|v| u32::try_from(v).ok()),
        });
    }
    Ok(servers)
}

// ─── Skills import ────────────────────────────────────────────────────────────

pub fn import_skills_from_provider(provider_id: &str, project_dir: PathBuf) -> Result<usize> {
    let desc = require_provider(provider_id)?;
    let import_paths = provider_skill_import_paths(desc, &project_dir)?;
    if import_paths.is_empty() {
        return Ok(0);
    }

    let mut existing_ids: HashSet<String> = crate::skill::list_skills(&project_dir)?
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut added = 0usize;

    for skills_dir in import_paths {
        if !skills_dir.exists() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&skills_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }
            let fallback_id = entry.file_name().to_string_lossy().to_string();
            let skill_md = skill_dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }

            let draft = match parse_imported_skill_file(&skill_md, &fallback_id) {
                Ok(Some(draft)) => draft,
                Ok(None) => continue,
                Err(error) => {
                    eprintln!(
                        "[ship] warning: skipped invalid skill import '{}': {}",
                        skill_md.display(),
                        error
                    );
                    continue;
                }
            };

            if existing_ids.contains(&draft.id) {
                continue;
            }

            if crate::skill::create_skill(&project_dir, &draft.id, &draft.name, &draft.content)
                .is_ok()
            {
                existing_ids.insert(draft.id);
                added += 1;
            }
        }
    }

    Ok(added)
}

fn provider_skill_import_paths(
    desc: &ProviderDescriptor,
    project_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let Some(relative_path) = provider_skills_relative_path(desc) else {
        return Ok(Vec::new());
    };

    let project_path_opt = project_dir.parent().map(|root| root.join(relative_path));
    let global_path = home()?.join(relative_path);
    if let Some(project_path) = project_path_opt.as_ref()
        && project_path.exists()
    {
        return Ok(vec![project_path.clone()]);
    }

    if global_path.exists() {
        return Ok(vec![global_path]);
    }

    Ok(project_path_opt.into_iter().collect())
}

fn provider_skills_relative_path(desc: &ProviderDescriptor) -> Option<&'static str> {
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => Some(".claude/skills"),
        SkillsOutput::AgentSkills => Some(".gemini/skills"),
        SkillsOutput::CodexSkills => Some(".agents/skills"),
        SkillsOutput::None => None,
    }
}

#[derive(Debug)]
struct ImportedSkillDraft {
    id: String,
    name: String,
    content: String,
}

#[derive(Deserialize, Default)]
struct ImportedSkillMetadata {
    #[serde(default)]
    display_name: Option<String>,
}

#[derive(Deserialize, Default)]
struct ImportedSkillFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    metadata: Option<ImportedSkillMetadata>,
}

fn parse_imported_skill_file(path: &Path, fallback_id: &str) -> Result<Option<ImportedSkillDraft>> {
    let raw = fs::read_to_string(path)?;
    let normalized = raw.replace("\r\n", "\n");
    let (frontmatter, body) = split_optional_frontmatter(&normalized);

    let id_seed = frontmatter
        .as_ref()
        .and_then(|fm| fm.name.as_deref())
        .unwrap_or(fallback_id);
    let Some(id) = normalize_skill_id(id_seed) else {
        return Ok(None);
    };

    let name = frontmatter
        .and_then(|fm| fm.metadata.and_then(|meta| meta.display_name))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| id.clone());

    let content = strip_managed_skill_header(body).trim().to_string();
    if content.is_empty() {
        return Ok(None);
    }

    Ok(Some(ImportedSkillDraft { id, name, content }))
}

fn split_optional_frontmatter(content: &str) -> (Option<ImportedSkillFrontmatter>, &str) {
    if !content.starts_with("---\n") {
        return (None, content);
    }
    let rest = &content[4..];
    let Some(end) = rest.find("\n---") else {
        return (None, content);
    };
    let frontmatter_raw = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n');
    let parsed = serde_yaml::from_str::<ImportedSkillFrontmatter>(frontmatter_raw).ok();
    (parsed, body)
}

fn strip_managed_skill_header(content: &str) -> &str {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("<!--") {
        return content;
    }
    let Some(end_idx) = trimmed.find("-->") else {
        return content;
    };
    let tail = &trimmed[end_idx + 3..];
    tail.trim_start_matches('\n')
}

fn normalize_skill_id(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in raw.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            prev_dash = false;
            continue;
        }
        if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    let normalized = out.trim_matches('-').to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

// ─── Permissions import ───────────────────────────────────────────────────────

fn import_permissions_from_claude(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let project_path = project_root.join(".claude").join("settings.json");
    let global_path = home()?.join(".claude").join("settings.json");
    let path = if project_path.exists() {
        project_path
    } else if global_path.exists() {
        global_path
    } else {
        return Ok(None);
    };

    let root: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path)?)?;
    let Some(perms) = root.get("permissions").and_then(|p| p.as_object()) else {
        return Ok(None);
    };
    let allow = perms
        .get("allow")
        .and_then(|v| v.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if allow.is_empty() {
        return Ok(None);
    }

    Ok(Some(Permissions {
        tools: ToolPermissions {
            allow,
            ..Default::default()
        },
        ..Default::default()
    }))
}

fn import_permissions_from_gemini(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let path = project_root
        .join(".gemini")
        .join("policies")
        .join("ship-permissions.toml");
    if !path.exists() {
        return Ok(None);
    }

    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let rules = root
        .get("rule")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if rules.is_empty() {
        return Ok(None);
    }

    let mut permissions = Permissions {
        tools: ToolPermissions {
            allow: vec![],
            deny: vec![],
        },
        commands: CommandPermissions {
            allow: vec![],
            deny: vec![],
        },
        agent: AgentLimits {
            require_confirmation: vec![],
        },
        ..Default::default()
    };

    for value in rules {
        let Some(rule) = value.as_table() else {
            continue;
        };
        let decision = rule
            .get("decision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if decision.is_empty() {
            continue;
        }

        let tool_names = value_to_string_list(rule.get("toolName"));
        let mcp_names = value_to_string_list(rule.get("mcpName"));
        let command_prefixes = value_to_string_list(rule.get("commandPrefix"));
        let command_regexes = value_to_string_list(rule.get("commandRegex"));

        for command in command_prefixes {
            match decision.as_str() {
                "allow" => permissions.commands.allow.push(format!("{}*", command)),
                "deny" => permissions.commands.deny.push(format!("{}*", command)),
                "ask_user" => permissions
                    .agent
                    .require_confirmation
                    .push(format!("{}*", command)),
                _ => {}
            }
        }
        for regex in command_regexes {
            let pattern = format!("regex:{}", regex);
            match decision.as_str() {
                "allow" => permissions.commands.allow.push(pattern),
                "deny" => permissions.commands.deny.push(pattern),
                "ask_user" => permissions.agent.require_confirmation.push(pattern),
                _ => {}
            }
        }

        let mut composite_tools = Vec::new();
        if tool_names.is_empty() && mcp_names.is_empty() {
            continue;
        }
        if tool_names.is_empty() {
            for mcp_name in &mcp_names {
                composite_tools.push(format!("{}__*", mcp_name));
            }
        } else if mcp_names.is_empty() {
            composite_tools.extend(tool_names.clone());
        } else {
            for mcp_name in &mcp_names {
                for tool_name in &tool_names {
                    composite_tools.push(format!("{}__{}", mcp_name, tool_name));
                }
            }
        }

        for tool in composite_tools {
            if tool == "run_shell_command" {
                continue;
            }
            match decision.as_str() {
                "allow" => permissions.tools.allow.push(tool),
                "deny" => permissions.tools.deny.push(tool),
                _ => {}
            }
        }
    }

    dedupe_strings(&mut permissions.tools.allow);
    dedupe_strings(&mut permissions.tools.deny);
    dedupe_strings(&mut permissions.commands.allow);
    dedupe_strings(&mut permissions.commands.deny);
    dedupe_strings(&mut permissions.agent.require_confirmation);

    if permissions.tools.allow.is_empty() {
        permissions.tools.allow.push("*".to_string());
    }
    Ok(Some(permissions))
}

fn import_permissions_from_codex(project_dir: &Path) -> Result<Option<Permissions>> {
    let Some(project_root) = project_dir.parent() else {
        return Ok(None);
    };
    let path = project_root.join(".codex").join("config.toml");
    if !path.exists() {
        return Ok(None);
    }

    let root: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let mut imported = false;
    let mut permissions = Permissions::default();

    if let Some(allow) = root.get("allow").and_then(|v| v.as_array()) {
        permissions.commands.allow = allow
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        imported = true;
    }

    if let Some(network_access) = root
        .get("sandbox_workspace_write")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("network_access"))
        .and_then(|v| v.as_bool())
    {
        imported = true;
        permissions.network.policy = if network_access {
            NetworkPolicy::Unrestricted
        } else {
            NetworkPolicy::None
        };
    }

    let prefix_rules = read_codex_prefix_rules_from_value(&root);
    for (prefix, decision) in prefix_rules {
        imported = true;
        let pattern = codex_prefix_to_pattern(&prefix);
        match decision.as_str() {
            "forbidden" => permissions.commands.deny.push(pattern),
            "prompt" => permissions.agent.require_confirmation.push(pattern),
            "allow" => permissions.commands.allow.push(pattern),
            _ => {}
        }
    }

    let rules_dir = project_root.join(".codex").join("rules");
    if rules_dir.is_dir()
        && let Ok(entries) = fs::read_dir(&rules_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rules") {
                continue;
            }
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            for (words, decision) in parse_starlark_prefix_rules(&content) {
                imported = true;
                let pattern = format!("{} *", words.join(" "));
                match decision.as_str() {
                    "allow" => permissions.commands.allow.push(pattern),
                    "forbidden" => permissions.commands.deny.push(pattern),
                    "prompt" => permissions.agent.require_confirmation.push(pattern),
                    _ => {}
                }
            }
        }
    }

    if !imported {
        return Ok(None);
    }
    dedupe_strings(&mut permissions.commands.allow);
    dedupe_strings(&mut permissions.commands.deny);
    dedupe_strings(&mut permissions.agent.require_confirmation);
    Ok(Some(permissions))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn value_to_string_list(value: Option<&toml::Value>) -> Vec<String> {
    match value {
        Some(toml::Value::String(s)) => vec![s.to_string()],
        Some(toml::Value::Array(values)) => values
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn read_codex_prefix_rules_from_value(root: &toml::Value) -> Vec<(String, String)> {
    root.get("rules")
        .and_then(|v| v.as_table())
        .and_then(|table| table.get("prefix_rules"))
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| {
                    let table = entry.as_table()?;
                    let prefix = table.get("prefix")?.as_str()?.to_string();
                    let decision = table.get("decision")?.as_str()?.to_string();
                    Some((prefix, decision))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn codex_prefix_to_pattern(prefix: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        "*".to_string()
    } else {
        format!("{trimmed} *")
    }
}

fn parse_starlark_prefix_rules(content: &str) -> Vec<(Vec<String>, String)> {
    let mut results = Vec::new();
    let mut remaining = content;
    while let Some(start) = remaining.find("prefix_rule(") {
        let after_paren = start + "prefix_rule(".len();
        let rest = &remaining[after_paren..];
        if let Some(end) = rest.find(')') {
            let block = &rest[..end];
            if let Some(entry) = parse_starlark_prefix_rule_block(block) {
                results.push(entry);
            }
            remaining = &rest[end + 1..];
        } else {
            break;
        }
    }
    results
}

fn parse_starlark_prefix_rule_block(block: &str) -> Option<(Vec<String>, String)> {
    let decision = {
        let line = block.lines().find(|l| l.trim().starts_with("decision"))?;
        let after_eq = line.split_once('=')?.1.trim();
        let inner = after_eq.trim_start_matches('"');
        let end = inner.find('"')?;
        inner[..end].to_string()
    };

    let pattern_line = block.lines().find(|l| l.trim().starts_with("pattern"))?;
    let bracket_start = pattern_line.find('[')?;
    let bracket_end = pattern_line.rfind(']')?;
    if bracket_end <= bracket_start {
        return None;
    }
    let inner = &pattern_line[bracket_start + 1..bracket_end];

    let words: Vec<String> = inner
        .split(',')
        .filter_map(|part| {
            let s = part.trim();
            if s.len() > 2 && s.starts_with('"') && s.ends_with('"') {
                Some(s[1..s.len() - 1].to_string())
            } else {
                None
            }
        })
        .collect();

    if words.is_empty() || decision.is_empty() {
        return None;
    }

    Some((words, decision))
}

fn dedupe_strings(values: &mut Vec<String>) {
    let mut seen = HashSet::new();
    values.retain(|item| seen.insert(item.clone()));
}

fn home() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let path = PathBuf::from(home);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }
    home::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}
