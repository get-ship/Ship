//! Skill install plumbing — skills.sh CLI wrapper + legacy migration + validation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;

use super::storage::{
    ensure_project_skills_storage, ensure_user_skills_storage, parse_skill,
};
use super::types::{Skill, is_valid_skill_name};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum SkillInstallScope {
    Project,
    User,
}

pub(super) fn looks_like_skills_cli_command(raw: &str) -> bool {
    let tokens = raw
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return false;
    }
    let starts_like_command = matches!(
        tokens.first().map(String::as_str),
        Some("skills") | Some("skills.sh") | Some("npx")
    );
    if !starts_like_command {
        return false;
    }
    let has_add = tokens.iter().any(|token| token == "add");
    let has_skills = tokens.iter().any(|token| {
        token == "skills"
            || token == "skills.sh"
            || token == "@skills/cli"
            || token.ends_with("/skills")
    });
    has_add && has_skills
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
    let is_skills_command =
        source.contains(char::is_whitespace) && looks_like_skills_cli_command(source);
    if skill_id.is_empty() {
        if !is_skills_command {
            return Err(anyhow!(
                "Skill ID is required when source is not a skills.sh command."
            ));
        }
    } else if !is_valid_skill_name(skill_id) {
        return Err(anyhow!(
            "Invalid skill id '{}'. Skill IDs must be kebab-case and <= 64 chars.",
            skill_id
        ));
    }
    if source.contains(char::is_whitespace) && !looks_like_skills_cli_command(source) {
        return Err(anyhow!(
            "Unsupported source '{}'. Provide either a skills.sh command (`npx skills add ...`) or a skill ID.",
            source
        ));
    }
    if !source.contains(char::is_whitespace)
        && (source.contains("://")
            || source.contains("github.com")
            || source.starts_with("git@")
            || source.ends_with(".git"))
    {
        return Err(anyhow!(
            "Git URL sources are not supported. Use a skills.sh command or a skill ID."
        ));
    }
    Ok(())
}

pub(super) fn format_command_failure(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return stdout;
    }
    "command exited with no diagnostic output".to_string()
}

pub(super) fn parse_skills_install_source(source: &str) -> Result<(String, Option<String>)> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("source cannot be empty"));
    }

    // Shorthand: owner/repo@skill-id
    if !trimmed.contains(char::is_whitespace)
        && let Some((repo, skill)) = trimmed.split_once('@')
    {
        let repo = repo.trim();
        let skill = skill.trim();
        if repo.contains('/') && !skill.is_empty() {
            return Ok((repo.to_string(), Some(skill.to_string())));
        }
    }

    if !looks_like_skills_cli_command(trimmed) {
        return Ok((trimmed.to_string(), None));
    }

    let tokens = trimmed.split_whitespace().collect::<Vec<_>>();
    let add_index = tokens
        .iter()
        .position(|token| token.eq_ignore_ascii_case("add"))
        .ok_or_else(|| anyhow!("skills.sh command is missing `add`"))?;

    let mut package: Option<String> = None;
    let mut parsed_skill: Option<String> = None;
    let mut index = add_index + 1;
    while index < tokens.len() {
        let token = tokens[index];
        let lower = token.to_ascii_lowercase();

        if lower == "--skill" || lower == "-s" {
            if let Some(next) = tokens.get(index + 1)
                && !next.starts_with('-')
            {
                parsed_skill = Some((*next).to_string());
            }
            index += 2;
            continue;
        }

        if let Some(value) = lower.strip_prefix("--skill=")
            && !value.is_empty()
        {
            parsed_skill = Some(value.to_string());
            index += 1;
            continue;
        }

        if token.starts_with('-') {
            // Known flags that take a value.
            let consumes_value = matches!(lower.as_str(), "--agent" | "-a");
            if consumes_value
                && let Some(next) = tokens.get(index + 1)
                && !next.starts_with('-')
            {
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }

        if package.is_none() {
            package = Some(token.to_string());
        }
        index += 1;
    }

    let package = package.ok_or_else(|| anyhow!("skills.sh command is missing package source"))?;
    Ok((package, parsed_skill))
}

fn run_skills_install_command(
    source: &str,
    skill_id: &str,
    workspace_dir: &Path,
    home_dir: &Path,
) -> Result<()> {
    let (package_source, parsed_skill) = parse_skills_install_source(source)?;

    // Normalize to a deterministic, non-interactive install:
    // - `--yes`: no prompts
    // - `--copy`: avoid symlink installs
    // - `--agent codex`: install only the codex/.agents surface
    let mut command = ProcessCommand::new("npx");
    command
        .arg("-y")
        .arg("skills")
        .arg("add")
        .arg(&package_source)
        .arg("--yes")
        .arg("--copy")
        .arg("--agent")
        .arg("codex");
    let requested_skill = if !skill_id.trim().is_empty() {
        Some(skill_id.trim().to_string())
    } else {
        parsed_skill
    };
    if let Some(skill) = requested_skill
        && !skill.trim().is_empty()
    {
        command.arg("--skill").arg(skill.trim());
    }

    command.current_dir(workspace_dir);
    command.env("HOME", home_dir);
    command.env("USERPROFILE", home_dir);
    command.env("XDG_CONFIG_HOME", home_dir.join(".config"));
    command.env("XDG_CACHE_HOME", home_dir.join(".cache"));

    let output = match command.output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(anyhow!(
                "skills.sh install requires Node.js (`npx`) on PATH."
            ));
        }
        Err(err) => return Err(err).with_context(|| "Failed to run `skills.sh` installer command"),
    };
    if !output.status.success() {
        return Err(anyhow!(
            "Failed to install skill via skills.sh CLI: {}",
            format_command_failure(&output)
        ));
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = dst.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

pub(super) fn migrate_legacy_project_skills(
    project_dir: &Path,
    target_root: &Path,
) -> Result<()> {
    let mut legacy_roots = vec![ship_substrate::project::legacy_agents_skills_dir(project_dir)];
    legacy_roots.extend(ship_substrate::project::legacy_project_skills_dir_candidates(
        project_dir,
    ));

    for legacy_root in legacy_roots {
        if legacy_root == target_root || !legacy_root.exists() || !legacy_root.is_dir() {
            continue;
        }

        for entry in fs::read_dir(&legacy_root)? {
            let entry = entry?;
            let source_path = entry.path();
            if !source_path.is_dir() {
                continue;
            }
            if !source_path.join("SKILL.md").is_file() {
                continue;
            }

            let destination_path = target_root.join(entry.file_name());
            if destination_path.exists() {
                continue;
            }

            match fs::rename(&source_path, &destination_path) {
                Ok(_) => {}
                Err(_) => {
                    copy_dir_recursive(&source_path, &destination_path)?;
                    fs::remove_dir_all(&source_path)?;
                }
            }
        }
    }

    Ok(())
}

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
        // skills.sh may install SKILL.md as symlinks under agent directories.
        // Accept symlinked files as long as they resolve to a readable file path.
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
    dest_root: &Path,
    source: &str,
    skill_id: &str,
    force: bool,
) -> Result<Skill> {
    validate_skill_install_request(source, skill_id)?;

    let tmp_root = std::env::temp_dir().join(format!("ship-skill-install-{}", ship_substrate::gen_nanoid()));
    fs::create_dir_all(&tmp_root)?;
    struct CleanupGuard(PathBuf);
    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = CleanupGuard(tmp_root.clone());

    let install_workspace = tmp_root.join("workspace");
    let install_home = tmp_root.join("home");
    fs::create_dir_all(&install_workspace)?;
    fs::create_dir_all(&install_home)?;

    run_skills_install_command(source, skill_id, &install_workspace, &install_home)?;

    let source_dir = find_skill_source_dir(&tmp_root, skill_id)?;
    let source_skill = parse_skill(&source_dir)?;

    fs::create_dir_all(dest_root)?;
    let destination_dir = dest_root.join(&source_skill.id);
    if destination_dir.exists() {
        if !force {
            return Err(anyhow!(
                "Skill '{}' already exists at {} (use --force to overwrite)",
                source_skill.id,
                destination_dir.display()
            ));
        }
        fs::remove_dir_all(&destination_dir)?;
    }

    copy_dir_recursive(&source_dir, &destination_dir)?;
    parse_skill(&destination_dir)
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

            let mut config = crate::config::get_config(Some(project_dir.to_path_buf()))?;
            if !config.agent.skills.contains(&installed.id) {
                config.agent.skills.push(installed.id.clone());
                crate::config::save_config(&config, Some(project_dir.to_path_buf()))?;
            }
            Ok(installed)
        }
    }
}
