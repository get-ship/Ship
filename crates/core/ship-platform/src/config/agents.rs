use super::project::AgentProfile;
use anyhow::{Context, Result};
use ship_substrate::fs_util::strip_jsonc_comments;
use ship_substrate::project::agents_ns;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Project a canonical [`compiler::AgentProfile`] (nested file-format type)
/// into the platform's flat [`AgentProfile`] (runtime/UI view). This is the
/// single bridge between the file-canonical schema and the platform's
/// in-memory shape. Lossy by design — the nested type carries fields the
/// platform UI does not surface (`harness_settings`, `apps`, `plugins`,
/// `rules.inline`, etc.).
fn project_canonical(src: &compiler::AgentProfile) -> AgentProfile {
    AgentProfile {
        id: src.agent.id.clone(),
        name: src.agent.name.clone(),
        description: src.agent.description.clone(),
        role: src.agent.role.clone(),
        tools_preset: src.agent.tools_preset.clone(),
        active_tools: src.agent.active_tools.clone(),
        mcp_servers: src.mcp.servers.clone(),
        skills: src.skills.refs.clone(),
        rules: Vec::new(),
        hooks: Vec::new(),
        permissions: super::types::PermissionConfig::default(),
        target_agents: Vec::new(),
    }
}

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

fn agent_file(ship_dir: &Path, id: &str) -> std::path::PathBuf {
    agents_ns(ship_dir).join(format!("{id}.jsonc"))
}

pub(super) fn get_agents_config(ship_dir: &Path) -> Result<Vec<AgentProfile>> {
    let dir = agents_ns(ship_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut agents = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("jsonc") {
            continue;
        }
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let stripped = strip_jsonc_comments(&raw);
        // Files are authored against the canonical schema (top-level `agent`
        // block). Parse via `compiler::AgentProfile`, then project to the
        // platform's flat runtime view.
        let value: serde_json::Value = serde_json::from_str(&stripped)
            .with_context(|| format!("parse agent profile {}", path.display()))?;
        if value.get("agent").is_some() {
            let canonical: compiler::AgentProfile = serde_json::from_value(value)
                .with_context(|| format!("parse agent profile {}", path.display()))?;
            agents.push(project_canonical(&canonical));
        } else {
            // Pre-canonical flat shape `{"id": ..., "name": ...}` written by
            // an older `save_agents_config`. Kept for the round-trip path
            // until every project re-saves into canonical form.
            let flat: AgentProfile = serde_json::from_value(value)
                .with_context(|| format!("parse agent profile {}", path.display()))?;
            agents.push(flat);
        }
    }
    agents.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(agents)
}

pub(super) fn save_agents_config(ship_dir: &Path, agents: &[AgentProfile]) -> Result<()> {
    let dir = agents_ns(ship_dir);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let keep: HashSet<String> = agents.iter().map(|a| a.id.clone()).collect();

    // Drop files for agents no longer in the desired set.
    if dir.exists() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("jsonc") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if !keep.contains(id) {
                fs::remove_file(&path)
                    .with_context(|| format!("remove {}", path.display()))?;
            }
        }
    }

    for agent in agents {
        let mut profile = agent.clone();
        profile.rules = profile
            .rules
            .iter()
            .map(|id| normalize_rule_external_id(id))
            .collect();
        let path = agent_file(ship_dir, &agent.id);
        let content = render_agent_file(&path, &profile)?;
        fs::write(&path, content)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

/// Render an agent profile to JSONC. When `path` already holds a canonical
/// nested-shape file, the platform-managed fields are merged in and the
/// remaining hand-authored content (`hooks`, `plugins`, `rules.inline`,
/// `harness_settings`, `apps`, schema reference) is preserved. When the
/// file is missing or in legacy flat form, the platform-managed fields are
/// emitted as a minimal canonical nested file.
fn render_agent_file(path: &Path, flat: &AgentProfile) -> Result<String> {
    let existing = fs::read_to_string(path).ok();
    let parsed: Option<serde_json::Value> = existing
        .as_ref()
        .map(|raw| strip_jsonc_comments(raw))
        .and_then(|stripped| serde_json::from_str(&stripped).ok());

    let canonical_doc = parsed.filter(|v| v.get("agent").is_some());

    if let Some(mut doc) = canonical_doc {
        merge_flat_into_canonical(&mut doc, flat);
        let json = serde_json::to_string_pretty(&doc)?;
        Ok(format!("{json}\n"))
    } else {
        // No existing canonical file — emit a minimal canonical document.
        let mut agent_block = serde_json::json!({
            "id": flat.id,
            "name": flat.name,
        });
        if let Some(desc) = &flat.description {
            agent_block["description"] = serde_json::Value::String(desc.clone());
        }
        if let Some(role) = &flat.role {
            agent_block["role"] = serde_json::Value::String(role.clone());
        }
        if let Some(preset) = &flat.tools_preset {
            agent_block["tools_preset"] = serde_json::Value::String(preset.clone());
        }
        if !flat.active_tools.is_empty() {
            agent_block["active_tools"] = serde_json::Value::Array(
                flat.active_tools
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            );
        }
        if !flat.target_agents.is_empty() {
            agent_block["harnesses"] = serde_json::Value::Array(
                flat.target_agents
                    .iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            );
        }
        let canonical = serde_json::json!({
            "agent": agent_block,
            "skills": { "refs": flat.skills },
            "mcp": { "servers": flat.mcp_servers },
        });
        let json = serde_json::to_string_pretty(&canonical)?;
        Ok(format!("{json}\n"))
    }
}

/// Overlay the platform-managed fields of a flat [`AgentProfile`] onto a
/// canonical nested JSONC document. Fields the platform does not own
/// (`hooks`, `plugins`, `rules.inline`, `harness_settings`, `apps`) are
/// left untouched.
fn merge_flat_into_canonical(doc: &mut serde_json::Value, flat: &AgentProfile) {
    let Some(root) = doc.as_object_mut() else {
        return;
    };
    let agent_obj = root
        .entry("agent".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(a) = agent_obj.as_object_mut() {
        a.insert("id".to_string(), serde_json::Value::String(flat.id.clone()));
        a.insert(
            "name".to_string(),
            serde_json::Value::String(flat.name.clone()),
        );
        if let Some(desc) = &flat.description {
            a.insert(
                "description".to_string(),
                serde_json::Value::String(desc.clone()),
            );
        }
        if let Some(role) = &flat.role {
            a.insert("role".to_string(), serde_json::Value::String(role.clone()));
        }
        if let Some(preset) = &flat.tools_preset {
            a.insert(
                "tools_preset".to_string(),
                serde_json::Value::String(preset.clone()),
            );
        }
        if !flat.active_tools.is_empty() {
            a.insert(
                "active_tools".to_string(),
                serde_json::Value::Array(
                    flat.active_tools
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
        if !flat.target_agents.is_empty() {
            a.insert(
                "harnesses".to_string(),
                serde_json::Value::Array(
                    flat.target_agents
                        .iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
    }

    let skills_obj = root
        .entry("skills".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(s) = skills_obj.as_object_mut() {
        s.insert(
            "refs".to_string(),
            serde_json::Value::Array(
                flat.skills
                    .iter()
                    .map(|v| serde_json::Value::String(v.clone()))
                    .collect(),
            ),
        );
    }

    let mcp_obj = root
        .entry("mcp".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(m) = mcp_obj.as_object_mut() {
        m.insert(
            "servers".to_string(),
            serde_json::Value::Array(
                flat.mcp_servers
                    .iter()
                    .map(|v| serde_json::Value::String(v.clone()))
                    .collect(),
            ),
        );
    }
}

