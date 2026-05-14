//! ship.lock parser and writer.
//!
//! `ship.lock` is the registry dependency lockfile (`.ship/ship.lock`).
//! It records direct resolved dependencies — path, version, commit, and content hash.
//! On disk, the file is JSON (with deterministic key ordering for stable diffs).
//!
//! NOTE: `.ship/ship.state` (formerly `.ship/ship.lock`) stores workspace state
//! (active_profile, compiled_at). These are completely separate files.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::manifest::ShipManifest;

// ── Types ──────────────────────────────────────────────────────────────────────

/// A single locked package entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockPackage {
    /// Package path, e.g. `github.com/owner/repo`.
    pub path: String,
    /// Resolved version string (tag or branch@commit).
    pub version: String,
    /// Exact 40-char hex commit SHA.
    pub commit: String,
    /// Content hash in `sha256:<hex>` format (combined export hash).
    pub hash: String,
    /// Per-export integrity hashes: export path -> `"sha256:<hex>"`.
    /// Present for packages published with per-export hashing (v0.1+).
    /// Absent for legacy lockfiles — deserialized as empty map.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub export_hashes: BTreeMap<String, String>,
}

/// The parsed `.ship/ship.lock` file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShipLock {
    /// Lockfile format version. Must be `1` for v0.1.
    pub version: u32,
    /// Locked packages, one entry per direct dependency.
    #[serde(default)]
    pub packages: Vec<LockPackage>,
}

impl Default for ShipLock {
    fn default() -> Self {
        ShipLock {
            version: 1,
            packages: vec![],
        }
    }
}

impl ShipLock {
    /// Parse a `ship.lock` file.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        Self::from_json_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    /// Parse a `ship.lock` JSON string.
    pub fn from_json_str(s: &str) -> anyhow::Result<Self> {
        serde_json::from_str(s).map_err(Into::into)
    }

    /// Serialize to a deterministic JSON string.
    ///
    /// Packages are sorted by path so the output is byte-identical for the
    /// same logical content.
    pub fn to_json_string(&self) -> anyhow::Result<String> {
        let mut sorted = self.clone();
        sorted.packages.sort_by(|a, b| a.path.cmp(&b.path));
        let mut s = serde_json::to_string_pretty(&sorted)?;
        s.push('\n');
        Ok(s)
    }

    /// Atomically write the lockfile: write to `<path>.tmp` then rename.
    pub fn write_atomic(&self, path: &Path) -> anyhow::Result<()> {
        let tmp = path.with_extension("lock.tmp");
        let content = self.to_json_string()?;
        std::fs::write(&tmp, &content)
            .with_context(|| format!("Cannot write temp lockfile {}", tmp.display()))?;
        std::fs::rename(&tmp, path)
            .with_context(|| format!("Cannot rename lockfile into place at {}", path.display()))?;
        Ok(())
    }

    /// Compare manifest dependencies against locked packages.
    ///
    /// Returns `(added, removed)` where:
    /// - `added`   = dep paths in manifest but not in lockfile
    /// - `removed` = dep paths in lockfile but not in manifest
    pub fn check_sync(&self, manifest: &ShipManifest) -> (Vec<String>, Vec<String>) {
        let manifest_keys: std::collections::HashSet<&str> =
            manifest.dependencies.keys().map(|k| k.as_str()).collect();
        let lock_paths: std::collections::HashSet<&str> =
            self.packages.iter().map(|p| p.path.as_str()).collect();

        let added: Vec<String> = manifest_keys
            .difference(&lock_paths)
            .map(|s| s.to_string())
            .collect();
        let removed: Vec<String> = lock_paths
            .difference(&manifest_keys)
            .map(|s| s.to_string())
            .collect();
        (added, removed)
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ShipManifest;
    use std::collections::BTreeMap;

    fn sample_lock() -> ShipLock {
        ShipLock {
            version: 1,
            packages: vec![
                LockPackage {
                    path: "github.com/b/second".into(),
                    version: "1.0.0".into(),
                    commit: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
                    hash: "sha256:bbbb".into(),
                    export_hashes: BTreeMap::new(),
                },
                LockPackage {
                    path: "github.com/a/first".into(),
                    version: "2.0.0".into(),
                    commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                    hash: "sha256:aaaa".into(),
                    export_hashes: BTreeMap::new(),
                },
            ],
        }
    }

    fn minimal_manifest_jsonc() -> &'static str {
        r#"{
            "module": { "name": "github.com/x/y", "version": "1.0.0" }
        }"#
    }

    #[test]
    fn to_json_string_is_sorted() {
        let lock = sample_lock();
        let s = lock.to_json_string().unwrap();
        let a_pos = s.find("github.com/a/first").unwrap();
        let b_pos = s.find("github.com/b/second").unwrap();
        assert!(a_pos < b_pos, "packages should be sorted by path");
    }

    #[test]
    fn to_json_string_deterministic() {
        let lock = sample_lock();
        assert_eq!(lock.to_json_string().unwrap(), lock.to_json_string().unwrap());
    }

    #[test]
    fn round_trip_via_file() {
        let lock = sample_lock();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ship.lock");
        lock.write_atomic(&path).unwrap();
        let loaded = ShipLock::from_file(&path).unwrap();
        assert_eq!(loaded.version, 1);
        let mut names: Vec<_> = loaded.packages.iter().map(|p| p.path.as_str()).collect();
        names.sort();
        assert_eq!(names, vec!["github.com/a/first", "github.com/b/second"]);
    }

    #[test]
    fn from_file_missing_returns_error() {
        let r = ShipLock::from_file(std::path::Path::new("/nonexistent/ship.lock"));
        assert!(r.is_err());
    }

    #[test]
    fn check_sync_both_empty() {
        let lock = ShipLock::default();
        let manifest = ShipManifest::from_jsonc_str(minimal_manifest_jsonc()).unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert!(added.is_empty());
        assert!(removed.is_empty());
    }

    #[test]
    fn check_sync_added_dep() {
        let lock = ShipLock::default();
        let manifest = ShipManifest::from_jsonc_str(
            r#"{
                "module": { "name": "github.com/x/y", "version": "1.0.0" },
                "dependencies": { "github.com/a/b": "^1.0.0" }
            }"#,
        )
        .unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert_eq!(added, vec!["github.com/a/b"]);
        assert!(removed.is_empty());
    }

    #[test]
    fn check_sync_removed_dep() {
        let lock = ShipLock {
            version: 1,
            packages: vec![LockPackage {
                path: "github.com/a/b".into(),
                version: "1.0.0".into(),
                commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                hash: "sha256:aaaa".into(),
                export_hashes: BTreeMap::new(),
            }],
        };
        let manifest = ShipManifest::from_jsonc_str(minimal_manifest_jsonc()).unwrap();
        let (added, removed) = lock.check_sync(&manifest);
        assert!(added.is_empty());
        assert_eq!(removed, vec!["github.com/a/b"]);
    }

    #[test]
    fn write_atomic_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ship.lock");
        assert!(!path.exists());
        ShipLock::default().write_atomic(&path).unwrap();
        assert!(path.exists());
        assert!(!dir.path().join("ship.lock.tmp").exists());
    }

    #[test]
    fn version_field_is_written() {
        let s = ShipLock::default().to_json_string().unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v.get("version").and_then(|x| x.as_u64()), Some(1));
    }
}
