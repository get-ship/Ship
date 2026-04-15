//! Skill data types + frontmatter validation.

use std::path::Path;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Origin of a skill document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Type, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSource {
    #[default]
    Custom,
    Builtin,
    AiGenerated,
    Community,
    Imported,
}

/// A callable slash command / skill (→ `.claude/commands/<id>.md`).
/// Skills are the canonical instruction primitive in Ship.
/// They can be invoked explicitly by the user with `/skill-name [args]`
/// and can use `$ARGUMENTS`.
/// Stored as:
/// - project scope: `.ship/agents/skills/<id>/SKILL.md`
/// - user scope: `~/.ship/skills/<id>/SKILL.md`
///
/// Uses Agent Skills spec format.
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct Skill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    /// The command template. Use `$ARGUMENTS` as a placeholder for user input.
    pub content: String,
    #[serde(default)]
    pub source: SkillSource,
    /// Artifact types this skill produces (e.g. `["html", "adr"]`).
    /// Used by the runtime to infer event subscriptions — see `events_for_artifact`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct SkillSpecMetadata {
    #[serde(default)]
    pub(super) display_name: Option<String>,
    #[serde(default)]
    pub(super) source: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct SkillSpecFrontmatter {
    pub(super) name: String,
    pub(super) description: String,
    #[serde(default)]
    pub(super) compatibility: Option<String>,
    #[serde(rename = "allowed-tools", default)]
    pub(super) allowed_tools: Option<Vec<String>>,
    #[serde(default)]
    pub(super) license: Option<String>,
    #[serde(default)]
    pub(super) metadata: Option<SkillSpecMetadata>,
    /// Artifact types produced by this skill (e.g. `[html, adr]`).
    #[serde(default)]
    pub(super) artifacts: Option<Vec<String>>,
}

pub(super) fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

pub(super) fn validate_skill_spec_frontmatter(
    fm: &SkillSpecFrontmatter,
    path: &Path,
) -> Result<()> {
    if !is_valid_skill_name(&fm.name) {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: name '{}' must match ^[a-z0-9]+(-[a-z0-9]+)*$ and be <= 64 chars",
            path.display(),
            fm.name
        ));
    }

    let desc_len = fm.description.trim().chars().count();
    if desc_len == 0 || desc_len > 1024 {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: description must be 1..=1024 chars",
            path.display()
        ));
    }

    if let Some(compatibility) = &fm.compatibility
        && compatibility.chars().count() > 500
    {
        return Err(anyhow!(
            "Invalid SKILL.md in {}: compatibility must be <= 500 chars",
            path.display()
        ));
    }

    Ok(())
}
