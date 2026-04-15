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
    let bytes = out.as_bytes();
    let mut cleaned = String::with_capacity(out.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                i += 1;
                continue;
            }
        }
        cleaned.push(bytes[i] as char);
        i += 1;
    }
    cleaned
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
        let skills = parsed.get("skills").map(extract_str_array).unwrap_or_default();
        let providers = agent_obj
            .get("providers")
            .map(extract_str_array)
            .unwrap_or_default();
        let mcp = parsed
            .get("mcp_servers")
            .or_else(|| parsed.get("mcpServers"))
            .map(extract_str_array)
            .unwrap_or_default();
        let rules = parsed.get("rules").map(extract_str_array).unwrap_or_default();
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
        let skills = parsed.get("skills").map(extract_str_array).unwrap_or_default();
        let providers = agent_obj
            .get("providers")
            .map(extract_str_array)
            .unwrap_or_default();
        let mcp_servers = parsed
            .get("mcp_servers")
            .or_else(|| parsed.get("mcpServers"))
            .map(extract_str_array)
            .unwrap_or_default();
        let rules = parsed.get("rules").map(extract_str_array).unwrap_or_default();
        let default_provider = agent_obj
            .get("default_provider")
            .or_else(|| agent_obj.get("defaultProvider"))
            .or_else(|| agent_obj.get("provider"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let permissions_tier = parsed
            .get("permissions")
            .and_then(|p| p.get("tier").and_then(|t| t.as_str()))
            .unwrap_or("standard")
            .to_string();
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
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_agent(ship_dir: &Path, id: &str, body: &str) {
        let d = ship_dir.join("agents");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join(format!("{id}.jsonc")), body).unwrap();
    }

    #[test]
    fn empty_dir_returns_empty_vec() {
        let tmp = tempdir().unwrap();
        assert!(list_agents(tmp.path()).is_empty());
    }

    #[test]
    fn parses_flat_and_nested_shapes() {
        let tmp = tempdir().unwrap();
        write_agent(
            tmp.path(),
            "coder",
            r#"{
                // flat shape
                "name": "Coder",
                "description": "Writes code.",
                "skills": ["tdd", "refactor"],
                "providers": ["claude", "codex"],
                "mcp_servers": ["ship"]
            }"#,
        );
        write_agent(
            tmp.path(),
            "reviewer",
            r#"{
                "agent": { "name": "Reviewer", "description": "Reviews PRs.", "providers": ["claude"] },
                "skills": [{"id": "review"}, "style"],
                "rules": ["kind", "clear"]
            }"#,
        );
        let out = list_agents(tmp.path());
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].name, "Coder");
        assert_eq!(out[0].skills_count, 2);
        assert_eq!(out[0].providers_count, 2);
        assert_eq!(out[0].mcp_count, 1);
        assert_eq!(out[0].letter, "C");
        assert_eq!(out[1].name, "Reviewer");
        assert_eq!(out[1].skills_count, 2);
        assert_eq!(out[1].rules_count, 2);
    }

    #[test]
    fn load_agent_returns_none_for_missing() {
        let tmp = tempdir().unwrap();
        assert!(load_agent(tmp.path(), "nope").is_none());
    }

    #[test]
    fn load_agent_returns_detail_with_defaults() {
        let tmp = tempdir().unwrap();
        write_agent(
            tmp.path(),
            "coder",
            r#"{ "name": "Coder", "description": "Writes code." }"#,
        );
        let d = load_agent(tmp.path(), "coder").unwrap();
        assert_eq!(d.name, "Coder");
        assert_eq!(d.permissions_tier, "standard");
        assert_eq!(d.letter, "C");
    }

    #[test]
    fn strip_jsonc_handles_comments() {
        let src = r#"{
            // line
            /* block */
            "x": 1,
        }"#;
        let v: serde_json::Value = serde_json::from_str(&strip_jsonc(src)).unwrap();
        assert_eq!(v["x"], 1);
    }
}
