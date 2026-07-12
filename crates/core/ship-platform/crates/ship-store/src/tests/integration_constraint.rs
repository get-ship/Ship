/// Integration tests — constraint + types sub-domain.
///
/// Exercises `parse_constraint`, `normalize_version`, `diff_manifest_against_lock`,
/// and `SyncStatus::is_in_sync` at the integration level. The constraint module has
/// inline unit tests for individual parse arms; here we test combined flows and the
/// manifest-vs-lock diff pipeline the install orchestrator relies on.
use std::collections::HashMap;

use stack_model::lockfile::{LockPackage, ShipLock};

use crate::constraint::{VersionConstraint, normalize_version, parse_constraint};
use crate::types::{Dependency, ShipManifest, SyncStatus, diff_manifest_against_lock};

// ── parse_constraint — integration coverage ───────────────────────────────────

#[test]
fn registry_constraint_parse_commit_sha_is_40_hex() {
    let sha = "abcdef1234567890".repeat(2) + "abcdef12"; // exactly 40 hex chars
    let c = parse_constraint(&sha).unwrap();
    assert!(matches!(c, VersionConstraint::Commit(_)));
}

#[test]
fn registry_constraint_parse_semver_range_caret() {
    let c = parse_constraint("^1.2.3").unwrap();
    assert!(matches!(c, VersionConstraint::Semver(_)));
}

#[test]
fn registry_constraint_parse_semver_range_gt() {
    let c = parse_constraint(">0.5.0").unwrap();
    assert!(matches!(c, VersionConstraint::Semver(_)));
}

#[test]
fn registry_constraint_parse_branch_with_slash() {
    let c = parse_constraint("feat/my-feature").unwrap();
    assert!(matches!(c, VersionConstraint::Branch(ref b) if b == "feat/my-feature"));
}

#[test]
fn registry_constraint_parse_invalid_semver_errors() {
    let err = parse_constraint("^not-valid").expect_err("bad semver must fail");
    let msg = err.to_string();
    assert!(msg.contains("semver") || msg.contains("invalid"), "error: {msg}");
}

#[test]
fn registry_constraint_normalize_version_strips_leading_v() {
    assert_eq!(normalize_version("v1.2.3"), "1.2.3");
    assert_eq!(normalize_version("1.2.3"), "1.2.3");
    assert_eq!(normalize_version("v0.0.1-alpha"), "0.0.1-alpha");
}

// ── diff_manifest_against_lock — integration ──────────────────────────────────

fn dep(version: &str) -> Dependency {
    Dependency { version: version.into(), grant: vec![] }
}

fn pkg(path: &str) -> LockPackage {
    LockPackage {
        path: path.into(),
        version: "1.0.0".into(),
        commit: "a".repeat(40),
        hash: "sha256:abc".into(),
        export_hashes: Default::default(),
    }
}

#[test]
fn registry_constraint_diff_in_sync_when_manifest_matches_lock() {
    let mut deps = HashMap::new();
    deps.insert("github.com/owner/pkg".to_string(), dep("^1.0"));
    let manifest = ShipManifest { dependencies: deps };

    let lock = ShipLock {
        version: 1,
        packages: vec![pkg("github.com/owner/pkg")],
    };
    let status = diff_manifest_against_lock(&manifest, &lock);
    assert!(status.is_in_sync(), "identical dep sets must be in sync");
    assert!(status.added.is_empty());
    assert!(status.removed.is_empty());
}

#[test]
fn registry_constraint_diff_added_when_manifest_has_extra_dep() {
    let mut deps = HashMap::new();
    deps.insert("github.com/owner/new".to_string(), dep("^2.0"));
    let manifest = ShipManifest { dependencies: deps };

    let lock = ShipLock::default(); // empty
    let status = diff_manifest_against_lock(&manifest, &lock);
    assert!(!status.is_in_sync());
    assert!(
        status.added.contains(&"github.com/owner/new".to_string()),
        "new dep must appear in added: {:?}",
        status.added
    );
    assert!(status.removed.is_empty());
}

#[test]
fn registry_constraint_diff_removed_when_lock_has_extra_package() {
    let manifest = ShipManifest::default(); // empty

    let lock = ShipLock {
        version: 1,
        packages: vec![pkg("github.com/owner/old")],
    };
    let status = diff_manifest_against_lock(&manifest, &lock);
    assert!(!status.is_in_sync());
    assert!(
        status.removed.contains(&"github.com/owner/old".to_string()),
        "removed dep must appear in removed: {:?}",
        status.removed
    );
    assert!(status.added.is_empty());
}

#[test]
fn registry_constraint_diff_both_added_and_removed() {
    let mut deps = HashMap::new();
    deps.insert("github.com/owner/new".to_string(), dep("^1.0"));
    let manifest = ShipManifest { dependencies: deps };

    let lock = ShipLock {
        version: 1,
        packages: vec![pkg("github.com/owner/old")],
    };
    let status = diff_manifest_against_lock(&manifest, &lock);
    assert!(!status.is_in_sync());
    assert!(status.added.contains(&"github.com/owner/new".to_string()));
    assert!(status.removed.contains(&"github.com/owner/old".to_string()));
}

// ── SyncStatus::is_in_sync ────────────────────────────────────────────────────

#[test]
fn registry_constraint_sync_status_default_is_in_sync() {
    let status = SyncStatus::default();
    assert!(status.is_in_sync(), "default SyncStatus must be in sync");
}

#[test]
fn registry_constraint_sync_status_not_in_sync_when_added_non_empty() {
    let status = SyncStatus {
        added: vec!["github.com/x/y".into()],
        removed: vec![],
    };
    assert!(!status.is_in_sync());
}
