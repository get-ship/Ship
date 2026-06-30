//! Parse SKILL.md frontmatter per the agentskills.io spec, and load skills
//! from all directories declared in `project.skill_paths`.
//!
//! This is the compile-pipeline skill loader. The parser itself is the
//! CANONICAL one in `compiler::skill_parser` (single source for both the
//! compile path here and the dep-skill cache in `ship-store` — see #106
//! for the inline-form regression the duplicate parsers caused). Re-exported
//! here so existing `agent_def::compile::skill_parser::parse_skill` paths keep
//! working without churn. The platform's own `agents::skill::storage::parse_skill_spec`
//! is a separate, stricter parser used by the skill management service
//! (install/list).

use anyhow::Result;
use compiler::{Skill, SkillSource};
use include_dir::include_dir;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// SMELL: the `include_dir!` macro below pointed at the in-tree
/// `skills/sdk/` path that was deleted in this cleanup. The right
/// answer is to depend on a `ship-sdk-skills` crate ship-contracts
/// publishes and use its `include_dir!` macro. Until that ship-side
/// dependency is wired, the include is disabled and the consumers
/// below return empty. Re-enable when ship-contracts publishes.
// static SDK_SKILLS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../../../../skills/sdk");

pub fn embedded_capability_skills() -> Vec<Skill> {
    // Disabled: the include_dir! above is commented out. Re-enable when
    // ship-contracts publishes a ship-sdk-skills crate. Until then,
    // no embedded capability skills; the compile pool is disk-only.
    Vec::new()
}


/// The non-`SKILL.md` files inside an embedded sdk-skill directory, as
/// `(path-relative-to-skill-dir, content)` — e.g.
/// `("references/docs/git.create.mdoc", "...")`. Empty for ids that aren't
/// embedded sdk-skills. The compiler writes `SKILL.md` itself (it's templated);
/// these are the static pages its TOC links to, delivered beside it so the
/// links resolve.
pub fn embedded_skill_reference_files(_id: &str) -> Vec<(String, String)> {
    // Disabled: see embedded_capability_skills above. Re-enable when
    // ship-contracts publishes ship-sdk-skills.
    Vec::new()
}

#[allow(dead_code)]
fn collect_embedded_files_unused() {}

/// The compile-time skill **pool**: a project's on-disk skills plus the embedded
/// SDK capability-skills (a project id wins on conflict, so a project can shadow
/// a generated skill). Distinct from [`load_skills`], which is disk-only and
/// drives the skills-listing UI — capability-skills belong in the compile pool,
/// not the user's authored-skill list.
pub fn load_compile_skills(ship_dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = load_skills(ship_dir)?;
    let have: HashSet<String> = skills.iter().map(|s| s.id.clone()).collect();
    for cap in embedded_capability_skills() {
        if !have.contains(&cap.id) {
            skills.push(cap);
        }
    }
    Ok(skills)
}

// ── Skill parser ─────────────────────────────────────────────────────────────
//
// Re-export the CANONICAL `compiler::skill_parser::parse_skill` so callers
// in this crate and its tests keep using `agent_def::compile::skill_parser::parse_skill`
// without churn. The agent-def specific helpers below stay local — they own
// the embedded SDK skills + the disk loader for compile.
pub use compiler::skill_parser::parse_skill;

// ── Skill loader ─────────────────────────────────────────────────────────────

/// Load skills from all directories declared in `project.skill_paths`.
/// Always scans `.ship/skills/` (primary); other dirs resolve relative to the
/// project root (parent of `.ship/`).
pub fn load_skills(ship_dir: &Path) -> Result<Vec<Skill>> {
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let skill_dirs = crate::skill_paths::read_skill_paths(ship_dir, project_root);
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
                    match load_skill_vars(ship_dir, state_key, &vars_path) {
                        Ok(state) => skill.vars = state,
                        Err(e) => {
                            eprintln!("warning: skill '{}': failed to read vars.json: {}", id, e);
                        }
                    }
                }
                skills.push(skill);
            } else if is_primary && path.extension().is_some_and(|x| x == "md") {
                // Flat `<id>.md` only accepted in the primary `.ship/skills/` dir.
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

// ── Vars loading ─────────────────────────────────────────────────────────────

fn load_skill_vars(
    ship_dir: &Path,
    state_key: &str,
    vars_path: &Path,
) -> Result<HashMap<String, serde_json::Value>> {
    let state = crate::skill_vars::get_skill_vars(ship_dir, state_key)
        .unwrap_or_default()
        .unwrap_or_default();

    // Emit enum validation warnings for out-of-range values.
    if let Ok(content) = std::fs::read_to_string(vars_path) {
        if let Ok(defs) =
            serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content)
        {
            for (name, def) in &defs {
                if name == "$schema" {
                    continue;
                }
                if def.get("type").and_then(|v| v.as_str()) == Some("enum") {
                    if let (Some(values), Some(current)) =
                        (def.get("values").and_then(|v| v.as_array()), state.get(name))
                    {
                        if let Some(s) = current.as_str() {
                            let allowed: Vec<&str> =
                                values.iter().filter_map(|v| v.as_str()).collect();
                            if !allowed.contains(&s) {
                                eprintln!(
                                    "warning: skill '{}': var '{}' has value '{}' \
                                     which is not in allowed values: {}",
                                    state_key,
                                    name,
                                    s,
                                    allowed.join(", ")
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(state)
}

#[cfg(test)]
mod sdk_skill_tests {
    use super::*;

    #[test]
    #[ignore = "embedded SDK skills disabled until ship-contracts publishes a ship-sdk-skills crate; see embedded_capability_skills() and the commented include_dir!"]
    fn embedded_capability_skills_present_and_parsed() {
        let skills = embedded_capability_skills();
        assert!(
            !skills.is_empty(),
            "sdk capability-skills must be embedded into the binary"
        );
        let pipelines = skills
            .iter()
            .find(|s| s.id == "ship-pipelines")
            .expect("ship-pipelines capability-skill embedded");
        assert!(
            pipelines.content.contains("ship ops call"),
            "capability-skill teaches the ops CLI"
        );
        assert_eq!(pipelines.source, SkillSource::Builtin);
    }

    #[test]
    #[ignore = "embedded SDK skills disabled until ship-contracts publishes a ship-sdk-skills crate; see embedded_skill_reference_files()"]
    fn embedded_skill_reference_pages_are_available() {
        let refs = embedded_skill_reference_files("ship-pipelines");
        assert!(!refs.is_empty(), "ship-pipelines ships per-op reference pages");
        assert!(
            refs.iter().all(|(p, _)| p != "SKILL.md"),
            "SKILL.md is delivered by the compiler, not as a reference file"
        );
        assert!(
            refs.iter().any(|(p, _)| p.starts_with("references/docs/") && p.ends_with(".mdoc")),
            "reference pages live under references/docs/*.mdoc"
        );
        // Unknown ids (authored skills) have no embedded references.
        assert!(embedded_skill_reference_files("not-an-sdk-skill").is_empty());
    }

    #[test]
    #[ignore = "embedded SDK skills disabled until ship-contracts publishes a ship-sdk-skills crate; the compile pool is disk-only until then"]
    fn compile_pool_includes_embedded_for_a_clean_project() {
        // A project with no skills on disk still gets the capability-skills in
        // its compile pool — the bundling that makes `ship ops` work outside the
        // dev checkout.
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();

        let pool = load_compile_skills(&ship_dir).unwrap();
        assert!(
            pool.iter().any(|s| s.id == "ship-pipelines"),
            "embedded capability-skills must be in the compile pool for a clean project"
        );
    }
}
