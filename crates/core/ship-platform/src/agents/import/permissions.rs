//! Import per-harness permission rules into the project's
//! `permissions.jsonc`. Each harness format has its own grammar:
//!
//! - **Claude** — `.claude/settings.json`'s `permissions.allow` becomes
//!   `tools.allow`. Anything else is left to the user.
//! - **Gemini** — `.gemini/policies/ship-permissions.toml`'s `[[rule]]`
//!   entries map decisions (allow / deny / ask_user) onto Ship's
//!   `tools` and `commands` lists. Composite rules (tool × mcp) are
//!   expanded with the `mcp__tool` separator.
//! - **Codex** — `.codex/config.toml`'s top-level `allow` array,
//!   `sandbox_workspace_write.network_access`, prefix rules in
//!   `[rules.prefix_rules]`, plus Starlark rules in
//!   `.codex/rules/*.rules` (parsed with a hand-rolled mini-grammar
//!   covering `decision` + `pattern`).
//!
//! Dispatch fan-out (`import_permissions_from_harness`) lives here so
//! all three variants share their helpers and the variant code stays
//! local. Helpers (`dedupe_strings`, TOML/Starlark accessors) are
//! private to this module since no other importer needs them.

use crate::permissions::{
    AgentLimits, CommandPermissions, NetworkPolicy, Permissions, ToolPermissions, save_permissions,
};
use anyhow::{Result, anyhow};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::home;

pub fn import_permissions_from_harness(harness_id: &str, project_dir: PathBuf) -> Result<bool> {
    let imported = match harness_id {
        "claude" => import_permissions_from_claude(&project_dir)?,
        "gemini" => import_permissions_from_gemini(&project_dir)?,
        "codex" => import_permissions_from_codex(&project_dir)?,
        _ => return Err(anyhow!("Unsupported harness '{}'", harness_id)),
    };

    if let Some(permissions) = imported {
        save_permissions(project_dir, &permissions)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

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

// ── Private helpers ─────────────────────────────────────────────────────────

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
