//! Load skills from all directories declared in `project.skill_paths`.
//! Always scans `.ship/skills/` (primary); other dirs resolve relative to the
//! project root (parent of `.ship/`).

use anyhow::Result;
use compiler::{Skill, SkillSource};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub fn load_skills(ship_dir: &Path) -> Result<Vec<Skill>> {
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let skill_dirs = runtime::read_skill_paths(ship_dir, project_root);
    let primary = skill_dirs.first().cloned();

    let mut skills = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for skills_path in &skill_dirs {
        if !skills_path.exists() {
            continue;
        }
        let is_primary = primary.as_ref().is_some_and(|p| p == skills_path);
        for entry in std::fs::read_dir(skills_path)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if !skill_md.exists() {
                    continue;
                }
                let id = entry.file_name().to_string_lossy().to_string();
                if !seen_ids.insert(id.clone()) {
                    continue;
                }
                let raw = std::fs::read_to_string(&skill_md)?;
                let mut skill = parse_skill(&id, &raw);
                let vars_path = path.join("assets").join("vars.json");
                if vars_path.exists() {
                    let state_key = skill.stable_id.as_deref().unwrap_or(id.as_str());
                    match crate::vars::load_vars_json(&vars_path) {
                        Ok(var_defs) => {
                            let state = runtime::skill_vars::get_skill_vars(ship_dir, state_key)
                                .unwrap_or_default()
                                .unwrap_or_default();
                            crate::vars::warn_invalid_enum_vars(state_key, &var_defs, &state);
                            skill.vars = state;
                        }
                        Err(e) => {
                            eprintln!("warning: skill '{}': failed to read vars.json: {}", id, e);
                        }
                    }
                }
                skills.push(skill);
            } else if is_primary && path.extension().is_some_and(|x| x == "md") {
                // Flat `<id>.md` only accepted in the primary `.ship/skills/` dir —
                // loose `.md` files elsewhere (docs/, etc.) collide with non-skill content.
                let id = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if !seen_ids.insert(id.clone()) {
                    continue;
                }
                let raw = std::fs::read_to_string(&path)?;
                skills.push(parse_skill(&id, &raw));
            }
        }
    }
    Ok(skills)
}

/// Parse a SKILL.md file per the agentskills.io spec.
///
/// Frontmatter keys: `name`, `stable-id`, `description`, `license`, `compatibility`,
/// `allowed-tools` (space-delimited), `artifacts`, `metadata` block. Legacy top-level
/// `version`/`author` fold into `metadata`.
pub fn parse_skill(id: &str, raw: &str) -> Skill {
    let mut name = id.to_string();
    let mut stable_id: Option<String> = None;
    let mut description = None;
    let mut license = None;
    let mut compatibility = None;
    let mut allowed_tools = vec![];
    let mut artifacts = vec![];
    let mut metadata: HashMap<String, String> = HashMap::new();
    let mut content_start = 0usize;

    if let Some(rest) = raw.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let fm = &rest[..end];
        let mut in_metadata = false;
        for line in fm.lines() {
            if in_metadata {
                if line.starts_with("  ") || line.starts_with('\t') {
                    let trimmed = line.trim();
                    if let Some((k, v)) = trimmed.split_once(':') {
                        metadata.insert(k.trim().to_string(), v.trim().to_string());
                    }
                    continue;
                } else {
                    in_metadata = false;
                }
            }
            if let Some(v) = line.strip_prefix("name:") {
                name = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("stable-id:") {
                let sid = v.trim().to_string();
                if crate::vars::state::validate_skill_id(&sid).is_ok() {
                    stable_id = Some(sid);
                } else {
                    eprintln!("warning: skill '{}': invalid stable-id '{}' ignored", id, sid);
                }
            } else if let Some(v) = line.strip_prefix("description:") {
                description = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("license:") {
                license = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("compatibility:") {
                compatibility = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("allowed-tools:") {
                allowed_tools = v.split_whitespace().map(str::to_string).collect();
            } else if let Some(v) = line.strip_prefix("artifacts:") {
                let trimmed = v.trim().trim_start_matches('[').trim_end_matches(']');
                artifacts = trimmed
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
            } else if line.trim_end() == "metadata:" {
                in_metadata = true;
            } else if let Some(v) = line.strip_prefix("version:") {
                metadata.insert("version".to_string(), v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("author:") {
                metadata.insert("author".to_string(), v.trim().to_string());
            }
        }
        content_start = 4 + end + 5; // "---\n" + fm + "\n---\n"
    }

    let normalized_name = name.to_lowercase().replace(' ', "-");
    if normalized_name != id && name != id {
        eprintln!(
            "warning: skill '{}': frontmatter name '{}' does not match directory name",
            id, name
        );
    }

    let content = raw[content_start..].trim().to_string();
    Skill {
        id: id.to_string(),
        name,
        stable_id,
        description,
        license,
        compatibility,
        allowed_tools,
        artifacts,
        metadata,
        content,
        source: SkillSource::default(),
        vars: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str, content: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    fn skill_md(id: &str, audience: Option<&str>) -> String {
        let extra = audience
            .map(|a| format!("audience: {}\n", a))
            .unwrap_or_default();
        format!(
            "---\nname: {id}\nstable-id: {id}\ndescription: Test skill {id}.\n{extra}---\n\nbody of {id}\n"
        )
    }

    /// Hypothesis #1 (validated): compiler only walks `.ship/skills/`.
    /// Fix: a skill at `<project_root>/docs/<id>/SKILL.md` now resolves when
    /// `project.skill_paths` includes `docs/`.
    #[test]
    fn resolves_docs_skill_via_skill_paths() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        let ship = project.join(".ship");
        fs::create_dir_all(&ship).unwrap();
        write(
            &ship,
            "ship.jsonc",
            r#"{"id":"t","project":{"skill_paths":["docs/"]}}"#,
        );
        write(project, "docs/ship-runtime/SKILL.md", &skill_md("ship-runtime", None));

        let skills = load_skills(&ship).unwrap();
        assert!(
            skills.iter().any(|s| s.id == "ship-runtime"),
            "expected ship-runtime to resolve; got: {:?}",
            skills.iter().map(|s| &s.id).collect::<Vec<_>>()
        );
    }

    /// Hypothesis #2 (refuted): there is no audience filter. Skills with
    /// `audience: internal` must resolve.
    #[test]
    fn resolves_skill_with_audience_internal() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        let ship = project.join(".ship");
        fs::create_dir_all(&ship).unwrap();
        write(
            &ship,
            "ship.jsonc",
            r#"{"id":"t","project":{"skill_paths":["docs/"]}}"#,
        );
        write(
            project,
            "docs/ship-architecture/SKILL.md",
            &skill_md("ship-architecture", Some("internal")),
        );

        let skills = load_skills(&ship).unwrap();
        let found = skills.iter().find(|s| s.id == "ship-architecture");
        assert!(found.is_some(), "audience: internal must not hide a skill");
    }

    /// Scanning multiple custom paths concatenates results; both skills resolve
    /// simultaneously. Simulates the real repro (`skill_paths: ["docs/","skills/"]`).
    #[test]
    fn resolves_across_multiple_skill_paths() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        let ship = project.join(".ship");
        fs::create_dir_all(&ship).unwrap();
        write(
            &ship,
            "ship.jsonc",
            r#"{"id":"t","project":{"skill_paths":["docs/","skills/"]}}"#,
        );
        write(project, "docs/ship-runtime/SKILL.md", &skill_md("ship-runtime", None));
        write(
            project,
            "docs/ship-coordination/SKILL.md",
            &skill_md("ship-coordination", Some("internal")),
        );
        write(&ship, "skills/tdd/SKILL.md", &skill_md("tdd", None));

        let skills = load_skills(&ship).unwrap();
        let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"ship-runtime"), "got {:?}", ids);
        assert!(ids.contains(&"ship-coordination"), "got {:?}", ids);
        assert!(ids.contains(&"tdd"), "got {:?}", ids);
    }

    /// Non-existent skill IDs remain absent — guards the `--with` error message.
    /// `apps/ship-cli/src/compile/mod.rs` emits "skill not found: X — check
    /// skill_paths in ship.jsonc" when the requested ID isn't in the loaded library.
    #[test]
    fn missing_skill_absent_from_library() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        let ship = project.join(".ship");
        fs::create_dir_all(&ship).unwrap();
        write(
            &ship,
            "ship.jsonc",
            r#"{"id":"t","project":{"skill_paths":["docs/"]}}"#,
        );
        write(project, "docs/ship-runtime/SKILL.md", &skill_md("ship-runtime", None));

        let skills = load_skills(&ship).unwrap();
        assert!(!skills.iter().any(|s| s.id == "does-not-exist"));
    }

    /// Loose `.md` files in a non-primary path must NOT be loaded as skills —
    /// `docs/README.md` would otherwise become a bogus "readme" skill.
    #[test]
    fn loose_md_in_custom_path_ignored() {
        let tmp = TempDir::new().unwrap();
        let project = tmp.path();
        let ship = project.join(".ship");
        fs::create_dir_all(&ship).unwrap();
        write(
            &ship,
            "ship.jsonc",
            r#"{"id":"t","project":{"skill_paths":["docs/"]}}"#,
        );
        write(project, "docs/README.md", "# not a skill\n");
        write(project, "docs/real-skill/SKILL.md", &skill_md("real-skill", None));

        let skills = load_skills(&ship).unwrap();
        let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"real-skill"));
        assert!(!ids.contains(&"README"), "got {:?}", ids);
    }
}
