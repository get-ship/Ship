use anyhow::Result;
use std::path::{Path, PathBuf};

/// Create a `.ship/` tree for L1 tests: an empty `ship.jsonc` config scaffold
/// (presence only — identity lives in the `projects` pointer) plus the
/// subdirectories that L1 functions (skills, rules, collections) expect.
///
/// Also installs the platform-level projection factory + supervisor hooks
/// so any `SqliteEventStore` constructed by the test sees the full bus.
/// Idempotent across tests.
pub fn init_project(base_dir: impl AsRef<Path>) -> Result<PathBuf> {
    crate::install_defaults();
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
