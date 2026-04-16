//! Agent discovery helpers for the `agents` daemon-rendered view.
//!
//! Reads agent configs from `<ship_dir>/agents/*.{jsonc,json}`. JSONC is
//! stripped locally to stay independent of the compiler crate (we don't
//! need full grammar-accurate parsing — just line/block comments and
//! trailing commas to feed serde_json).

use serde::Serialize;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
pub struct AgentSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skills_count: usize,
    pub providers_count: usize,
    pub mcp_count: usize,
    pub rules_count: usize,
    /// First letter of `name` uppercased, for the avatar glyph.
    pub letter: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AgentDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub letter: String,
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub rules: Vec<String>,
    pub providers: Vec<String>,
    pub default_provider: Option<String>,
    pub permissions_tier: String,
}

pub(crate) fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_block = false;
    let mut in_str = false;
    let mut it = input.chars().peekable();
    while let Some(c) = it.next() {
        if in_block {
            if c == '*' && it.peek() == Some(&'/') {
                it.next();
                in_block = false;
            }
            continue;
        }
        if in_str {
            out.push(c);
            if c == '\\' {
                if let Some(&n) = it.peek() {
                    out.push(n);
                    it.next();
                }
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            match it.peek() {
                Some(&'/') => {
                    for r in it.by_ref() {
                        if r == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some(&'*') => {
                    it.next();
                    in_block = true;
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    // Second pass: drop trailing commas before `}` or `]`.
    // Walk as bytes so we can peek ahead cheaply; preserve multi-byte UTF-8
    // sequences verbatim by copying the original byte slice (not byte-casting
    // to char, which mangles any codepoint above U+007F).
    let src = out.as_bytes();
    let mut cleaned_bytes: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        if src[i] == b',' {
            let mut j = i + 1;
            while j < src.len() && src[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < src.len() && (src[j] == b'}' || src[j] == b']') {
                i += 1;
                continue;
            }
        }
        cleaned_bytes.push(src[i]);
        i += 1;
    }
    // Safe: we only dropped ASCII commas at valid char boundaries, every other
    // byte was copied unchanged from the original UTF-8 String.
    String::from_utf8(cleaned_bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

pub(crate) fn first_letter(s: &str) -> String {
    s.chars()
        .next()
        .map(|c| c.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

fn parse_file(path: &Path) -> Option<(String, serde_json::Value)> {
    let id = path.file_stem().and_then(|s| s.to_str())?.to_string();
    let raw = std::fs::read_to_string(path).ok()?;
    let json_str = strip_jsonc(&raw);
    let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;
    Some((id, parsed))
}

fn extract_str_array(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|item| match item {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Object(map) => map
                        .get("id")
                        .or_else(|| map.get("name"))
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Resolve a collection-ish field from an agent config, tolerating the two
/// shapes Ship actually emits:
///   1. Flat: `"skills": ["a", "b"]` or `[{"id": "a"}, {"name": "b"}]`
///   2. Nested: `"skills": { "refs": [...] }` / `"mcp": { "servers": [...] }`
/// The `nested_keys` parameter lists candidate sub-keys to inspect when the
/// top-level value is an object; the first one that contains an array wins.
fn resolve_array_field(
    obj: &serde_json::Value,
    top_keys: &[&str],
    nested_keys: &[&str],
) -> Vec<String> {
    for key in top_keys {
        let Some(v) = obj.get(*key) else { continue };
        if v.is_array() {
            return extract_str_array(v);
        }
        if let Some(inner) = v.as_object() {
            for nk in nested_keys {
                if let Some(arr) = inner.get(*nk) {
                    if arr.is_array() {
                        return extract_str_array(arr);
                    }
                }
            }
        }
    }
    Vec::new()
}

/// List agents from `<ship_dir>/agents/*.{jsonc,json}`.
pub fn list_agents(ship_dir: &Path) -> Vec<AgentSummary> {
    let dir = ship_dir.join("agents");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if !path.is_file() || !matches!(ext, Some("jsonc" | "json")) {
            continue;
        }
        let Some((id, parsed)) = parse_file(&path) else { continue };
        let agent_obj = parsed.get("agent").unwrap_or(&parsed);
        let name = agent_obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        let description = agent_obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let skills = resolve_array_field(&parsed, &["skills"], &["refs", "ids"]);
        let providers = resolve_array_field(agent_obj, &["providers"], &["ids"]);
        let mcp = resolve_array_field(
            &parsed,
            &["mcp_servers", "mcpServers", "mcp"],
            &["servers", "refs"],
        );
        let rules = resolve_array_field(&parsed, &["rules"], &["refs", "ids"]);
        let letter = first_letter(&name);
        out.push(AgentSummary {
            id,
            name,
            description,
            skills_count: skills.len(),
            providers_count: providers.len(),
            mcp_count: mcp.len(),
            rules_count: rules.len(),
            letter,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load a single agent by id.
pub fn load_agent(ship_dir: &Path, id: &str) -> Option<AgentDetail> {
    let dir = ship_dir.join("agents");
    for ext in ["jsonc", "json"] {
        let path = dir.join(format!("{id}.{ext}"));
        if !path.is_file() {
            continue;
        }
        let (file_id, parsed) = parse_file(&path)?;
        let agent_obj = parsed.get("agent").unwrap_or(&parsed);
        let name = agent_obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&file_id)
            .to_string();
        let description = agent_obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let skills = resolve_array_field(&parsed, &["skills"], &["refs", "ids"]);
        let providers = resolve_array_field(agent_obj, &["providers"], &["ids"]);
        let mcp_servers = resolve_array_field(
            &parsed,
            &["mcp_servers", "mcpServers", "mcp"],
            &["servers", "refs"],
        );
        let rules = resolve_array_field(&parsed, &["rules"], &["refs", "ids"]);
        let default_provider = agent_obj
            .get("default_provider")
            .or_else(|| agent_obj.get("defaultProvider"))
            .or_else(|| agent_obj.get("provider"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        // Permissions tier lives under `permissions.tier` (flat) or — for
        // project configs that use the preset name — `permissions.preset`
        // with a `ship-<tier>` prefix. Strip the prefix to canonicalize.
        let permissions_obj = parsed.get("permissions");
        let permissions_tier = permissions_obj
            .and_then(|p| p.get("tier").and_then(|t| t.as_str()).map(|s| s.to_string()))
            .or_else(|| {
                permissions_obj
                    .and_then(|p| p.get("preset").and_then(|t| t.as_str()))
                    .map(|s| s.trim_start_matches("ship-").to_string())
            })
            .unwrap_or_else(|| "standard".to_string());
        return Some(AgentDetail {
            id: file_id,
            name: name.clone(),
            description,
            letter: first_letter(&name),
            skills,
            mcp_servers,
            rules,
            providers,
            default_provider,
            permissions_tier,
        });
    }
    None
}

#[cfg(test)]
#[path = "agents_state_tests.rs"]
mod tests;
