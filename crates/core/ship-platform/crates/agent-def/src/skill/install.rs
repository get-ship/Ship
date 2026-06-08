//! Skill install plumbing — registry install + validation.

use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use super::storage::{ensure_project_skills_storage, ensure_user_skills_storage};
use super::types::{Skill, is_valid_skill_name};
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SkillInstallScope {
    Project,
    User,
}

pub(super) fn validate_skill_install_request(source: &str, skill_id: &str) -> Result<()> {
    let source = source.trim();
    let skill_id = skill_id.trim();
    if source.is_empty() {
        return Err(anyhow!("source cannot be empty"));
    }
    if source.contains('\n') || source.contains('\r') {
        return Err(anyhow!("source must be a single-line value"));
    }
    if source.contains(char::is_whitespace) {
        return Err(anyhow!(
            "Unsupported source '{}'. Provide a skill ID (e.g. owner/repo@skill-id).",
            source
        ));
    }
    if skill_id.is_empty() {
        // Allow owner/repo@skill-id shorthand — skill ID embedded in source.
        if !source.contains('@') {
            return Err(anyhow!(
                "Skill ID is required when source does not embed one (owner/repo@skill-id)."
            ));
        }
    } else if !is_valid_skill_name(skill_id) {
        return Err(anyhow!(
            "Invalid skill id '{}'. Skill IDs must be kebab-case and <= 64 chars.",
            skill_id
        ));
    }
    if source.contains("://")
        || source.contains("github.com")
        || source.starts_with("git@")
        || source.ends_with(".git")
    {
        return Err(anyhow!(
            "Git URL sources are not supported. Use a skill ID (e.g. owner/repo@skill-id)."
        ));
    }
    Ok(())
}

pub(super) fn parse_skills_install_source(source: &str) -> Result<(String, Option<String>)> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("source cannot be empty"));
    }

    // Shorthand: owner/repo@skill-id
    if let Some((repo, skill)) = trimmed.split_once('@') {
        let repo = repo.trim();
        let skill = skill.trim();
        if repo.contains('/') && !skill.is_empty() {
            return Ok((repo.to_string(), Some(skill.to_string())));
        }
    }

    Ok((trimmed.to_string(), None))
}

#[cfg(test)]
pub(super) fn find_skill_source_dir(search_root: &Path, skill_id: &str) -> Result<PathBuf> {
    let mut discovered = Vec::<(String, PathBuf)>::new();

    for entry in walkdir::WalkDir::new(search_root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_name() != "SKILL.md" {
            continue;
        }
        let file_type = entry.file_type();
        if !file_type.is_file() && !file_type.is_symlink() {
            continue;
        }
        if file_type.is_symlink() && !entry.path().is_file() {
            continue;
        }
        let Some(parent) = entry.path().parent() else {
            continue;
        };
        let Some(dir_name) = parent.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        discovered.push((dir_name.to_string(), parent.to_path_buf()));
    }

    discovered.sort_by(|left, right| left.0.cmp(&right.0));
    discovered.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);

    let mut available = discovered
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    available.sort();
    available.dedup();

    if discovered.is_empty() {
        let list = if available.is_empty() {
            "(no skills found)".to_string()
        } else {
            available.join(", ")
        };
        return Err(anyhow!(
            "Skill '{}' not found under {}. Available IDs: {}",
            skill_id,
            search_root.display(),
            list
        ));
    }

    let requested = skill_id.trim();
    if !requested.is_empty() {
        if let Some((_, path)) = discovered.iter().find(|(id, _)| id == requested) {
            return Ok(path.clone());
        }
        if discovered.len() == 1 {
            return Ok(discovered[0].1.clone());
        }
        return Err(anyhow!(
            "Skill '{}' was not found in installer output. Available IDs: {}",
            requested,
            available.join(", ")
        ));
    }

    if discovered.len() == 1 {
        return Ok(discovered[0].1.clone());
    }

    Err(anyhow!(
        "Multiple skills were produced. Provide a skill ID. Available IDs: {}",
        available.join(", ")
    ))
}

fn install_skill_from_source_into_dir(
    _dest_root: &Path,
    source: &str,
    skill_id: &str,
    _force: bool,
) -> Result<Skill> {
    validate_skill_install_request(source, skill_id)?;

    let (package_source, _parsed_skill) = parse_skills_install_source(source)?;

    // Registry download is not yet implemented. The skills.sh/npx path has
    // been removed (no Node in production). Wire up registry fetch here.
    Err(anyhow!(
        "Skill registry install is not yet implemented. \
         Copy the skill directory into .ship/skills/ or skills/ manually. \
         Source: '{package_source}'."
    ))
}

pub fn install_skill_from_source(
    project_dir: Option<&Path>,
    source: &str,
    skill_id: &str,
    _git_ref: Option<&str>,
    _repo_path: Option<&str>,
    scope: SkillInstallScope,
    force: bool,
) -> Result<Skill> {
    match scope {
        SkillInstallScope::User => {
            let dest_root = ensure_user_skills_storage()?;
            install_skill_from_source_into_dir(&dest_root, source, skill_id, force)
        }
        SkillInstallScope::Project => {
            let project_dir =
                project_dir.ok_or_else(|| anyhow!("Project scope requires project_dir"))?;
            let dest_root = ensure_project_skills_storage(project_dir)?;
            let installed =
                install_skill_from_source_into_dir(&dest_root, source, skill_id, force)?;

            let mut config = ship_config::project::get_config(Some(project_dir.to_path_buf()))?;
            if !config.agent.skills.contains(&installed.id) {
                config.agent.skills.push(installed.id.clone());
                ship_config::project::save_config(&config, Some(project_dir.to_path_buf()))?;
            }
            Ok(installed)
        }
    }
}
