//! Load skills from all directories declared in `project.skill_paths`.
//! Always scans `.ship/skills/` (primary); other dirs resolve relative to the
//! project root (parent of `.ship/`).

use anyhow::Result;
use compiler::Skill;
use std::collections::HashSet;
use std::path::Path;

pub use crate::skill_parse::parse_skill;

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
