use anyhow::Result;
use std::path::{Path, PathBuf};

/// Create a `.ship/` tree for L1 tests.
/// Writes `ship.jsonc` with a generated id, plus the subdirectories that
/// L1 functions (skills, rules, collections) expect to exist.
///
/// Also installs the platform-level projection factory + supervisor hooks
/// so any `SqliteEventStore` constructed by the test sees the full bus.
/// Idempotent across tests.
pub fn init_project(base_dir: impl AsRef<Path>) -> Result<PathBuf> {
    crate::install_defaults();
    let base_dir = base_dir.as_ref();
    let ship_dir = base_dir.join(".ship");
    std::fs::create_dir_all(&ship_dir)?;
    let id = ship_substrate::gen_nanoid();
    std::fs::write(
        ship_dir.join("ship.jsonc"),
        format!("{{\"id\": \"{id}\"}}\n"),
    )?;
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
