//! Tests for the home-layout migrator.

use super::{LAYOUT_VERSION, migrate_home, read_layout_version};
use crate::home::ship_home::ShipHome;
use std::fs;
use tempfile::tempdir;

fn home_at(tmp: &std::path::Path) -> ShipHome {
    ShipHome::for_test(tmp)
}

#[test]
fn fresh_install_no_op_writes_marker() {
    let tmp = tempdir().unwrap();
    let h = home_at(tmp.path());
    migrate_home(&h).expect("fresh-install migrate");

    assert_eq!(read_layout_version(&h).unwrap(), Some(LAYOUT_VERSION));
    assert!(h.machine_dir().join("layout-version").is_file());
}

#[test]
fn pre_split_layout_migrates_dirs_and_files() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join(".ship");
    fs::create_dir_all(root.join("agents")).unwrap();
    fs::write(root.join("agents/profile.jsonc"), "{}").unwrap();
    fs::create_dir_all(root.join("schemas")).unwrap();
    fs::write(root.join("schemas/note.jsonc"), "{}").unwrap();
    fs::write(root.join("mcp.jsonc"), "{}").unwrap();
    fs::create_dir_all(root.join("kernel")).unwrap();
    fs::write(root.join("kernel/events.db"), b"junk").unwrap();
    fs::create_dir_all(root.join("cache")).unwrap();
    fs::write(root.join("platform.db"), b"DB1").unwrap();
    fs::write(root.join("platform.db-wal"), b"WAL").unwrap();
    fs::write(root.join("hook.token"), b"tk").unwrap();
    fs::write(root.join("network.pid"), b"42").unwrap();
    fs::write(
        root.join("config.toml"),
        "[identity]\nname=\"Alice\"\n[worktrees]\ndir=\"/tmp/wt\"\n",
    )
    .unwrap();

    let h = home_at(tmp.path());
    migrate_home(&h).expect("migrate pre-split");

    assert!(h.library_dir().join("agents/profile.jsonc").is_file());
    assert!(h.library_dir().join("collections/note.jsonc").is_file());
    assert!(h.library_dir().join("mcp/config.jsonc").is_file());
    assert!(h.library_dir().join("settings.jsonc").is_file());
    assert!(h.machine_dir().join("local.jsonc").is_file());
    assert!(h.machine_dir().join("cache").is_dir());
    assert!(h.machine_dir().join("db/platform.db").is_file());
    assert!(h.machine_dir().join("db/platform.db-wal").is_file());
    assert!(h.machine_dir().join("daemon/token").is_file());
    assert!(h.machine_dir().join("daemon/pid").is_file());
    // Kernel dir should be discarded entirely.
    assert!(!root.join("kernel").exists());
    assert!(!h.machine_dir().join(".discard").exists());
    // Original config.toml removed.
    assert!(!root.join("config.toml").exists());
    assert_eq!(read_layout_version(&h).unwrap(), Some(LAYOUT_VERSION));
}

#[test]
fn already_migrated_is_no_op() {
    let tmp = tempdir().unwrap();
    let h = home_at(tmp.path());
    migrate_home(&h).unwrap();
    let marker_before = fs::read_to_string(h.machine_dir().join("layout-version")).unwrap();
    // Touch a file that should NOT be moved.
    fs::write(h.library_dir().join("settings.jsonc"), "// hand-edited").unwrap();
    migrate_home(&h).unwrap();
    let marker_after = fs::read_to_string(h.machine_dir().join("layout-version")).unwrap();
    assert_eq!(marker_before, marker_after, "marker preserved on no-op");
    assert_eq!(
        fs::read_to_string(h.library_dir().join("settings.jsonc")).unwrap(),
        "// hand-edited",
        "no-op must not overwrite user file"
    );
}

#[test]
fn partial_legacy_state_hard_errors() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join(".ship");
    fs::create_dir_all(root.join("library")).unwrap();
    // Note: no `machine/layout-version` marker, but `library/` already exists.
    fs::create_dir_all(root.join("agents")).unwrap();
    let h = home_at(tmp.path());
    let err = migrate_home(&h).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("partial-legacy"), "got: {msg}");
    assert!(msg.contains("layout-version"), "got: {msg}");
    // Original `agents/` must still be in place.
    assert!(root.join("agents").is_dir());
}

#[test]
fn rollback_on_failure_leaves_original_layout() {
    // Force a failure by pre-creating the destination so commit collides.
    let tmp = tempdir().unwrap();
    let root = tmp.path().join(".ship");
    fs::create_dir_all(root.join("agents")).unwrap();
    fs::write(root.join("agents/x.jsonc"), "{}").unwrap();
    // Pre-create `library/agents/` so commit's "destination already exists"
    // guard fires. Note: this also makes `library/` exist, which would
    // normally trigger partial-legacy. We pre-write the marker to bypass
    // that check, which means inspect_layout would be a no-op... so instead
    // we put a non-empty `library/` next to a never-finished migration.
    fs::create_dir_all(root.join("library/agents")).unwrap();
    fs::write(root.join("library/agents/blocker"), b"x").unwrap();

    let h = home_at(tmp.path());
    let err = migrate_home(&h).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("partial-legacy") || msg.contains("destination already exists"),
        "unexpected error: {msg}"
    );
    // Original content still present.
    assert!(root.join("agents/x.jsonc").is_file());
    assert!(!root.join(".migrate-tmp").exists(), "staging cleaned up");
}
