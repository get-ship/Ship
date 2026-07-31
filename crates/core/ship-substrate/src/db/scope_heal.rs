//! Heal pass for unregistered Space scope ids.
//!
//! Historical collections/triggers misuse could mint `spaces/<key>/space.db`
//! without a corresponding `spaces` registry row. Before the strict scope
//! validator rejects those ids at the write chokepoint, this heal pass
//! enumerates orphaned space directories and quarantines them.
//!
//! Called at daemon boot (before integrity) and available for offline heal
//! against a copy of production ship-home. When no production db is available,
//! tests exercise the pass with fabricated fixtures.
//!
//! **Fail-closed on registry read:** if the `spaces` registry cannot be read,
//! this returns `Err` and renames nothing. Never treat a failed read as an
//! empty registered set (that would quarantine every live space dir).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::db::spaces;

/// One orphaned space directory moved under `spaces/.quarantine/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantinedScope {
    pub scope_key: String,
    pub from: PathBuf,
    pub to: PathBuf,
}

/// Result of [`heal_unregistered_scopes`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HealReport {
    /// Scope keys that were quarantined (had a directory, no registry row).
    pub quarantined: Vec<QuarantinedScope>,
    /// Scope keys still unregistered after the pass (should be empty).
    pub remaining_unregistered: Vec<String>,
    /// Registered space ids observed in the registry at heal time.
    pub registered_count: usize,
    /// Space directories examined under `spaces/`.
    pub examined_dirs: usize,
}

impl HealReport {
    /// True when no unregistered scope ids remain.
    pub fn is_clean(&self) -> bool {
        self.remaining_unregistered.is_empty()
    }
}

/// Enumerate `spaces/<id>/` under `ship_dir`, quarantine any directory whose
/// id has no `spaces` registry row, and report remaining unregistered ids.
///
/// Quarantine path: `spaces/.quarantine/<id>/` (the directory is renamed,
/// preserving the orphaned `space.db` for forensics). Registry rows are never
/// invented — only moved aside so the strict validator sees a clean set.
///
/// Returns `Err` if the spaces registry cannot be read — no directories are
/// renamed in that case (fail closed).
pub fn heal_unregistered_scopes(ship_dir: &Path) -> Result<HealReport> {
    let spaces_root = ship_dir.join("spaces");
    let mut report = HealReport::default();

    // Fail closed: a registry read error must not look like "zero registered
    // spaces" (which would quarantine every spaces/<id>/ dir).
    let registered = spaces::list_open_at(ship_dir).with_context(|| {
        format!(
            "heal_unregistered_scopes: read spaces registry under {}",
            ship_dir.display()
        )
    })?;
    report.registered_count = registered.len();
    let registered_ids: std::collections::HashSet<String> =
        registered.into_iter().map(|r| r.space_id).collect();

    if !spaces_root.is_dir() {
        // No space dirs → nothing to heal; remaining is empty.
        return Ok(report);
    }

    let quarantine_root = spaces_root.join(".quarantine");
    let entries = fs::read_dir(&spaces_root)
        .with_context(|| format!("read spaces dir {}", spaces_root.display()))?;

    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue; // skip .quarantine and other dot-dirs
        }
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        report.examined_dirs += 1;
        if registered_ids.contains(name.as_ref()) {
            continue;
        }
        // Orphan: quarantine.
        fs::create_dir_all(&quarantine_root).with_context(|| {
            format!("create quarantine dir {}", quarantine_root.display())
        })?;
        let dest = unique_quarantine_path(&quarantine_root, name.as_ref());
        fs::rename(&path, &dest).with_context(|| {
            format!(
                "quarantine unregistered space `{}`: {} → {}",
                name,
                path.display(),
                dest.display()
            )
        })?;
        report.quarantined.push(QuarantinedScope {
            scope_key: name.to_string(),
            from: path,
            to: dest,
        });
    }

    // Re-scan: any remaining unregistered dirs?
    report.remaining_unregistered = list_unregistered_space_dirs(ship_dir)?;
    Ok(report)
}

/// List scope keys under `spaces/` that have no registry row (excludes
/// `.quarantine` and other dot-dirs). Used for heal verification.
///
/// Returns `Err` if the spaces registry cannot be read (fail closed).
pub fn list_unregistered_space_dirs(ship_dir: &Path) -> Result<Vec<String>> {
    let spaces_root = ship_dir.join("spaces");
    if !spaces_root.is_dir() {
        return Ok(Vec::new());
    }
    let registered = spaces::list_open_at(ship_dir).with_context(|| {
        format!(
            "list_unregistered_space_dirs: read spaces registry under {}",
            ship_dir.display()
        )
    })?;
    let registered_ids: std::collections::HashSet<String> =
        registered.into_iter().map(|r| r.space_id).collect();

    let mut out = Vec::new();
    for entry in fs::read_dir(&spaces_root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || !entry.path().is_dir() {
            continue;
        }
        if !registered_ids.contains(name.as_ref()) {
            out.push(name.to_string());
        }
    }
    out.sort();
    Ok(out)
}

fn unique_quarantine_path(root: &Path, name: &str) -> PathBuf {
    let base = root.join(name);
    if !base.exists() {
        return base;
    }
    for i in 1..1000 {
        let candidate = root.join(format!("{name}.{i}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    root.join(format!("{name}.{}", ulid_like()))
}

fn ulid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn heal_quarantines_unregistered_and_leaves_registered() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        crate::testutil::register_space(root, "good").unwrap();
        // Fabricated orphan dirs (collections/triggers-style misuse).
        fs::create_dir_all(root.join("spaces/orphan-a")).unwrap();
        fs::write(root.join("spaces/orphan-a/space.db"), b"fake").unwrap();
        fs::create_dir_all(root.join("spaces/orphan-b")).unwrap();

        let report = heal_unregistered_scopes(root).unwrap();
        // register_space only inserts a registry row — it does not create
        // spaces/good/, so examined_dirs is the two orphans only.
        assert_eq!(report.examined_dirs, 2);
        assert_eq!(report.quarantined.len(), 2);
        assert!(report.is_clean(), "remaining: {:?}", report.remaining_unregistered);
        assert!(!root.join("spaces/orphan-a").exists());
        assert!(!root.join("spaces/orphan-b").exists());
        assert!(root.join("spaces/.quarantine/orphan-a").is_dir());
        assert!(root.join("spaces/.quarantine/orphan-b").is_dir());
    }

    #[test]
    fn heal_clean_when_no_orphans() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        crate::testutil::register_space(root, "only-good").unwrap();
        fs::create_dir_all(root.join("spaces/only-good")).unwrap();

        let report = heal_unregistered_scopes(root).unwrap();
        assert!(report.quarantined.is_empty());
        assert!(report.is_clean());
        assert_eq!(report.remaining_unregistered, Vec::<String>::new());
    }

    #[test]
    fn list_unregistered_finds_orphan() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("spaces/ghost")).unwrap();
        // Ensure runtime.db exists so list_open_at works.
        let _ = crate::db::open_runtime_db(root);
        let orphans = list_unregistered_space_dirs(root).unwrap();
        assert_eq!(orphans, vec!["ghost".to_string()]);
    }

    #[test]
    fn heal_fails_closed_when_registry_unreadable() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        // Corrupt runtime.db so open/list of the spaces registry fails.
        // (A missing runtime.db is auto-created by open_runtime_db — not a fail.)
        fs::write(root.join("runtime.db"), b"not-a-sqlite-database").unwrap();
        fs::create_dir_all(root.join("spaces/would-be-orphaned")).unwrap();
        fs::write(root.join("spaces/would-be-orphaned/space.db"), b"fake").unwrap();

        let err = heal_unregistered_scopes(root).unwrap_err();
        assert!(
            err.to_string().contains("registry")
                || err.to_string().contains("spaces")
                || err.to_string().contains("database")
                || err.to_string().contains("SQLite")
                || err.to_string().contains("sqlite"),
            "expected registry read error, got: {err}"
        );
        // Must not have quarantined anything.
        assert!(root.join("spaces/would-be-orphaned").is_dir());
        assert!(!root.join("spaces/.quarantine").exists());
    }
}
