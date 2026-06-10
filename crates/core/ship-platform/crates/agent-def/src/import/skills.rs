//! Import harness-managed skills (Claude `~/.claude/skills`,
//! Codex `~/.agents/skills`) into the project's
//! `skills/<id>/SKILL.md`. Frontmatter is YAML; we tolerate either a
//! top-level `name:` or a `metadata.display_name:` entry. Skills that
//! already exist by id are skipped (no overwrite).

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::super::harnesses::{HarnessDescriptor, SkillsOutput, require_harness};
use super::home;

pub fn import_skills_from_harness(harness_id: &str, project_dir: PathBuf) -> Result<usize> {
    let desc = require_harness(harness_id)?;
    let import_paths = harness_skill_import_paths(desc, &project_dir)?;
    if import_paths.is_empty() {
        return Ok(0);
    }

    let mut existing_ids: HashSet<String> = crate::skill::list_skills(&project_dir)?
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut added = 0usize;

    for skills_dir in import_paths {
        if !skills_dir.exists() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&skills_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let skill_dir = entry.path();
            if !skill_dir.is_dir() {
                continue;
            }
            let fallback_id = entry.file_name().to_string_lossy().to_string();
            let skill_md = skill_dir.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }

            let draft = match parse_imported_skill_file(&skill_md, &fallback_id) {
                Ok(Some(draft)) => draft,
                Ok(None) => continue,
                Err(error) => {
                    tracing::warn!(
                        path = %skill_md.display(),
                        %error,
                        "skipped invalid skill import"
                    );
                    continue;
                }
            };

            if existing_ids.contains(&draft.id) {
                continue;
            }

            if crate::skill::create_skill(&project_dir, &draft.id, &draft.name, &draft.content)
                .is_ok()
            {
                existing_ids.insert(draft.id);
                added += 1;
            }
        }
    }

    Ok(added)
}

fn harness_skill_import_paths(
    desc: &HarnessDescriptor,
    project_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let Some(relative_path) = harness_skills_relative_path(desc) else {
        return Ok(Vec::new());
    };

    let project_path_opt = project_dir.parent().map(|root| root.join(relative_path));
    let global_path = home()?.join(relative_path);
    if let Some(project_path) = project_path_opt.as_ref()
        && project_path.exists()
    {
        return Ok(vec![project_path.clone()]);
    }

    if global_path.exists() {
        return Ok(vec![global_path]);
    }

    Ok(project_path_opt.into_iter().collect())
}

fn harness_skills_relative_path(desc: &HarnessDescriptor) -> Option<&'static str> {
    match desc.skills_output {
        SkillsOutput::ClaudeSkills => Some(".claude/skills"),
        SkillsOutput::CodexSkills => Some(".agents/skills"),
        SkillsOutput::None => None,
    }
}

#[derive(Debug)]
struct ImportedSkillDraft {
    id: String,
    name: String,
    content: String,
}

#[derive(Deserialize, Default)]
struct ImportedSkillMetadata {
    #[serde(default)]
    display_name: Option<String>,
}

#[derive(Deserialize, Default)]
struct ImportedSkillFrontmatter {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    metadata: Option<ImportedSkillMetadata>,
}

fn parse_imported_skill_file(path: &Path, fallback_id: &str) -> Result<Option<ImportedSkillDraft>> {
    let raw = fs::read_to_string(path)?;
    let normalized = raw.replace("\r\n", "\n");
    let (frontmatter, body) = split_optional_frontmatter(&normalized);

    let id_seed = frontmatter
        .as_ref()
        .and_then(|fm| fm.name.as_deref())
        .unwrap_or(fallback_id);
    let Some(id) = normalize_skill_id(id_seed) else {
        return Ok(None);
    };

    let name = frontmatter
        .and_then(|fm| fm.metadata.and_then(|meta| meta.display_name))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| id.clone());

    let content = strip_managed_skill_header(body).trim().to_string();
    if content.is_empty() {
        return Ok(None);
    }

    Ok(Some(ImportedSkillDraft { id, name, content }))
}

fn split_optional_frontmatter(content: &str) -> (Option<ImportedSkillFrontmatter>, &str) {
    if !content.starts_with("---\n") {
        return (None, content);
    }
    let rest = &content[4..];
    let Some(end) = rest.find("\n---") else {
        return (None, content);
    };
    let frontmatter_raw = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n');
    let parsed = serde_yaml::from_str::<ImportedSkillFrontmatter>(frontmatter_raw).ok();
    (parsed, body)
}

fn strip_managed_skill_header(content: &str) -> &str {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("<!--") {
        return content;
    }
    let Some(end_idx) = trimmed.find("-->") else {
        return content;
    };
    let tail = &trimmed[end_idx + 3..];
    tail.trim_start_matches('\n')
}

fn normalize_skill_id(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in raw.trim().to_ascii_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            prev_dash = false;
            continue;
        }
        if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }

    let normalized = out.trim_matches('-').to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}
