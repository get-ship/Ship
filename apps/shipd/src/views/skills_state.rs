//! Skill discovery helpers for the `skills` daemon-rendered view.
//!
//! Scans two scopes and merges them (project wins on id collisions):
//!
//! 1. Global (`~/.ship/skills/` and any `skill_paths` declared in the global
//!    manifest, though there rarely is one).
//! 2. Project (the enclosing `.ship/` discovered from cwd, plus its
//!    `project.skill_paths` — typically `docs/` for doc-skills and `skills/`
//!    for auxiliary smart skills).
//!
//! The callers pass the daemon's global ship_dir — we resolve the project
//! scope via `runtime::project::get_project_dir(None)` so the second scope
//! is additive and only materializes when the daemon is actually running
//! inside a Ship project.

use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillKind {
    Smart,
    Docs,
}

#[derive(Clone, Debug, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: SkillKind,
    pub source: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SkillDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: SkillKind,
    pub source: String,
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub variables: Vec<String>,
    pub artifacts: Vec<String>,
    /// Raw SKILL.md body (no frontmatter).
    pub body: String,
}

/// Source label applied to project-scoped skills (enclosing `.ship/` of cwd).
pub const SOURCE_PROJECT: &str = "PROJECT SKILLS";
/// Source label applied to global-scoped skills (`~/.ship/skills/`).
pub const SOURCE_GLOBAL: &str = "USER SKILLS";

pub(crate) fn parse_skill_md(content: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut map = std::collections::HashMap::new();
    if !content.starts_with("---") {
        return (map, content.to_string());
    }
    let rest = &content[3..];
    let Some(end) = rest.find("---") else { return (map, content.to_string()) };
    let fm = &rest[..end];
    let body = rest[end + 3..].trim_start_matches('\n').to_string();
    for line in fm.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
        }
    }
    (map, body)
}

pub(crate) fn parse_yaml_list(val: &str) -> Vec<String> {
    let t = val.trim();
    if t.starts_with('[') && t.ends_with(']') {
        t[1..t.len() - 1]
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else if t.is_empty() {
        Vec::new()
    } else {
        vec![t.to_string()]
    }
}

fn kind_from_path(skill_dir_path: &Path) -> SkillKind {
    if skill_dir_path.components().any(|c| c.as_os_str() == "docs") {
        SkillKind::Docs
    } else {
        SkillKind::Smart
    }
}

/// Resolved skill search roots for a (global, project) pair. The project
/// list is empty when the daemon is not running inside a Ship project.
pub(crate) struct SkillScopes {
    pub global_dirs: Vec<PathBuf>,
    pub project_dirs: Vec<PathBuf>,
}

fn resolve_scopes(ship_dir: &Path) -> SkillScopes {
    let project_ship = runtime::project::get_project_dir(None)
        .ok()
        .filter(|p| p != ship_dir);
    scopes_from(ship_dir, project_ship.as_deref())
}

/// Build scopes for an explicit (global, project) pair. Tests call this to
/// inject both sides without touching the process-global `SHIP_DIR` env var.
pub(crate) fn scopes_from(
    ship_dir: &Path,
    project_ship_dir: Option<&Path>,
) -> SkillScopes {
    let global_parent = ship_dir.parent().unwrap_or(ship_dir);
    let global_dirs = runtime::read_skill_paths(ship_dir, global_parent);

    let project_dirs = match project_ship_dir {
        Some(project_ship) if project_ship != ship_dir => {
            let project_root = project_ship.parent().unwrap_or(project_ship);
            runtime::read_skill_paths(project_ship, project_root)
        }
        _ => Vec::new(),
    };

    SkillScopes { global_dirs, project_dirs }
}

/// Enumerate SKILL.md-bearing subdirectories of `skills_root` and produce
/// summaries tagged with `source`. Non-directories, missing SKILL.md, and
/// unreadable entries are skipped silently.
fn scan_skill_dir(skills_root: &Path, source: &str) -> Vec<SkillSummary> {
    let mut out = Vec::new();
    if !skills_root.exists() {
        return out;
    }
    let Ok(entries) = std::fs::read_dir(skills_root) else { return out };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
        let (fm, _body) = parse_skill_md(&content);
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let id = fm.get("name").cloned().unwrap_or_else(|| dir_name.clone());
        let description = fm.get("description").cloned().unwrap_or_default();
        let tags = fm.get("tags").map(|v| parse_yaml_list(v)).unwrap_or_default();
        out.push(SkillSummary {
            id,
            name: fm.get("name").cloned().unwrap_or(dir_name),
            description,
            kind: kind_from_path(&path),
            source: source.to_string(),
            tags,
        });
    }
    out
}

fn load_detail_from_dir(
    skills_root: &Path,
    id: &str,
    source: &str,
) -> Option<SkillDetail> {
    if !skills_root.exists() {
        return None;
    }
    let entries = std::fs::read_dir(skills_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
        let (fm, body) = parse_skill_md(&content);
        let dir_name = entry.file_name().to_string_lossy().to_string();
        let candidate = fm.get("name").cloned().unwrap_or_else(|| dir_name.clone());
        if candidate != id && dir_name != id {
            continue;
        }
        let tags = fm.get("tags").map(|v| parse_yaml_list(v)).unwrap_or_default();
        let authors = fm.get("authors").map(|v| parse_yaml_list(v)).unwrap_or_default();
        let artifacts = fm.get("artifacts").map(|v| parse_yaml_list(v)).unwrap_or_default();
        let mut variables = Vec::new();
        if let Ok(vs) = std::fs::read_to_string(path.join("assets").join("vars.json"))
            && let Ok(v) = serde_json::from_str::<serde_json::Value>(&vs)
            && let Some(map) = v.as_object()
        {
            variables = map.keys().cloned().collect();
            variables.sort();
        }
        return Some(SkillDetail {
            id: candidate.clone(),
            name: fm.get("name").cloned().unwrap_or(dir_name),
            description: fm.get("description").cloned().unwrap_or_default(),
            kind: kind_from_path(&path),
            source: source.to_string(),
            tags,
            authors,
            variables,
            artifacts,
            body,
        });
    }
    None
}

/// List all skills discovered across global and (when present) project scopes.
/// Deduplicates by id, with project entries overriding global on collision.
pub fn list_skills(ship_dir: &Path) -> Vec<SkillSummary> {
    list_skills_with_scopes(&resolve_scopes(ship_dir))
}

pub(crate) fn list_skills_with_scopes(scopes: &SkillScopes) -> Vec<SkillSummary> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::new();

    // Project first so its skills win dedup on id collision.
    for dir in &scopes.project_dirs {
        for skill in scan_skill_dir(dir, SOURCE_PROJECT) {
            if seen.insert(skill.id.clone()) {
                out.push(skill);
            }
        }
    }
    for dir in &scopes.global_dirs {
        for skill in scan_skill_dir(dir, SOURCE_GLOBAL) {
            if seen.insert(skill.id.clone()) {
                out.push(skill);
            }
        }
    }

    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load a single skill by id. Project scope wins over global on conflict.
pub fn load_skill(ship_dir: &Path, id: &str) -> Option<SkillDetail> {
    load_skill_with_scopes(&resolve_scopes(ship_dir), id)
}

pub(crate) fn load_skill_with_scopes(
    scopes: &SkillScopes,
    id: &str,
) -> Option<SkillDetail> {
    for dir in &scopes.project_dirs {
        if let Some(d) = load_detail_from_dir(dir, id, SOURCE_PROJECT) {
            return Some(d);
        }
    }
    for dir in &scopes.global_dirs {
        if let Some(d) = load_detail_from_dir(dir, id, SOURCE_GLOBAL) {
            return Some(d);
        }
    }
    None
}

#[cfg(test)]
#[path = "skills_state_tests.rs"]
mod tests;
