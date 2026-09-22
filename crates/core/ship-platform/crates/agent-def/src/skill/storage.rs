//! Filesystem layout + SKILL.md read/write.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use super::types::{
    Skill, SkillSource, SkillSpecFrontmatter, validate_against_schema,
    validate_skill_spec_frontmatter,
};

pub(super) fn skills_dir(project_dir: &Path) -> PathBuf {
    ship_substrate::project::skills_dir(project_dir)
}

pub(super) fn user_skills_dir() -> PathBuf {
    ship_substrate::project::user_skills_dir()
}

pub(super) fn ensure_project_skills_storage(project_dir: &Path) -> Result<PathBuf> {
    let target = skills_dir(project_dir);
    fs::create_dir_all(&target)?;
    Ok(target)
}

pub(super) fn ensure_user_skills_storage() -> Result<PathBuf> {
    let target = user_skills_dir();
    fs::create_dir_all(&target)?;
    Ok(target)
}

pub(super) fn skill_dir(project_dir: &Path, id: &str) -> PathBuf {
    skills_dir(project_dir).join(id)
}

pub(super) fn user_skill_dir(id: &str) -> Result<PathBuf> {
    Ok(user_skills_dir().join(id))
}

pub(super) fn parse_skill_spec(dir: &Path) -> Result<Skill> {
    let path = dir.join("SKILL.md");
    if !path.exists() {
        return Err(anyhow!("Missing SKILL.md in {}", dir.display()));
    }

    let raw = fs::read_to_string(&path)?.replace("\r\n", "\n");
    if !raw.starts_with("---\n") {
        return Err(anyhow!(
            "Invalid SKILL.md frontmatter in {}",
            path.display()
        ));
    }
    let rest = &raw[4..];
    let end = rest.find("\n---").ok_or_else(|| {
        anyhow!(
            "Invalid SKILL.md: missing closing frontmatter in {}",
            path.display()
        )
    })?;
    let yaml = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n').to_string();

    // Round-trip the YAML through serde_json::Value so we can apply the
    // canonical JSON Schema. This is the load-time contract enforcement: any
    // unknown field, missing required field, or wrong-shape value is rejected
    // here with the skill directory and offending field path in the message.
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml)
        .with_context(|| format!("Failed to parse SKILL.md frontmatter in {}", path.display()))?;
    let json_value: serde_json::Value = serde_json::to_value(&yaml_value).with_context(|| {
        format!(
            "Failed to convert SKILL.md frontmatter to JSON in {}",
            path.display()
        )
    })?;
    validate_against_schema(&json_value, &path)?;

    let fm: SkillSpecFrontmatter = serde_yaml::from_value(yaml_value)
        .with_context(|| format!("Failed to parse SKILL.md frontmatter in {}", path.display()))?;
    validate_skill_spec_frontmatter(&fm, &path)?;

    let id = dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid skill directory name: {}", dir.display()))?
        .to_string();

    if fm.name != id {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: frontmatter name '{}' must match directory id '{}'",
            path.display(),
            fm.name,
            id
        ));
    }

    let name = fm
        .metadata
        .as_ref()
        .and_then(|meta| meta.display_name.clone())
        .unwrap_or_else(|| id.clone());
    let source = fm
        .metadata
        .as_ref()
        .and_then(|meta| meta.source.as_deref())
        .map(|source| match source {
            "builtin" => SkillSource::Builtin,
            "ai-generated" => SkillSource::AiGenerated,
            "community" => SkillSource::Community,
            "imported" => SkillSource::Imported,
            _ => SkillSource::Custom,
        })
        .unwrap_or(SkillSource::Custom);

    Ok(Skill {
        id,
        name,
        description: Some(fm.description),
        version: fm.version,
        authors: fm.authors,
        stable_id: Some(fm.stable_id),
        tags: fm.tags,
        audience: fm.audience,
        content: body,
        source,
        artifacts: fm.artifacts.unwrap_or_default(),
    })
}

pub(super) fn list_skills_from_dir(dir: &Path) -> Result<Vec<Skill>> {
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            match parse_skill_spec(&path) {
                Ok(s) => skills.push(s),
                Err(e) => tracing::warn!(path = %path.display(), error = %e, "skipping skill spec"),
            }
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}
