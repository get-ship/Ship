//! Registry manifest types used by the install/resolve pipeline.
//!
//! The lock-file types (`ShipLock`, `LockPackage`) are owned by
//! `stack_model::lockfile`. This module previously kept a parallel pair —
//! that was the source of a real bug in P19 where the two sides
//! disagreed on the on-disk JSON field name (`package` vs `packages`).
//! Both reads and writes now go through the compiler crate.
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A single dependency entry in ship.jsonc `dependencies`.
///
/// Distinct from `stack_model::manifest::ManifestDependency` — the install
/// pipeline only needs version + grants and avoids the full manifest schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Version constraint: semver range, branch, or 40-char commit SHA.
    pub version: String,
    /// Optional tool-permission grants for this dep's skills.
    #[serde(default)]
    pub grant: Vec<String>,
}

/// Minimal ship.jsonc manifest projection for install/resolve.
///
/// Distinct from `stack_model::manifest::ShipManifest` — that one carries the
/// `[module]` section and richer metadata that install doesn't need.
#[derive(Debug, Clone, Default)]
pub struct ShipManifest {
    /// `dependencies` map: dep path → Dependency.
    pub dependencies: HashMap<String, Dependency>,
}

/// Result of comparing ship.jsonc and ship.lock.
#[derive(Debug, Default)]
pub struct SyncStatus {
    /// Deps in manifest but not in lock (newly added).
    pub added: Vec<String>,
    /// Deps in lock but not in manifest (removed).
    pub removed: Vec<String>,
}

impl SyncStatus {
    pub fn is_in_sync(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

/// Diff a manifest against a lockfile. Used by the install pipeline to
/// decide which deps need re-resolution.
///
/// `manifest` carries the user-declared deps; `lock` carries the resolved
/// package paths. `added` = in manifest, not in lock. `removed` = in lock,
/// not in manifest.
pub fn diff_manifest_against_lock(
    manifest: &ShipManifest,
    lock: &stack_model::lockfile::ShipLock,
) -> SyncStatus {
    let lock_paths: std::collections::HashSet<&str> =
        lock.packages.iter().map(|p| p.path.as_str()).collect();
    let manifest_paths: std::collections::HashSet<&str> =
        manifest.dependencies.keys().map(|s| s.as_str()).collect();
    SyncStatus {
        added: manifest_paths
            .difference(&lock_paths)
            .map(|s| s.to_string())
            .collect(),
        removed: lock_paths
            .difference(&manifest_paths)
            .map(|s| s.to_string())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_model::lockfile::{LockPackage, Rails, ShipLock};

    #[test]
    fn diff_in_sync() {
        let manifest = ShipManifest {
            dependencies: [(
                "github.com/owner/pkg".into(),
                Dependency {
                    version: "^1.0".into(),
                    grant: vec![],
                },
            )]
            .into(),
        };
        let lock = ShipLock {
            version: 1,
            packages: vec![LockPackage {
                path: "github.com/owner/pkg".into(),
                version: "v1.0.0".into(),
                commit: "a".repeat(40),
                hash: "sha256:abc".into(),
                rails: Rails::Stack,
                export_hashes: Default::default(),
            }],
        };
        assert!(diff_manifest_against_lock(&manifest, &lock).is_in_sync());
    }

    #[test]
    fn diff_added() {
        let manifest = ShipManifest {
            dependencies: [(
                "github.com/owner/new".into(),
                Dependency {
                    version: "^1.0".into(),
                    grant: vec![],
                },
            )]
            .into(),
        };
        let lock = ShipLock::default();
        let status = diff_manifest_against_lock(&manifest, &lock);
        assert!(!status.is_in_sync());
        assert!(status.added.contains(&"github.com/owner/new".to_string()));
        assert!(status.removed.is_empty());
    }

    #[test]
    fn diff_removed() {
        let manifest = ShipManifest::default();
        let lock = ShipLock {
            version: 1,
            packages: vec![LockPackage {
                path: "github.com/owner/old".into(),
                version: "v1.0.0".into(),
                commit: "a".repeat(40),
                hash: "sha256:abc".into(),
                rails: Rails::Stack,
                export_hashes: Default::default(),
            }],
        };
        let status = diff_manifest_against_lock(&manifest, &lock);
        assert!(!status.is_in_sync());
        assert!(status.removed.contains(&"github.com/owner/old".to_string()));
    }
}
