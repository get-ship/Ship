//! File writing logic for `ship add --from` — writes agent profiles, skill
//! files, and merges dependencies into the project manifest.

use std::path::Path;

use anyhow::{Context, Result};

use crate::add::from::{AgentBundle, SkillBundle, TransferBundle};

/// Write the transfer bundle to `.ship/`.
pub fn write_bundle(project_root: &Path, bundle: &TransferBundle) -> Result<()> {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(project_root);

    // 1. Write agent profile as JSONC.
    write_agent(&ship_dir, &bundle.agent)?;

    // 2. Write inline skills.
    for (skill_id, skill) in &bundle.skills {
        write_skill(&ship_dir, skill_id, skill)?;
    }

    // 3. Merge dependencies into manifest.
    if !bundle.dependencies.is_empty() {
        merge_dependencies(&ship_dir, &bundle.dependencies)?;
    }

    Ok(())
}

/// Write an agent JSONC profile.
pub fn write_agent(ship_dir: &Path, agent: &AgentBundle) -> Result<()> {
    let agents_dir = ship_dir.join("agents");
    std::fs::create_dir_all(&agents_dir)?;

    let dest = agents_dir.join(format!("{}.jsonc", agent.id));
    if dest.exists() {
        eprintln!("warning: overwriting existing agent '{}'", agent.id);
    }

    let profile = build_agent_jsonc(agent);
    std::fs::write(&dest, profile).with_context(|| format!("writing agent {}", dest.display()))?;

    Ok(())
}

/// Build JSONC content for an agent profile in the canonical `agent.schema.json`
/// nested shape. The previous flat emission (`id`, `skills: [...]` at root) was
/// rejected by `to_profile` (which guards on `agent:` block presence), making
/// `ship add --from` → `ship use <id>` silently broken. This function now emits
/// the conformant nested shape that `to_profile` and `read_agent_profile_at` expect.
pub fn build_agent_jsonc(agent: &AgentBundle) -> String {
    // ── agent block ──────────────────────────────────────────────────────────
    let name = agent.name.as_deref().unwrap_or(&agent.id);
    let mut agent_block = serde_json::Map::new();
    agent_block.insert("id".into(), serde_json::json!(agent.id));
    agent_block.insert("name".into(), serde_json::json!(name));
    if let Some(ref desc) = agent.description {
        agent_block.insert("description".into(), serde_json::json!(desc));
    }

    let mut root = serde_json::Map::new();
    root.insert("agent".into(), serde_json::Value::Object(agent_block));

    // ── skills → skills.refs ─────────────────────────────────────────────────
    if !agent.skills.is_empty() {
        root.insert("skills".into(), serde_json::json!({ "refs": agent.skills }));
    }

    // ── mcp_servers → mcp.servers ────────────────────────────────────────────
    // The bundle's mcp_servers is a flexible Value (string or object with `id`).
    let servers: Vec<String> = agent.mcp_servers.iter()
        .filter_map(mcp_server_id)
        .collect();
    if !servers.is_empty() {
        root.insert("mcp".into(), serde_json::json!({ "servers": servers }));
    }

    // ── rules Vec → rules.inline (joined) ────────────────────────────────────
    if !agent.rules.is_empty() {
        root.insert("rules".into(), serde_json::json!({ "inline": agent.rules.join("\n") }));
    }

    // ── model → harness_settings.floki.model ─────────────────────────────────
    if let Some(ref model) = agent.model {
        root.insert(
            "harness_settings".into(),
            serde_json::json!({ "floki": { "model": model } }),
        );
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(root))
        .unwrap_or_else(|_| "{}".into())
}

/// Extract a string ID from an MCP server value that may be a bare string or
/// an object with an `id` key. Returns `None` for any other shape.
fn mcp_server_id(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(m) => m.get("id")?.as_str().map(str::to_string),
        _ => None,
    }
}

/// Write inline skill files to `.ship/skills/<id>/`.
pub fn write_skill(ship_dir: &Path, skill_id: &str, skill: &SkillBundle) -> Result<()> {
    let skill_dir = ship_dir.join("skills").join(skill_id);
    std::fs::create_dir_all(&skill_dir)?;

    for (rel_path, content) in &skill.files {
        let dest = skill_dir.join(rel_path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, content)
            .with_context(|| format!("writing skill file {}", dest.display()))?;
    }

    Ok(())
}

/// Merge dependencies into the existing ship.jsonc manifest.
pub fn merge_dependencies(
    ship_dir: &Path,
    deps: &std::collections::HashMap<String, String>,
) -> Result<()> {
    let manifest_path = ship_dir.join("ship.jsonc");
    if !manifest_path.exists() {
        anyhow::bail!("no ship.jsonc found to add dependencies to");
    }

    let raw = std::fs::read_to_string(&manifest_path)?;
    let mut updated = raw;
    for (path, version) in deps {
        if updated.contains(path) {
            continue; // Already present.
        }
        updated = crate::add::append_dependency_jsonc(&updated, path, version);
    }

    std::fs::write(&manifest_path, &updated)
        .with_context(|| format!("writing {}", manifest_path.display()))?;

    Ok(())
}
