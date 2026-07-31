//! Test helpers for agent-def.
//!
//! Local copy of the L1 project scaffold (same shape as
//! `ship_platform::testutil::init_project`). Kept in-tree so agent-def
//! tests do not re-export through the platform facade.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Create a `.ship/` tree for L1 tests: an empty config scaffold plus the
/// subdirectories that L1 functions (skills, rules, agents) expect.
pub fn init_project(base_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let base_dir = base_dir.as_ref();
    std::fs::create_dir_all(base_dir)?;
    std::fs::write(
        base_dir.join(ship_substrate::project::STACK_CONFIG_FILENAME),
        "export default {};\n",
    )?;
    for rel in ["skills", "rules", "agents"] {
        std::fs::create_dir_all(base_dir.join(rel))?;
    }
    Ok(base_dir.to_path_buf())
}
