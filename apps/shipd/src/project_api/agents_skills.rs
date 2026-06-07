//! Agent and skill config endpoints — serve .ship/agents/ and .ship/skills/ data.

use axum::{Json, extract::{Path, State}, http::StatusCode};

use crate::routes::rest_api::{ApiState, MeshResponse};

use super::{err_response, ok_response};

/// GET /api/v1/projects/{id}/agents
pub async fn list_agents(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = state.resolve_worktree(&id)?;
    let agents_dir = ship_substrate::ShipHome::project_ship_dir(&worktree).join("agents");

    if !agents_dir.exists() {
        return Ok(ok_response(serde_json::json!({ "agents": [] })));
    }

    let entries = std::fs::read_dir(&agents_dir).map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to read agents dir: {e}"),
        )
    })?;

    let mut agents = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if !path.is_file() || !matches!(ext, Some("jsonc" | "json")) {
            continue;
        }

        let id_str = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let json_str = ship_substrate::fs_util::strip_jsonc_comments(&raw);
        let parsed: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Agent config may nest under "agent" key or be flat
        let agent_obj = parsed.get("agent").unwrap_or(&parsed);
        agents.push(serde_json::json!({
            "id": agent_obj.get("id").and_then(|v| v.as_str()).unwrap_or(&id_str),
            "name": agent_obj.get("name").and_then(|v| v.as_str()).unwrap_or(&id_str),
            "description": agent_obj.get("description").and_then(|v| v.as_str()),
            "skills": parsed.get("skills").cloned().unwrap_or(serde_json::json!([])),
            "harnesses": agent_obj.get("harnesses").cloned().unwrap_or(serde_json::json!([])),
        }));
    }

    Ok(ok_response(serde_json::json!({ "agents": agents })))
}

/// GET /api/v1/projects/{id}/skills
pub async fn list_skills(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = state.resolve_worktree(&id)?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(&worktree);

    // Use read_skill_paths to discover all skill directories (respects skill_paths config)
    let skill_dirs = ship_agents::agent_domain::skill_paths::read_skill_paths(&ship_dir, &worktree);

    let mut skills = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    for skills_dir in &skill_dirs {
        if !skills_dir.exists() { continue; }
        let dir_entries = match std::fs::read_dir(skills_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in dir_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if !seen_ids.insert(dir_name.clone()) { continue; }
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let mut name = dir_name.clone();
        let mut stable_id: Option<String> = None;
        let mut description: Option<String> = None;

        // Try to read SKILL.md frontmatter for metadata
        let skill_md = path.join("SKILL.md");
        if skill_md.is_file()
            && let Ok(content) = std::fs::read_to_string(&skill_md)
            && content.starts_with("---")
            && let Some(end) = content[3..].find("---")
        {
            let frontmatter = &content[3..3 + end];
            for line in frontmatter.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("name:") {
                    name = val.trim().trim_matches('"').to_string();
                } else if let Some(val) = line.strip_prefix("stable_id:") {
                    stable_id = Some(val.trim().trim_matches('"').to_string());
                } else if let Some(val) = line.strip_prefix("description:") {
                    description = Some(val.trim().trim_matches('"').to_string());
                }
            }
        }

        // Recursively list all files in the skill directory
        let files = collect_files_recursive(&path, &path);

        // Read SKILL.md content for the editor
        let skill_content = std::fs::read_to_string(&skill_md).unwrap_or_default();

        // Read vars.json if present
        let vars_schema: Option<serde_json::Value> = std::fs::read_to_string(path.join("assets/vars.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

        // Read reference docs
        let mut reference_docs = serde_json::Map::new();
        let refs_dir = path.join("references");
        if refs_dir.is_dir() {
            for rel_path in collect_files_recursive(&path, &refs_dir) {
                let full = path.join(&rel_path);
                if let Ok(content) = std::fs::read_to_string(&full) {
                    reference_docs.insert(rel_path, serde_json::Value::String(content));
                }
            }
        }

        // Read evals if present
        let evals: Option<serde_json::Value> = std::fs::read_to_string(path.join("evals/evals.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

        // Parse tags and authors from frontmatter
        let mut tags: Vec<String> = Vec::new();
        let mut authors: Vec<String> = Vec::new();
        let mut artifacts: Vec<String> = Vec::new();
        if skill_md.is_file()
            && let Ok(content) = std::fs::read_to_string(&skill_md)
            && content.starts_with("---")
            && let Some(end) = content[3..].find("---")
        {
            let fm = &content[3..3 + end];
            for line in fm.lines() {
                let line = line.trim();
                if let Some(val) = line.strip_prefix("tags:") {
                    tags = parse_yaml_list(val);
                } else if let Some(val) = line.strip_prefix("authors:") {
                    authors = parse_yaml_list(val);
                } else if let Some(val) = line.strip_prefix("artifacts:") {
                    artifacts = parse_yaml_list(val);
                }
            }
        }

        skills.push(serde_json::json!({
            "id": dir_name,
            "name": name,
            "stable_id": stable_id,
            "description": description,
            "content": skill_content,
            "files": files,
            "vars_schema": vars_schema,
            "reference_docs": reference_docs,
            "evals": evals,
            "tags": tags,
            "authors": authors,
            "artifacts": artifacts,
        }));
    } // entry loop
    } // skill_dirs loop

    Ok(ok_response(serde_json::json!({ "skills": skills })))
}

/// Recursively collect all file paths relative to `root`.
fn collect_files_recursive(root: &std::path::Path, dir: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return files };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            files.extend(collect_files_recursive(root, &p));
        } else if p.is_file()
            && let Ok(rel) = p.strip_prefix(root)
        {
            files.push(rel.to_string_lossy().to_string());
        }
    }
    files.sort();
    files
}

/// Parse a YAML inline list like `[foo, bar]` or `[foo]`.
fn parse_yaml_list(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed[1..trimmed.len() - 1]
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![trimmed.to_string()]
    }
}

