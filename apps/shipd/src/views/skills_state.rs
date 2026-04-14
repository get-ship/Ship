//! Skill discovery helpers for the `skills` daemon-rendered view.
//!
//! Uses `runtime::read_skill_paths` to enumerate all configured skill
//! roots (respects `project.skill_paths`, so `docs/` doc-skills appear
//! alongside project skills when listed in the manifest).

use serde::Serialize;
use std::path::Path;

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

fn parse_skill_md(content: &str) -> (std::collections::HashMap<String, String>, String) {
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

fn parse_yaml_list(val: &str) -> Vec<String> {
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
    if skill_dir_path
        .components()
        .any(|c| c.as_os_str() == "docs")
    {
        SkillKind::Docs
    } else {
        SkillKind::Smart
    }
}

fn source_label(skill_dir_path: &Path, project_root: &Path) -> String {
    if skill_dir_path.starts_with(project_root) {
        "PROJECT SKILLS".to_string()
    } else {
        "USER SKILLS".to_string()
    }
}

/// List all skills discovered via `runtime::read_skill_paths`.
pub fn list_skills(ship_dir: &Path) -> Vec<SkillSummary> {
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let dirs = runtime::read_skill_paths(ship_dir, project_root);
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for skills_root in &dirs {
        if !skills_root.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(skills_root) else { continue };
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
            if !seen.insert(id.clone()) {
                continue;
            }
            let description = fm.get("description").cloned().unwrap_or_default();
            let tags = fm.get("tags").map(|v| parse_yaml_list(v)).unwrap_or_default();
            out.push(SkillSummary {
                id,
                name: fm.get("name").cloned().unwrap_or(dir_name),
                description,
                kind: kind_from_path(&path),
                source: source_label(&path, project_root),
                tags,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Load a single skill by id (matches frontmatter `name` then dir name).
pub fn load_skill(ship_dir: &Path, id: &str) -> Option<SkillDetail> {
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let dirs = runtime::read_skill_paths(ship_dir, project_root);
    for skills_root in &dirs {
        let Ok(entries) = std::fs::read_dir(skills_root) else { continue };
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
            let authors = fm
                .get("authors")
                .map(|v| parse_yaml_list(v))
                .unwrap_or_default();
            let artifacts = fm
                .get("artifacts")
                .map(|v| parse_yaml_list(v))
                .unwrap_or_default();
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
                source: source_label(&path, project_root),
                tags,
                authors,
                variables,
                artifacts,
                body,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_skill(dir: &Path, name: &str, fm: &str, body: &str) {
        let d = dir.join(name);
        std::fs::create_dir_all(&d).unwrap();
        let content = format!("---\n{fm}---\n{body}");
        std::fs::write(d.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn empty_returns_empty() {
        let tmp = tempdir().unwrap();
        let ship = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship).unwrap();
        assert!(list_skills(&ship).is_empty());
    }

    #[test]
    fn parses_frontmatter_into_summary() {
        let tmp = tempdir().unwrap();
        let ship = tmp.path().join(".ship");
        std::fs::create_dir_all(ship.join("skills")).unwrap();
        write_skill(
            &ship.join("skills"),
            "tdd",
            "name: tdd\ndescription: Test first.\ntags: [testing, rust]\n",
            "# Body\n",
        );
        let out = list_skills(&ship);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id, "tdd");
        assert_eq!(out[0].source, "PROJECT SKILLS");
        assert!(out[0].tags.contains(&"testing".to_string()));
    }

    #[test]
    fn load_skill_reads_body_and_vars() {
        let tmp = tempdir().unwrap();
        let ship = tmp.path().join(".ship");
        std::fs::create_dir_all(ship.join("skills").join("x").join("assets")).unwrap();
        write_skill(
            &ship.join("skills"),
            "x",
            "name: x\ndescription: d\n",
            "# hi\nbody",
        );
        std::fs::write(
            ship.join("skills").join("x").join("assets").join("vars.json"),
            r#"{"foo": {"type":"string"}, "bar": {"type":"bool"}}"#,
        )
        .unwrap();
        let d = load_skill(&ship, "x").unwrap();
        assert!(d.body.contains("# hi"));
        assert_eq!(d.variables, vec!["bar".to_string(), "foo".to_string()]);
    }

    #[test]
    fn load_skill_returns_none_for_missing() {
        let tmp = tempdir().unwrap();
        let ship = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship).unwrap();
        assert!(load_skill(&ship, "nope").is_none());
    }
}
