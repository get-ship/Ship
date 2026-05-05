//! Agent and skill config endpoints — serve .ship/agents/ and .ship/skills/ data.

use axum::{Json, extract::{Path, State}, http::StatusCode};

use crate::rest_api::{ApiState, MeshResponse};

use super::{err_response, ok_response, resolve_worktree};

/// Strip JSONC comments (// and /* */) and trailing commas to produce valid JSON.
fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            out.push(c);
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }

        if c == '/' {
            match chars.peek() {
                Some(&'/') => {
                    // Line comment — skip to end of line
                    for rest in chars.by_ref() {
                        if rest == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some(&'*') => {
                    chars.next();
                    in_block_comment = true;
                    continue;
                }
                _ => {}
            }
        }

        out.push(c);
    }

    // Strip trailing commas before } or ] (JSONC allows them, JSON does not)
    strip_trailing_commas(&out)
}

fn strip_trailing_commas(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            // Look ahead past whitespace for } or ]
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                // Skip the comma, keep the whitespace
                i += 1;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// GET /api/v1/projects/{id}/agents
#[utoipa::path(
    operation_id = "listAgents",
    get,
    path = "/api/v1/projects/{id}/agents",
    params(("id" = String, Path, description = "Workspace id")),
    responses(
        (status = 200, description = "Agents discovered in .ship/agents/", body = MeshResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Workspace not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn list_agents(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
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

        let json_str = strip_jsonc_comments(&raw);
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
            "providers": agent_obj.get("providers").cloned().unwrap_or(serde_json::json!([])),
        }));
    }

    Ok(ok_response(serde_json::json!({ "agents": agents })))
}

/// GET /api/v1/projects/{id}/skills
#[utoipa::path(
    operation_id = "listSkills",
    get,
    path = "/api/v1/projects/{id}/skills",
    params(("id" = String, Path, description = "Workspace id")),
    responses(
        (status = 200, description = "Skills discovered in .ship/skills/", body = MeshResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "Workspace not found", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    tag = "project"
)]
pub async fn list_skills(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<MeshResponse>), (StatusCode, Json<MeshResponse>)> {
    let worktree = resolve_worktree(state.ship_dir(), &id)?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(&worktree);

    // Use read_skill_paths to discover all skill directories (respects skill_paths config)
    let skill_dirs = ship_platform::agents::skill_paths::read_skill_paths(&ship_dir, &worktree);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_line_comments() {
        let input = r#"{
    // This is a comment
    "name": "test"
}"#;
        let result = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "test");
    }

    #[test]
    fn strip_block_comments() {
        let input = r#"{
    /* block comment */
    "name": "test"
}"#;
        let result = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], "test");
    }

    #[test]
    fn preserve_strings_with_slashes() {
        let input = r#"{"url": "https://example.com"}"#;
        let result = strip_jsonc_comments(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["url"], "https://example.com");
    }
}
