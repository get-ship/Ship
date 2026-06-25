//! Namespace path helpers. All document paths are derived from these — never
//! construct paths with raw string joins outside of these helpers.

use std::path::{Path, PathBuf};

use super::global::get_global_dir;

/// `.ship/project/` — specs, features, releases, notes, ADRs
pub fn project_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("project")
}

/// `.ship/agents/` — agent profile `.jsonc` files only.
/// Rules, skills, permissions, MCP config, and teams live at the `.ship/` root.
pub fn agents_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("agents")
}

pub fn teams_dir(ship_dir: &Path) -> PathBuf {
    ship_dir.join("teams")
}

pub fn adrs_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("adrs")
}

pub fn releases_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("releases")
}

/// `.ship/project/releases/upcoming/` — planned/active release plans.
pub fn upcoming_releases_dir(ship_dir: &Path) -> PathBuf {
    releases_dir(ship_dir).join("upcoming")
}

pub fn notes_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("notes")
}

pub fn features_dir(ship_dir: &Path) -> PathBuf {
    project_ns(ship_dir).join("features")
}

pub fn modes_dir(ship_dir: &Path) -> PathBuf {
    agents_ns(ship_dir).join("modes")
}

pub fn skills_dir(ship_dir: &Path) -> PathBuf {
    project_skills_dir(ship_dir)
}

pub fn rules_dir(ship_dir: &Path) -> PathBuf {
    ship_dir.join("rules")
}

pub fn mcp_config_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("mcp.jsonc")
}

pub fn permissions_config_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("permissions.jsonc")
}

/// `.ship/vision.md` — project north-star document
pub fn vision_doc_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("vision.md")
}

/// Legacy path for vision template (`.ship/TEMPLATE.md`).
/// New projects should rely on `.ship/vision.md` as the canonical document.
pub fn vision_template_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("TEMPLATE.md")
}

/// Global/shared skills store: `~/.ship/library/skills/`
pub fn user_skills_dir() -> PathBuf {
    get_global_dir()
        .unwrap_or_else(|_| PathBuf::from(".ship"))
        .join("library")
        .join("skills")
}

/// Project-scoped skills store: `.ship/skills/`
pub fn project_skills_dir(ship_dir: &Path) -> PathBuf {
    ship_dir.join("skills")
}

