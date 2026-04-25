use runtime::{ShipHome, Vector};
use std::fs;
use std::path::PathBuf;

// ── Global (~/.ship/) ─────────────────────────────────────────────────────────

fn home() -> ShipHome {
    ShipHome::from_env()
}

pub fn global_dir() -> PathBuf {
    home().root().to_path_buf()
}

pub fn global_skills_dir() -> PathBuf {
    home().vector_dir(runtime::Scope::User, Vector::Skills, None)
}
pub fn global_mcp_dir() -> PathBuf {
    home().vector_dir(runtime::Scope::User, Vector::Mcp, None)
}
pub fn global_cache_dir() -> PathBuf {
    home().machine_dir().join("cache")
}

pub fn ensure_global_dirs() -> anyhow::Result<()> {
    for dir in [
        global_dir(),
        global_skills_dir(),
        global_mcp_dir(),
        global_cache_dir(),
    ] {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

// ── Project (.ship/) ──────────────────────────────────────────────────────────

pub fn project_dir() -> PathBuf {
    PathBuf::from(ShipHome::ship_dir_name())
}
pub fn agents_dir() -> PathBuf {
    ShipHome::from_env().vector_dir(runtime::Scope::Project, Vector::Agents, None)
}
pub fn rules_dir() -> PathBuf {
    ShipHome::from_env().vector_dir(runtime::Scope::Project, Vector::Rules, None)
}
pub fn skills_dir() -> PathBuf {
    ShipHome::from_env().vector_dir(runtime::Scope::Project, Vector::Skills, None)
}
pub fn mcp_path() -> PathBuf {
    // Project-scope MCP is still a single file (not yet promoted to a
    // directory like the user library). Resolved through the typed
    // accessor so the literal lives in one place.
    ShipHome::from_env()
        .vector_dir(runtime::Scope::Project, Vector::Mcp, None)
        .with_extension("jsonc")
}
pub fn project_ship_toml() -> PathBuf {
    project_dir().join("ship.toml")
}
pub fn project_ship_jsonc() -> PathBuf {
    project_dir().join("ship.jsonc")
}

/// Returns true if the path has a .jsonc or .json extension.
pub fn is_config_ext(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|x| x == "toml" || x == "jsonc" || x == "json")
}

/// Returns true if the path has a .jsonc or .json extension (not .toml).
pub fn is_jsonc_ext(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|x| x == "jsonc" || x == "json")
}

/// Returns true if the path is an agent config file (not mcp.* or permissions.*).
pub fn is_agent_config(path: &std::path::Path) -> bool {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    stem != "mcp" && stem != "permissions"
}

/// Returns the absolute path to `.ship/` for the current project, or errors.
/// Uses git-worktree-aware traversal so this works from subdirs and worktrees.
pub fn project_ship_dir_required() -> anyhow::Result<std::path::PathBuf> {
    runtime::project::get_project_dir(None)
}

pub fn ensure_project_dirs() -> anyhow::Result<()> {
    for dir in [
        project_dir(),
        agents_dir(),
        rules_dir(),
        skills_dir(),
    ] {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_paths_under_home_ship() {
        let g = global_dir();
        assert!(g.to_string_lossy().ends_with(".ship"));
        assert_eq!(global_skills_dir(), g.join("skills"));
    }

    #[test]
    fn project_paths_are_relative() {
        assert_eq!(project_dir(), PathBuf::from(".ship"));
        assert_eq!(agents_dir(), PathBuf::from(".ship/agents"));
        assert_eq!(mcp_path(), PathBuf::from(".ship/mcp.jsonc"));
    }
}
