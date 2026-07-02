//! Space enumeration — reads the `projects` pointer (runtime.db), never an
//! FS scan. `.ship/` presence only locates a root; only registration
//! (`registry::add`) makes it a space.

use crate::project::ProjectDiscovery;
use anyhow::Result;
use std::path::PathBuf;

/// Enumerate registered spaces whose canonical path lives under `root`
/// (inclusive). Sourced from the `projects` pointer — an unregistered
/// `.ship/` directory is NOT a space and does not appear. `path` is the
/// registered project root (the pointer's canonical path).
pub fn discover_projects(root: PathBuf) -> Result<Vec<ProjectDiscovery>> {
    let root_key = PathBuf::from(ship_substrate::db::projects::canonical_path_key(&root));
    Ok(ship_substrate::db::projects::list_active()?
        .into_iter()
        .filter(|record| PathBuf::from(&record.path).starts_with(&root_key))
        .map(|record| ProjectDiscovery {
            name: record.name,
            path: PathBuf::from(record.path),
        })
        .collect())
}
