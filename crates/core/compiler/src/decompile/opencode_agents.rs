//! OpenCode agent decompilation — parse `"agent"` entries from `opencode.json`
//! into Ship [`AgentProfile`] structs.

use std::collections::HashMap;

use serde_json::Value as Json;

use crate::types::{
    AgentProfile, McpRefs, PluginRefs, ProfileMeta, ProfilePermissions, ProfileRules, SkillRefs,
};

/// Known agent-level keys that map to structured Ship fields (not passthrough).
const KNOWN_AGENT_KEYS: &[&str] = &["description", "model", "prompt", "permission"];

/// Parse an OpenCode agent entry into an [`AgentProfile`].
pub(super) fn parse_opencode_agent(name: &str, entry: &Json) -> Option<AgentProfile> {
    let obj = entry.as_object()?;

    let description = obj
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);
    let prompt = obj.get("prompt").and_then(|v| v.as_str()).map(String::from);
    let permissions = obj
        .get("permission")
        .map(parse_agent_permissions)
        .unwrap_or_default();

    // Collect unknown fields + model into harness_settings.opencode
    let mut oc_settings = serde_json::Map::new();
    if let Some(Json::String(m)) = obj.get("model") {
        oc_settings.insert("model".to_string(), Json::String(m.clone()));
    }
    for (k, v) in obj {
        if !KNOWN_AGENT_KEYS.contains(&k.as_str()) {
            oc_settings.insert(k.clone(), v.clone());
        }
    }

    let mut harness_settings = HashMap::new();
    if !oc_settings.is_empty() {
        harness_settings.insert("opencode".to_string(), Json::Object(oc_settings));
    }

    Some(AgentProfile {
        agent: ProfileMeta {
            id: name.to_string(),
            name: name.to_string(),
            version: None,
            description,
            role: None,
            tags: vec![],
            tools_preset: None,
            active_tools: vec![],
            icon: None,
        },
        capabilities: Vec::new(),
        skills: SkillRefs::default(),
        mcp: McpRefs::default(),
        plugins: PluginRefs::default(),
        permissions,
        rules: ProfileRules { inline: prompt },
        hooks: Default::default(),
        apps: Default::default(),
        harness_settings,
    })
}

/// Reverse-translate OpenCode permission object into Ship's [`ProfilePermissions`].
///
/// OpenCode keys map to Ship tool names:
///   bash -> Bash, edit -> Edit/Write, read -> Read, glob -> Glob,
///   grep -> Grep, list -> LS, webfetch -> WebFetch, websearch -> WebSearch.
///
/// Values: "allow" -> tools_allow, "deny" -> tools_deny, "ask" -> tools_ask.
/// Granular objects like `{ "git *": "allow" }` -> `Bash(git *)` in the appropriate list.
fn parse_agent_permissions(value: &Json) -> ProfilePermissions {
    let mut perms = ProfilePermissions::default();
    let obj = match value.as_object() {
        Some(o) => o,
        None => return perms,
    };

    for (key, val) in obj {
        let tool_names = opencode_key_to_ship_tools(key);
        if tool_names.is_empty() {
            continue;
        }
        match val {
            Json::String(action) => {
                for tool in &tool_names {
                    push_permission(&mut perms, tool, action);
                }
            }
            Json::Object(globs) => {
                // Granular: { "git *": "allow", "rm *": "deny" }
                for (glob, action_val) in globs {
                    if let Some(action) = action_val.as_str() {
                        let base = &tool_names[0];
                        let pattern = format!("{base}({glob})");
                        push_permission(&mut perms, &pattern, action);
                    }
                }
            }
            _ => {}
        }
    }

    perms
}

/// Map an OpenCode permission key to Ship tool name(s).
fn opencode_key_to_ship_tools(key: &str) -> Vec<String> {
    match key {
        "bash" => vec!["Bash".to_string()],
        "edit" => vec!["Edit".to_string(), "Write".to_string()],
        "read" => vec!["Read".to_string()],
        "glob" => vec!["Glob".to_string()],
        "grep" => vec!["Grep".to_string()],
        "list" => vec!["LS".to_string()],
        "webfetch" => vec!["WebFetch".to_string()],
        "websearch" => vec!["WebSearch".to_string()],
        _ => vec![],
    }
}

/// Push a tool pattern into the appropriate permission list.
fn push_permission(perms: &mut ProfilePermissions, tool: &str, action: &str) {
    match action {
        "allow" => perms.tools_allow.push(tool.to_string()),
        "deny" => perms.tools_deny.push(tool.to_string()),
        "ask" => perms.tools_ask.push(tool.to_string()),
        _ => {}
    }
}

