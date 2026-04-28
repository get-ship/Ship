//! Project registry — the daemon's record of project paths it has seen.
//!
//! A "project" today is any directory the daemon has been pointed at
//! (via `--workspace <path>`, or future API calls). The registry exists
//! so the launcher / shell can list projects without walking the
//! filesystem, and so other features (private project-keyed config,
//! per-project KV namespacing) can address projects by stable id.
//!
//! Storage is `kv_state` under namespace `project`. One row per project,
//! keyed by `project_id` (8-char hex hash of the canonical path). Value
//! is a JSON `ProjectRecord`.
//!
//! Registration is idempotent: calling `register` against a known path
//! returns the existing record with `last_seen_at` bumped. Callers that
//! want strict create semantics can check `get` first.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::db::kv;
use crate::home::path_hash;

const NAMESPACE: &str = "project";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub created_at: String,
    pub last_seen_at: String,
}

/// Idempotent register. Returns the record (newly created or existing
/// with `last_seen_at` bumped). Path is canonicalised when possible.
pub fn register(path: &Path, name: Option<&str>) -> Result<ProjectRecord> {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let id = path_hash(&canonical);
    let now = Utc::now().to_rfc3339();
    let record = match get(&id)? {
        Some(mut existing) => {
            existing.last_seen_at = now;
            existing
        }
        None => ProjectRecord {
            id: id.clone(),
            path: canonical.to_string_lossy().to_string(),
            name: name
                .map(str::to_string)
                .unwrap_or_else(|| derive_name(&canonical)),
            created_at: now.clone(),
            last_seen_at: now,
        },
    };
    let value = serde_json::to_value(&record).context("serialize project record")?;
    kv::set(NAMESPACE, &id, &value)?;
    Ok(record)
}

/// Get a project by id.
pub fn get(id: &str) -> Result<Option<ProjectRecord>> {
    let Some(value) = kv::get(NAMESPACE, id)? else {
        return Ok(None);
    };
    let record: ProjectRecord =
        serde_json::from_value(value).context("deserialize project record")?;
    Ok(Some(record))
}

/// Look up a project by filesystem path. Canonicalises the input, hashes,
/// and returns the registered record if any. Returns `None` for paths that
/// haven't been registered (or aren't readable) — does NOT register on miss.
pub fn find_by_path(path: &Path) -> Result<Option<ProjectRecord>> {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let id = path_hash(&canonical);
    get(&id)
}

/// List all known projects, sorted by `last_seen_at` desc.
pub fn list() -> Result<Vec<ProjectRecord>> {
    let entries = kv::list_all(NAMESPACE)?;
    let mut out: Vec<ProjectRecord> = entries
        .into_iter()
        .filter_map(|(_, v)| serde_json::from_value(v).ok())
        .collect();
    out.sort_by(|a, b| b.last_seen_at.cmp(&a.last_seen_at));
    Ok(out)
}

/// Remove a project from the registry. Does not touch the on-disk
/// project; only forgets the daemon's record of having seen it.
pub fn forget(id: &str) -> Result<()> {
    kv::delete(NAMESPACE, id)
}

/// Update only the `last_seen_at` for an existing project. Returns
/// `false` if the project isn't registered.
pub fn touch(id: &str) -> Result<bool> {
    let Some(mut existing) = get(id)? else {
        return Ok(false);
    };
    existing.last_seen_at = Utc::now().to_rfc3339();
    let value = serde_json::to_value(&existing)?;
    kv::set(NAMESPACE, id, &value)?;
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
        let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        crate::project::init_project(tmp.path().to_path_buf()).unwrap();
        crate::db::ensure_db().unwrap();
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
        register(a.path(), Some("a")).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        register(b.path(), Some("b")).unwrap();
        let listed = list().unwrap();
        assert!(listed.len() >= 2);
        // b registered most recently → should be at front
        let b_pos = listed.iter().position(|p| p.name == "b").unwrap();
        let a_pos = listed.iter().position(|p| p.name == "a").unwrap();
        assert!(b_pos < a_pos, "newer project sorts first");
    }

    #[test]
    fn forget_removes_record() {
        let _tmp = fresh_db();
        let tmp = TempDir::new().unwrap();
        let r = register(tmp.path(), None).unwrap();
        assert!(get(&r.id).unwrap().is_some());
        forget(&r.id).unwrap();
        assert!(get(&r.id).unwrap().is_none());
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
