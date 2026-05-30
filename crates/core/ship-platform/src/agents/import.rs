//! Import harness-managed config (MCP servers, skills, permissions)
//! into the project's `.ship/` tree.
//!
//! Each importer is keyed by `harness_id` and reads the corresponding
//! harness's on-disk format (Claude's JSON, Codex/Cursor's TOML, etc.).
//! Project-scope files win over user-scope ones; ids that already exist
//! in the project are skipped (no overwrite). Wired up by
//! `cli-framework::core_primitives` as part of the `ship init` /
//! `ship pull` flows.
//!
//! Split (P26 follow-up): one submodule per concern. The dispatcher
//! `import_permissions_from_harness` lives with the claude/codex/cursor
//! variants in `permissions.rs` so all three share their helpers locally.

use anyhow::{Result, anyhow};
use std::path::PathBuf;

pub mod mcp;
pub mod permissions;
pub mod skills;

pub use mcp::import_from_harness;
pub use permissions::import_permissions_from_harness;
pub use skills::import_skills_from_harness;

/// Resolve the user home directory. Honours `HOME` if set, falls back
/// to `home::home_dir()`. Shared by all three importers since each
/// needs to find a global-scope config file under `~`.
pub(super) fn home() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        let path = PathBuf::from(home);
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }
    home::home_dir().ok_or_else(|| anyhow!("Cannot determine home directory"))
}
