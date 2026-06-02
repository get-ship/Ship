//! Project registry — the daemon's record of project paths it has seen.
//!
//! A "project" is any directory the daemon has been pointed at (via
//! `--workspace <path>` or future API calls). The registry exists so the
//! launcher / shell can list projects without walking the filesystem, and so
//! other features (per-project state, sync) can address projects by stable id.
//!
//! Storage is event-sourced: `project.registered`, `project.seen`, and
//! `project.archived` events feed the `projects` projection table. Callers
//! read through this module's accessors; writes emit events through
//! `SqliteEventStore::append`.
//!
//! Registration is idempotent: calling `register` against a known path
//! emits a `project.seen` event (cheaper, just bumps `last_seen_at`).

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

use ship_substrate::events::EventEnvelope;
use ship_substrate::home::path_hash;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use crate::projections::projects;

/// Open the Runtime-scope store for the daemon-wide project registry.
///
/// The `projects` table is the machine's record of known project paths, read
/// daemon-wide with no space in hand (and the source the `space_id` resolver
/// itself consults), so `project.registered/seen/archived` are Runtime-owned
/// and land in runtime.db — matching the projection's read side.
fn projects_store() -> Result<SqliteEventStore> {
    SqliteEventStore::new_for_scope(
        &ship_substrate::project::get_global_dir()?,
        ship_types::event_registry::EventScope::Runtime,
        None,
    )
}
use ship_types::event_types::{PROJECT_ARCHIVED, PROJECT_REGISTERED, PROJECT_SEEN};
use ship_types::payload::{ProjectArchived, ProjectRegistered, ProjectSeen};

// Project bootstrap / registry-backed init flow. L1 because it depends on
// config schemas and the package registry — see `init.rs` doc comment.
mod init;
pub use init::{InitWithRegistryResult, init_project, init_project_with_registry};

mod active;
pub use active::{get_active, get_active_id, switch_to};

#[cfg(test)]
mod init_tests;

/// Public-facing record. Backed by the `projects` projection row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub created_at: String,
    pub last_seen_at: String,
}

impl From<projects::ProjectRow> for ProjectRecord {
    fn from(row: projects::ProjectRow) -> Self {
        Self {
            id: row.id,
            path: row.path,
            name: row.name,
            created_at: row.created_at,
            last_seen_at: row.last_seen_at,
        }
    }
}

/// Idempotent register. Returns the record (newly created or existing
/// with `last_seen_at` bumped). Path is canonicalised when possible.
pub fn register(path: &Path, name: Option<&str>) -> Result<ProjectRecord> {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let id = path_hash(&canonical);
    let now = Utc::now().to_rfc3339();
    let store = projects_store()?;

    if let Some(existing) = projects::get(&id)? {
        let payload = ProjectSeen {
            seen_at: now.clone(),
        };
        let envelope = EventEnvelope::new(PROJECT_SEEN, &id, &payload)
            .context("build project.seen envelope")?;
        store.append(&envelope)?;
        return Ok(ProjectRecord {
            id,
            path: existing.path,
            name: existing.name,
            created_at: existing.created_at,
            last_seen_at: now,
        });
    }

    let resolved_name = name
        .map(str::to_string)
        .unwrap_or_else(|| derive_name(&canonical));
    let payload = ProjectRegistered {
        path: canonical.to_string_lossy().to_string(),
        name: resolved_name.clone(),
        label: None,
        created_at: now.clone(),
    };
    let envelope = EventEnvelope::new(PROJECT_REGISTERED, &id, &payload)
        .context("build project.registered envelope")?;
    store.append(&envelope)?;
    Ok(ProjectRecord {
        id,
        path: canonical.to_string_lossy().to_string(),
        name: resolved_name,
        created_at: now.clone(),
        last_seen_at: now,
    })
}

/// Get a project by id.
pub fn get(id: &str) -> Result<Option<ProjectRecord>> {
    Ok(projects::get(id)?.map(ProjectRecord::from))
}

/// Look up a project by filesystem path.
pub fn find_by_path(path: &Path) -> Result<Option<ProjectRecord>> {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let path_str = canonical.to_string_lossy().to_string();
    Ok(projects::find_by_path(&path_str)?.map(ProjectRecord::from))
}

/// List all known live (non-archived) projects, sorted by `last_seen_at` desc.
pub fn list() -> Result<Vec<ProjectRecord>> {
    Ok(projects::list_active()?
        .into_iter()
        .map(ProjectRecord::from)
        .collect())
}

/// Archive a project (soft-delete from the registry — projection row stays
/// for audit/replay but `list()` skips it).
pub fn forget(id: &str) -> Result<()> {
    let payload = ProjectArchived {
        archived_at: Utc::now().to_rfc3339(),
    };
    let envelope = EventEnvelope::new(PROJECT_ARCHIVED, id, &payload)
        .context("build project.archived envelope")?;
    let store = projects_store()?;
    store.append(&envelope)?;
    Ok(())
}

/// Update only the `last_seen_at` for an existing project. Returns
/// `false` if the project isn't registered.
pub fn touch(id: &str) -> Result<bool> {
    if projects::get(id)?.is_none() {
        return Ok(false);
    }
    let payload = ProjectSeen {
        seen_at: Utc::now().to_rfc3339(),
    };
    let envelope = EventEnvelope::new(PROJECT_SEEN, id, &payload)?;
    let store = projects_store()?;
    store.append(&envelope)?;
    Ok(true)
}


fn derive_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh_db() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        crate::testutil::init_project(tmp.path()).unwrap();
        ship_substrate::db::ensure_db().unwrap();
        tmp
    }

    #[test]
    fn register_returns_stable_id() {
        let _tmp = fresh_db();
        let tmp = TempDir::new().unwrap();
        let r1 = register(tmp.path(), Some("test")).unwrap();
        let r2 = register(tmp.path(), Some("test-renamed")).unwrap();
        assert_eq!(r1.id, r2.id, "id stable across re-register");
        assert_eq!(r2.name, "test", "name preserved on second register");
    }

    #[test]
    fn list_returns_recent_first() {
        let _tmp = fresh_db();
        let a = TempDir::new().unwrap();
        let b = TempDir::new().unwrap();
        register(a.path(), Some("re-a")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        register(b.path(), Some("re-b")).unwrap();
        let listed = list().unwrap();
        let b_pos = listed.iter().position(|p| p.name == "re-b").unwrap();
        let a_pos = listed.iter().position(|p| p.name == "re-a").unwrap();
        assert!(b_pos < a_pos, "newer project sorts first");
    }

    #[test]
    fn forget_archives_project() {
        let _tmp = fresh_db();
        let tmp = TempDir::new().unwrap();
        let r = register(tmp.path(), None).unwrap();
        assert!(get(&r.id).unwrap().is_some());
        forget(&r.id).unwrap();
        // Row still exists in the projection (audit), but list() filters it out.
        assert!(!list().unwrap().iter().any(|p| p.id == r.id));
    }

    #[test]
    fn touch_updates_last_seen() {
        let _tmp = fresh_db();
        let tmp = TempDir::new().unwrap();
        let r1 = register(tmp.path(), None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(touch(&r1.id).unwrap());
        let r2 = get(&r1.id).unwrap().unwrap();
        assert!(r2.last_seen_at > r1.last_seen_at);
    }

    #[test]
    fn touch_unknown_returns_false() {
        let _tmp = fresh_db();
        assert!(!touch("deadbeef").unwrap());
    }

    #[test]
    fn find_by_path_returns_registered_record() {
        let _tmp = fresh_db();
        let tmp = TempDir::new().unwrap();
        let registered = register(tmp.path(), Some("by-path")).unwrap();
        let found = find_by_path(tmp.path()).unwrap().expect("path lookup hit");
        assert_eq!(found.id, registered.id);
    }

    #[test]
    fn find_by_path_returns_none_for_unregistered() {
        let _tmp = fresh_db();
        let unregistered = TempDir::new().unwrap();
        assert!(find_by_path(unregistered.path()).unwrap().is_none());
    }
}
