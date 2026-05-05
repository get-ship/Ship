//! Minimal test scaffolding for ship-substrate tests.
//!
//! `init_project_minimal` creates just enough of a `.ship/` tree for DB
//! tests to work (stable state-directory path derivation). It is intentionally
//! thinner than `ship_application::project::init_project` — no config schemas, no
//! skills directories, no registry calls.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub const SHIP_DIR_NAME: &str = ".ship";
pub const PRIMARY_CONFIG_FILE: &str = "ship.jsonc";

/// Create a minimal `.ship/` directory under `base_dir` suitable for L0 tests.
/// Writes only the `ship.jsonc` id field; no config schemas or skills.
pub fn init_project_minimal(base_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let base_dir = base_dir.as_ref();
    let ship_dir = base_dir.join(SHIP_DIR_NAME);
    std::fs::create_dir_all(&ship_dir)?;
    let id = crate::gen_nanoid();
    std::fs::write(
        ship_dir.join(PRIMARY_CONFIG_FILE),
        format!("{{\"id\": \"{id}\"}}\n"),
    )?;
    Ok(ship_dir)
}
