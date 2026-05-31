//! `ship validate` — check .ship/ config for errors before compile.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::profile::load_agent_profile;
use ship_platform::services::compile::{load_permission_preset, load_skills, McpEntry, McpFile};
use crate::profile::find_agent_file;

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub agent: String,
    pub file: String,
    pub error: String,
}

/// Per-agent result: name + collected errors.
#[derive(Debug)]
pub struct AgentReport {
    pub agent_id: String,
    pub errors: Vec<ValidationError>,
}

// ── Public entry points ───────────────────────────────────────────────────────

/// Run validation for one or all agents. Print results. Return Err if any error found.
pub fn run_validate(agent_id: Option<&str>, json: bool, project_root: &Path) -> Result<()> {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(project_root);
    if !ship_dir.exists() {
        anyhow::bail!(".ship/ not found. Run: ship init");
    }
    let agents_dir = ship_dir.join("agents");

    let reports = if let Some(id) = agent_id {
        let agent_path = find_agent_file(id, project_root)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))?;
        vec![validate_agent(id, &agent_path, &agents_dir)]
    } else {
        validate_all(&agents_dir, project_root)
    };

    let all_errors: Vec<&ValidationError> = reports.iter().flat_map(|r| r.errors.iter()).collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&all_errors)?);
        if !all_errors.is_empty() {
            anyhow::bail!("");
        }
        return Ok(());
    }

    let mut any_errors = false;
    for report in &reports {
        if report.errors.is_empty() {
            println!("✓ agent {} — valid", report.agent_id);
        } else {
            any_errors = true;
            println!(
                "✗ agent {} — {} error{}",
                report.agent_id,
                report.errors.len(),
                if report.errors.len() == 1 { "" } else { "s" }
            );
            for e in &report.errors {
                println!("  {} — {}", e.file, e.error);
            }
        }
    }

    if any_errors {
        anyhow::bail!("validation failed");
    }
    Ok(())
}

// ── Core validation ───────────────────────────────────────────────────────────

/// Validate all agents found in agents/ (primary) and agents/profiles/ (compat).
fn validate_all(agents_dir: &Path, _project_root: &Path) -> Vec<AgentReport> {
    let mut reports = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Primary: agents/*.{jsonc,toml} (flat)
    if let Ok(entries) = std::fs::read_dir(agents_dir) {
        let mut paths: Vec<_> = entries
            .flatten()
            .filter(|e| crate::paths::is_config_ext(&e.path()) && e.path().is_file())
            // Exclude known non-agent config files
            .filter(|e| crate::paths::is_agent_config(&e.path()))
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for entry in paths {
            let path = entry.path();
            let id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            seen_ids.insert(id.clone());
            reports.push(validate_agent(&id, &path, agents_dir));
        }
    }

    // Compat: agents/profiles/*.{jsonc,toml}
    let profiles_dir = agents_dir.join("profiles");
    if profiles_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&profiles_dir)
    {
        let mut paths: Vec<_> = entries
            .flatten()
            .filter(|e| crate::paths::is_config_ext(&e.path()))
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for entry in paths {
            let path = entry.path();
            let id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            // Skip if already found in agents/
            if seen_ids.contains(&id) {
                continue;
            }
            seen_ids.insert(id.clone());
            reports.push(validate_agent(&id, &path, agents_dir));
        }
    }

    // Also check any compat presets dir
    let presets_dir = agents_dir.join("presets");
    if presets_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&presets_dir)
    {
        let mut paths: Vec<_> = entries
            .flatten()
            .filter(|e| crate::paths::is_config_ext(&e.path()))
            .collect();
        paths.sort_by_key(|e| e.file_name());
        for entry in paths {
            let path = entry.path();
            let id = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            // Skip if already found in agents/ or agents/profiles/
            if seen_ids.contains(&id) {
                continue;
            }
            reports.push(validate_agent(&id, &path, agents_dir));
        }
    }
    reports
}

/// Validate a single agent at `agent_path` against `agents_dir`.
pub fn validate_agent(agent_id: &str, agent_path: &Path, agents_dir: &Path) -> AgentReport {
    let mut errors = Vec::new();

    // 1. Parse JSONC config.
    let agent = match load_agent_profile(agent_path) {
        Ok(p) => p,
        Err(e) => {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: agent_path.display().to_string(),
                error: format!("config parse error: {e}. Check JSONC syntax"),
            });
            return AgentReport {
                agent_id: agent_id.to_string(),
                errors,
            };
        }
    };

    let agent_file = agent_path.display().to_string();

    // Canonical layout: `.ship/` is the parent of `agents/`. Skills
    // live at `.ship/skills/` (NOT under `agents/skills/`), and the
    // `project.skill_paths` setting in `ship.jsonc` can add more roots
    // (typically `docs/`, `skills/`). Use the compiler's loader so the
    // validator's notion of "skill exists" matches the loader's exactly
    // — same set of resolved IDs, same precedence rules, same flat-vs-
    // directory tolerance.
    //
    // Previously this used `agents_dir.join("skills")` and only checked
    // for `.ship/agents/skills/<id>.md` or `<id>/SKILL.md` — neither
    // location anyone actually uses — so every skill ref failed and
    // `ship validate` produced N false positives.
    let ship_dir = agents_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| agents_dir.to_path_buf());

    // 2. Skill refs resolve through the same loader the compiler uses.
    let resolved_skill_ids: std::collections::HashSet<String> = load_skills(&ship_dir)
        .map(|skills| skills.into_iter().map(|s| s.id).collect())
        .unwrap_or_default();
    for skill_id in &agent.skills.refs {
        if !resolved_skill_ids.contains(skill_id) {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: agent_file.clone(),
                error: format!(
                    "skill '{}' not found via skill_paths. Add it to .ship/skills/{}/SKILL.md, \
                     declare a `project.skill_paths` entry in ship.jsonc that contains it, or \
                     run `ship install`",
                    skill_id, skill_id,
                ),
            });
        }
    }

    // 3. MCP server refs exist in mcp config AND have required fields.
    //    Canonical location: `.ship/mcp.jsonc` (one above the `agents/`
    //    directory). Previously this used `agents_dir.join("mcp.jsonc")`,
    //    which looked at `.ship/agents/mcp.jsonc` and silently never
    //    found any servers — the validator was effectively a no-op for
    //    MCP entries written via `ship mcp add`.
    let mcp_path = ship_dir.join("mcp.jsonc");
    let mcp_file = load_mcp_file(&mcp_path);
    for server_id in &agent.mcp.servers {
        match mcp_file.servers.iter().find(|s| &s.id == server_id) {
            None => errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: "mcp.jsonc".to_string(),
                error: format!(
                    "mcp server '{}' not found in mcp.jsonc. Add it with `ship mcp add-stdio {} <command>`",
                    server_id, server_id,
                ),
            }),
            Some(s) => {
                if let Some(e) = check_mcp_entry(s) {
                    errors.push(ValidationError {
                        agent: agent_id.to_string(),
                        file: "mcp.jsonc".to_string(),
                        error: format!("mcp.servers.{} — {}", server_id, e),
                    });
                }
            }
        }
    }

    // 4. Validate all mcp.jsonc entries (regardless of agent refs)
    for server in &mcp_file.servers {
        if let Some(e) = check_mcp_entry(server) {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: "mcp.jsonc".to_string(),
                error: format!("mcp.servers.{} — {}", server.id, e),
            });
        }
    }
    // Deduplicate errors from mcp entry checks (a server might be caught twice)
    errors.dedup_by(|a, b| a.file == b.file && a.error == b.error);

    // 5. permissions.preset references a known preset
    if let Some(preset_name) = &agent.permissions.preset {
        if preset_name.trim().is_empty() {
            errors.push(ValidationError {
                agent: agent_id.to_string(),
                file: agent_file.clone(),
                error: "permissions.preset must be a non-empty string".to_string(),
            });
        } else {
            let perm_path = ship_dir.join("permissions.jsonc");
            if perm_path.exists() && load_permission_preset(&ship_dir, preset_name).is_none() {
                errors.push(ValidationError {
                    agent: agent_id.to_string(),
                    file: "permissions.jsonc".to_string(),
                    error: format!("preset '{}' not found in permissions.jsonc", preset_name),
                });
            }
        }
    }

    // 5. Isolation block (if present) parses to a valid IsolationProfile.
    // The schema is documented in ship-isolation::tier::IsolationProfile
    // and consumed by ship-platform::isolation::parse_agent_override at
    // run time. We catch malformed blocks here so users don't discover
    // them at agent-launch time.
    if let Some(detail) = check_isolation_field(agent_path) {
        errors.push(ValidationError {
            agent: agent_id.to_string(),
            file: agent_file.clone(),
            error: detail,
        });
    }

    AgentReport {
        agent_id: agent_id.to_string(),
        errors,
    }
}

/// Read the agent jsonc, look for a top-level `isolation:` block, and
/// try to parse it via `ship_platform::isolation::parse_agent_override`.
/// Returns `None` when the field is absent or parses cleanly;
/// `Some(error message)` when it's present and malformed.
fn check_isolation_field(agent_path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(agent_path).ok()?;
    let stripped = ship_substrate::fs_util::strip_jsonc_comments(&raw);
    let value: serde_json::Value = serde_json::from_str(&stripped).ok()?;
    // `isolation:` lives at the top level of the canonical agent
    // shape (sibling to `agent`, `mcp`, `skills`, `rules`).
    let iso = value.get("isolation")?;
    match ship_platform::isolation::parse_agent_override(Some(iso)) {
        Ok(_) => None,
        Err(e) => Some(format!(
            "isolation: block is malformed — {e}. \
             Expected: {{ tier?, strategy?, policy? }}. \
             Tier ids: none|process|container|vm|cloud. \
             Strategy ids: noop|harness_native|srt|docker|youki|wsl2|lima|fly."
        )),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn load_mcp_file(path: &Path) -> McpFile {
    McpFile::load(path).unwrap_or_default()
}

/// Returns an error message if the entry is missing required fields; None if valid.
fn check_mcp_entry(s: &McpEntry) -> Option<String> {
    let is_http = s.url.is_some() && s.command.is_none()
        || s.server_type.as_deref() == Some("http")
        || s.server_type.as_deref() == Some("sse");
    if is_http {
        if s.url.as_deref().is_none_or(|u| u.trim().is_empty()) {
            return Some("HTTP/SSE server missing 'url' field. Add a url to the server entry".to_string());
        }
    } else {
        // stdio
        if s.command.as_deref().is_none_or(|c| c.trim().is_empty()) {
            return Some("stdio server missing 'command' field. Add a command to the server entry".to_string());
        }
    }
    None
}
