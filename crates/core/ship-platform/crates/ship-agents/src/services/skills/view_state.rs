//! Skill discovery for the `skills` view. Scans global + project scopes,
//! deduplicates by id (project wins). Callers pass the daemon's global
//! ship_dir; project scope resolved via `ship_substrate::project::get_stack_root`.

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
    pub files: Vec<String>,
    pub has_vars: bool,
    pub has_refs: bool,
    pub has_evals: bool,
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
    pub files: Vec<String>,
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

pub(crate) fn scan_files(skill_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut sorted: Vec<_> = entries.flatten().collect();
        sorted.sort_by_key(|e| e.file_name());
        for e in sorted {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            let rel = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
            if e.path().is_dir() { walk(&e.path(), &rel, out); } else { out.push(rel); }
        }
    }
    walk(skill_dir, "", &mut out);
    out
}

fn kind_from_path(skill_dir_path: &Path) -> SkillKind {
    if skill_dir_path.components().any(|c| c.as_os_str() == "docs") {
        SkillKind::Docs
    } else {
        SkillKind::Smart
    }
}

pub(crate) struct SkillScopes {
    pub global_dirs: Vec<PathBuf>,
    pub project_dirs: Vec<PathBuf>,
}

fn resolve_scopes(ship_dir: &Path) -> SkillScopes {
    let project_root = ship_substrate::project::get_stack_root(None)
        .ok()
        .filter(|p| p != ship_dir);
    scopes_from(ship_dir, project_root.as_deref())
}

pub(crate) fn scopes_from(
    ship_dir: &Path,
    project_ship_dir: Option<&Path>,
) -> SkillScopes {
    // LEGACY-COMPETITOR nuance (pending the catalog fold): the global tier
    // still derives parent-of-home — the shape this scanner always had and
    // its tests encode. The catalog work replaces this scanner wholesale.
    let space_root = ship_dir.parent().unwrap_or(ship_dir);
    let global_dirs = crate::agent_domain::skill_paths::read_skill_paths(space_root);

    let project_dirs = match project_ship_dir {
        Some(project_root) if project_root != ship_dir => {
            // Marker cutover: the resolved value IS the stack root.
            crate::agent_domain::skill_paths::read_skill_paths(project_root)
        }
        _ => Vec::new(),
    };

    SkillScopes { global_dirs, project_dirs }
}

pub(crate) fn scan_skill_dir(skills_root: &Path, source: &str) -> Vec<SkillSummary> {
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
        let files = scan_files(&path);
        let has_vars = files.iter().any(|f| f.ends_with("vars.json"));
        let has_refs = files.iter().any(|f| f.starts_with("references/"));
        let has_evals = files.iter().any(|f| f.starts_with("evals/"));
        out.push(SkillSummary {
            id,
            name: fm.get("name").cloned().unwrap_or(dir_name),
            description,
            kind: kind_from_path(&path),
            source: source.to_string(),
            tags,
            files,
            has_vars,
            has_refs,
            has_evals,
        });
    }
    out
}

fn find_skill_subdir(skills_root: &Path, id: &str) -> Option<(PathBuf, String)> {
    if !skills_root.exists() { return None; }
    let entries = std::fs::read_dir(skills_root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").is_file() { continue; }
        let content = std::fs::read_to_string(path.join("SKILL.md")).unwrap_or_default();
        let (fm, _) = parse_skill_md(&content);
        let dn = entry.file_name().to_string_lossy().to_string();
        let candidate = fm.get("name").cloned().unwrap_or_else(|| dn.clone());
        if candidate == id || dn == id { return Some((path, dn)); }
    }
    None
}

fn load_detail_from_dir(skills_root: &Path, id: &str, source: &str) -> Option<SkillDetail> {
    let (path, dir_name) = find_skill_subdir(skills_root, id)?;
    let content = std::fs::read_to_string(path.join("SKILL.md")).unwrap_or_default();
    let (fm, body) = parse_skill_md(&content);
    let candidate = fm.get("name").cloned().unwrap_or_else(|| dir_name.clone());
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
    let files = scan_files(&path);
    Some(SkillDetail {
        id: candidate.clone(),
        name: fm.get("name").cloned().unwrap_or(dir_name),
        description: fm.get("description").cloned().unwrap_or_default(),
        kind: kind_from_path(&path),
        source: source.to_string(),
        tags, authors, variables, artifacts, files, body,
    })
}

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

pub fn resolve_skill_dir(ship_dir: &Path, id: &str) -> Option<PathBuf> {
    resolve_skill_dir_with_scopes(&resolve_scopes(ship_dir), id)
}

pub(crate) fn resolve_skill_dir_with_scopes(scopes: &SkillScopes, id: &str) -> Option<PathBuf> {
    for dir in scopes.project_dirs.iter().chain(scopes.global_dirs.iter()) {
        if let Some((path, _)) = find_skill_subdir(dir, id) {
            return Some(path);
        }
    }
    None
}

/// Load a file from a skill directory. Path-traversal safe.
pub fn load_skill_file(ship_dir: &Path, id: &str, file_path: &str) -> Option<String> {
    let skill_dir = resolve_skill_dir(ship_dir, id)?;
    let full = skill_dir.join(file_path);
    let canonical = full.canonicalize().ok()?;
    let base = skill_dir.canonicalize().ok()?;
    if !canonical.starts_with(&base) {
        return None; // path traversal attempt
    }
    std::fs::read_to_string(&canonical).ok()
}

#[cfg(test)]
#[path = "view_state_tests.rs"]
mod tests;
