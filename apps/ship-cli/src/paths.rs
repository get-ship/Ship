use ship_substrate::ShipHome;
use ship_substrate::home::Vector;
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
    home().vector_dir(ship_substrate::home::Scope::User, Vector::Skills, None)
}
/// User-scope MCP config file: `~/.ship/library/mcp.jsonc`.
pub fn global_cache_dir() -> PathBuf {
    home().machine_dir().join("cache")
}

pub fn ensure_global_dirs() -> anyhow::Result<()> {
    for dir in [global_dir(), global_skills_dir(), global_cache_dir()] {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

// ── Project (.ship/) ──────────────────────────────────────────────────────────

pub fn project_dir() -> PathBuf {
    PathBuf::from(ShipHome::ship_dir_name())
}
pub fn agents_dir() -> PathBuf {
    ShipHome::from_env().vector_dir(ship_substrate::home::Scope::Project, Vector::Agents, None)
}
pub fn rules_dir() -> PathBuf {
    ShipHome::from_env().vector_dir(ship_substrate::home::Scope::Project, Vector::Rules, None)
}
pub fn skills_dir() -> PathBuf {
    ShipHome::from_env().vector_dir(ship_substrate::home::Scope::Project, Vector::Skills, None)
}
pub fn views_dir() -> PathBuf {
    project_dir().join("views")
}
/// Project-scope MCP config file: `<project>/.ship/mcp.jsonc`.
///
/// MCP is intentionally a file at every scope (per ADR cleanup; see
/// `crates/core/runtime/src/home/vector.rs`). The directory-symmetry
/// the original migrator wanted was overengineering.
pub fn mcp_path() -> PathBuf {
    project_dir().join("mcp.jsonc")
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
    ship_substrate::project::get_project_dir(None)
}

pub fn ensure_project_dirs() -> anyhow::Result<()> {
    for dir in [
        project_dir(),
        agents_dir(),
        rules_dir(),
        skills_dir(),
        views_dir(),
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
        // Other tests in this binary mutate SHIP_HOME to a tempdir that does
        // not end in `.ship`. Pin the env to a deterministic value for the
        // duration of this assertion (the guard restores on drop).
        let tmp = tempfile::tempdir().unwrap();
        let ship_home = tmp.path().join(".ship");
        let _guard = crate::test_env::isolated_ship_home(&ship_home);
        let g = global_dir();
        assert!(g.to_string_lossy().ends_with(".ship"));
        // Per home-layout ADR, user-scope vectors live under `library/`.
        assert_eq!(global_skills_dir(), g.join("library").join("skills"));
    }

    #[test]
    fn project_paths_are_relative() {
        assert_eq!(project_dir(), PathBuf::from(".ship"));
        assert_eq!(agents_dir(), PathBuf::from(".ship/agents"));
        assert_eq!(views_dir(), PathBuf::from(".ship/views"));
        assert_eq!(mcp_path(), PathBuf::from(".ship/mcp.jsonc"));
    }
}
