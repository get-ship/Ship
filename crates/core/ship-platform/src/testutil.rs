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
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(base_dir);
    std::fs::create_dir_all(&ship_dir)?;
    std::fs::write(ship_dir.join("ship.jsonc"), "{}\n")?;
    for rel in [
        "project/adrs",
        "project/specs",
        "project/features",
        "project/releases",
        "project/notes",
        "skills",
        "rules",
    ] {
        std::fs::create_dir_all(ship_dir.join(rel))?;
    }
    Ok(ship_dir)
}
