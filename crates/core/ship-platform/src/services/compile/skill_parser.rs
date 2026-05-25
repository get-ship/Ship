//! Parse SKILL.md frontmatter per the agentskills.io spec, and load skills
//! from all directories declared in `project.skill_paths`.
//!
//! This is the compile-pipeline skill parser. It produces `compiler::Skill`
//! values for use by `resolve_library` + `compile`. The platform's own
//! `agents::skill::storage::parse_skill_spec` is a separate, stricter parser
//! used by the skill management service (install/list).

use anyhow::Result;
use compiler::{Skill, SkillSource};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ── Skill parser ─────────────────────────────────────────────────────────────

/// Parse a SKILL.md file per the agentskills.io spec.
///
/// Frontmatter keys: `name`, `stable-id`, `description`, `license`,
/// `compatibility`, `allowed-tools` (space-delimited), `artifacts`,
/// `grants`, `requires`, `metadata` block. Legacy top-level `version`/`author`
/// fold into `metadata`.
pub fn parse_skill(id: &str, raw: &str) -> Skill {
    let mut name = id.to_string();
    let mut stable_id: Option<String> = None;
    let mut description = None;
    let mut license = None;
    let mut compatibility = None;
    let mut allowed_tools = vec![];
    let mut artifacts = vec![];
    let mut grants = vec![];
    let mut requires = vec![];
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
                if compiler::types::is_valid_skill_name(&sid) {
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
            } else if let Some(v) = line.strip_prefix("grants:") {
                let trimmed = v.trim().trim_start_matches('[').trim_end_matches(']');
                grants = trimmed
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
            } else if let Some(v) = line.strip_prefix("requires:") {
                let trimmed = v.trim().trim_start_matches('[').trim_end_matches(']');
                requires = trimmed
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
        grants,
        requires,
        metadata,
        content,
        source: SkillSource::default(),
        vars: Default::default(),
    }
}

// ── Skill loader ─────────────────────────────────────────────────────────────

/// Load skills from all directories declared in `project.skill_paths`.
/// Always scans `.ship/skills/` (primary); other dirs resolve relative to the
/// project root (parent of `.ship/`).
pub fn load_skills(ship_dir: &Path) -> Result<Vec<Skill>> {
    let project_root = ship_dir.parent().unwrap_or(ship_dir);
    let skill_dirs = crate::agents::skill_paths::read_skill_paths(ship_dir, project_root);
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
    let state = crate::agents::skill_vars::get_skill_vars(ship_dir, state_key)
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
