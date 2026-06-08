//! Skill data types + frontmatter validation.

use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Result, anyhow};
use jsonschema::Validator;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical SKILL.md frontmatter schema (embedded at compile time).
///
/// Source of truth: `schemas/skill.schema.json` at the workspace root. Keeping
/// the schema string embedded means the runtime can validate every SKILL.md
/// without depending on an installed copy at runtime.
const SKILL_SCHEMA_JSON: &str =
    include_str!("../../../../../../../schemas/skill.schema.json");

fn skill_schema_validator() -> &'static Validator {
    static VALIDATOR: OnceLock<Validator> = OnceLock::new();
    VALIDATOR.get_or_init(|| {
        let schema: Value = serde_json::from_str(SKILL_SCHEMA_JSON)
            .expect("schemas/skill.schema.json must be valid JSON");
        jsonschema::validator_for(&schema)
            .expect("schemas/skill.schema.json must be a valid JSON Schema")
    })
}

/// Validate a parsed SKILL.md frontmatter (already converted from YAML to JSON)
/// against the canonical schema. On violation, returns an error that names the
/// skill directory and every offending field.
pub(super) fn validate_against_schema(value: &Value, path: &Path) -> Result<()> {
    let validator = skill_schema_validator();
    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|err| {
            let location = err.instance_path().to_string();
            let field = if location.is_empty() {
                "<root>".to_string()
            } else {
                location
            };
            format!("  - {field}: {err}")
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(
            "Invalid SKILL.md frontmatter in {}:\n{}",
            path.display(),
            errors.join("\n")
        ))
    }
}

/// Origin of a skill document.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    /// Free-form author strings. Required by the canonical schema; defaulted
    /// at create time when the caller does not supply one.
    #[serde(default)]
    pub authors: Vec<String>,
    /// Stable storage key. Survives directory renames. Defaults to `id` when
    /// absent in frontmatter.
    #[serde(default)]
    pub stable_id: Option<String>,
    /// Discovery tags. Required by the canonical schema.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Visibility. None means default-public.
    #[serde(default)]
    pub audience: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) source: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct SkillSpecFrontmatter {
    pub(super) name: String,
    #[serde(rename = "stable-id")]
    pub(super) stable_id: String,
    pub(super) description: String,
    pub(super) tags: Vec<String>,
    pub(super) authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) audience: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) supersedes: Option<String>,
    #[serde(
        default,
        rename = "superseded-by",
        skip_serializing_if = "Option::is_none"
    )]
    pub(super) superseded_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) compatibility: Option<String>,
    #[serde(
        rename = "allowed-tools",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub(super) allowed_tools: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) metadata: Option<SkillSpecMetadata>,
    /// Artifact types produced by this skill (e.g. `[html, adr]`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
