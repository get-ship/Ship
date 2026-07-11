//! Namespace path helpers. All document paths are derived from these — never
//! construct paths with raw string joins outside of these helpers.

use std::path::{Path, PathBuf};

use super::global::get_global_dir;

/// `.ship/agents/` — agent profile `.jsonc` files only.
/// Rules, skills, permissions, MCP config, and teams live at the `.ship/` root.
pub fn agents_ns(ship_dir: &Path) -> PathBuf {
    ship_dir.join("agents")
}

pub fn mcp_config_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join("mcp.jsonc")
}

pub fn skills_dir(ship_dir: &Path) -> PathBuf {
    project_skills_dir(ship_dir)
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
