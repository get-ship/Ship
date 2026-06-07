//! Test helpers for ship-agents integration tests.
//!
//! Provides `init_project` — creates a minimal `.ship/` tree in the given
//! directory and installs platform defaults (projection factory + supervisor
//! hooks) so `SqliteEventStore` sees the full bus. This mirrors the
//! `ship_platform::testutil::init_project` helper that ship-platform exposes
//! only when compiling its own tests (cfg(test)); ship-agents tests reference
//! this copy directly to avoid the cfg(test) gating.

use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn init_project(base_dir: impl AsRef<Path>) -> Result<PathBuf> {
    // Install platform-level projection factory + supervisor hooks exactly
    // as ship_platform::install_defaults() does.
    ship_platform::install_defaults();
    let base_dir = base_dir.as_ref();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(base_dir);
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
