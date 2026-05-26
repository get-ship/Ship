//! Project identity: slug derivation, name/id tokens, and ship.jsonc id persistence.

use std::fs;
use std::path::{Component, Path};

use anyhow::{Context, Result, anyhow};

use super::SHIP_DIR_NAME;

/// Derive a stable, filesystem-safe project slug from a .ship path.
/// Uses "<project-name>-<project-id>" (or "tmp-..." for transient test projects).
pub fn project_slug_from_ship_dir(ship_dir: &Path) -> String {
    project_namespace_slug(ship_dir)
}

fn project_namespace_slug(ship_dir: &Path) -> String {
    let project_root = project_root_from_ship_dir(ship_dir);
    let mut identity = read_space_identity(ship_dir).unwrap_or_default();

    if identity.id.is_none()
        && let Ok(id) = ensure_space_id(ship_dir)
    {
        identity.id = Some(id);
    }

    let name_token = project_name_token(project_root, identity.name.as_deref());
    let id_token = space_id_token(ship_dir, identity.id.as_deref());
    let transient_prefix = if is_transient_project_root(project_root) {
        "tmp-"
    } else {
        ""
    };

    format!("{transient_prefix}{name_token}-{id_token}")
}

fn project_root_from_ship_dir(ship_dir: &Path) -> &Path {
    if ship_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == SHIP_DIR_NAME)
    {
        ship_dir.parent().unwrap_or(ship_dir)
    } else {
        ship_dir
    }
}

#[derive(Debug, Default)]
struct ProjectIdentity {
    id: Option<String>,
    name: Option<String>,
}

fn read_space_identity(ship_dir: &Path) -> Option<ProjectIdentity> {
    let path = ship_dir.join(crate::project::PRIMARY_CONFIG_FILE);
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value =
        serde_json::from_str(&crate::fs_util::strip_jsonc_comments(&content)).ok()?;

    let id = parsed
        .get("id")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let name = parsed
        .get("name")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Some(ProjectIdentity { id, name })
}

fn project_name_token(project_root: &Path, configured_name: Option<&str>) -> String {
    let raw = configured_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            project_root
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "project".to_string());

    let mut token = sanitize_file_name(&raw);
    if token.len() > 40 {
        token.truncate(40);
        token = token.trim_end_matches('-').to_string();
    }
    if token.is_empty() {
        "project".to_string()
    } else {
        token
    }
}

fn space_id_token(ship_dir: &Path, configured_id: Option<&str>) -> String {
    let raw = configured_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path_slug_from_ship_dir(ship_dir));

    let mut token = sanitize_file_name(&raw);
    if token.len() > 16 {
        token.truncate(16);
        token = token.trim_end_matches('-').to_string();
    }
    if token.is_empty() {
        "unknown".to_string()
    } else {
        token
    }
}

fn project_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_ascii_lowercase()),
            _ => None,
        })
        .collect()
}

fn starts_with_components(path: &Path, components: &[&str]) -> bool {
    let normalized = project_components(path);
    if normalized.len() < components.len() {
        return false;
    }
    normalized
        .iter()
        .take(components.len())
        .zip(components.iter())
        .all(|(lhs, rhs)| lhs == rhs)
}

fn contains_component_sequence(path: &Path, sequence: &[&str]) -> bool {
    let normalized = project_components(path);
    if sequence.is_empty() || normalized.len() < sequence.len() {
        return false;
    }
    normalized.windows(sequence.len()).any(|window| {
        window
            .iter()
            .zip(sequence.iter())
            .all(|(lhs, rhs)| lhs == rhs)
    })
}

pub(crate) fn is_transient_project_root(project_root: &Path) -> bool {
    let canonical = fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    starts_with_components(&canonical, &["tmp"])
        || starts_with_components(&canonical, &["private", "tmp"])
        || starts_with_components(&canonical, &["var", "folders"])
        || starts_with_components(&canonical, &["private", "var", "folders"])
        || contains_component_sequence(&canonical, &["appdata", "local", "temp"])
        || contains_component_sequence(&canonical, &["examples", "e2e"])
        || contains_component_sequence(&canonical, &["examples", "projects-e2e"])
        || contains_component_sequence(&canonical, &["target", "tmp"])
}

/// Derive a slug-form identifier from a project path. Used when no
/// explicit `id` is configured in `ship.jsonc`.
fn path_slug_from_ship_dir(ship_dir: &Path) -> String {
    let project_root = project_root_from_ship_dir(ship_dir);
    let canonical =
        std::fs::canonicalize(project_root).unwrap_or_else(|_| project_root.to_path_buf());
    let raw = canonical.to_string_lossy();
    let slug: String = raw
        .trim_start_matches('/')
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "unknown-project".to_string()
    } else {
        slug
    }
}

/// Ensure a project has a persistent id in ship.jsonc and return it.
pub fn ensure_space_id(ship_dir: &Path) -> Result<String> {
    if let Some(identity) = read_space_identity(ship_dir)
        && let Some(id) = identity.id
    {
        return Ok(id);
    }

    let path = ship_dir.join(crate::project::PRIMARY_CONFIG_FILE);
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Project at {} has no readable ship.jsonc. Re-run `ship init` to initialize it.",
            ship_dir.display()
        )
    })?;

    let mut parsed: serde_json::Value =
        serde_json::from_str(&crate::fs_util::strip_jsonc_comments(&content)).with_context(|| {
            format!(
                "Project at {} has an invalid ship.jsonc. Re-run `ship init` or fix the file.",
                ship_dir.display()
            )
        })?;

    let obj = parsed
        .as_object_mut()
        .ok_or_else(|| anyhow!("ship.jsonc must contain a top-level object."))?;

    let generated_id = crate::gen_nanoid();
    obj.insert(
        "id".to_string(),
        serde_json::Value::String(generated_id.clone()),
    );

    let updated = serde_json::to_string_pretty(&parsed)?;
    crate::fs_util::write_atomic(&path, updated)?;

    Ok(generated_id)
}

pub fn sanitize_file_name(name: &str) -> String {
    let mut sanitized = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    while sanitized.contains("--") {
        sanitized = sanitized.replace("--", "-");
    }

    sanitized = sanitized.trim_matches('-').to_string();

    if sanitized.len() > 60 {
        sanitized.truncate(60);
        sanitized = sanitized.trim_end_matches('-').to_string();
    }

    sanitized
}

pub fn get_project_name(ship_path: &Path) -> String {
    ship_path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string()
}
