//! LocalFsBroker confinement + durability suite.

use super::fs_broker::{FsBroker, FsCap, FsError, FsMode, FsRoot};
use super::local_fs_broker::LocalFsBroker;

fn cap(mode: FsMode) -> (FsCap, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    (FsCap::new_unchecked(dir.path(), FsRoot::Device, mode), dir)
}

#[test]
fn rel_within_confinement_ok() {
    let broker = LocalFsBroker::new();
    let (cap, _dir) = cap(FsMode::ReadWrite);
    broker.write_atomic(&cap, "sub/dir/file.txt", b"data").unwrap();
    assert!(broker.exists(&cap, "sub/dir/file.txt").unwrap());
    assert_eq!(broker.read(&cap, "sub/dir/file.txt").unwrap(), b"data");
    assert_eq!(
        broker.read_string(&cap, "sub/dir/file.txt").unwrap(),
        "data"
    );
}

#[test]
fn path_escape_rejected() {
    let broker = LocalFsBroker::new();
    let (cap, _dir) = cap(FsMode::ReadWrite);
    for bad in ["../escape", "a/../../escape", "/etc/passwd"] {
        match broker.read(&cap, bad) {
            Err(FsError::PathEscape) => {}
            other => panic!("expected PathEscape for {bad:?}, got {other:?}"),
        }
    }
}

#[test]
fn read_only_rejects_write() {
    let broker = LocalFsBroker::new();
    let (cap, _dir) = cap(FsMode::ReadOnly);
    match broker.write_atomic(&cap, "file.txt", b"x") {
        Err(FsError::ReadOnlyViolation) => {}
        other => panic!("expected ReadOnlyViolation, got {other:?}"),
    }
    match broker.create_dir_all(&cap, "d") {
        Err(FsError::ReadOnlyViolation) => {}
        other => panic!("expected ReadOnlyViolation, got {other:?}"),
    }
}

#[test]
fn write_atomic_durability() {
    let broker = LocalFsBroker::new();
    let (cap, dir) = cap(FsMode::ReadWrite);
    broker.write_atomic(&cap, "durable.txt", b"v1").unwrap();
    // Overwrite atomically; the file is always a complete version, never a
    // leftover temp file in the directory.
    broker.write_atomic(&cap, "durable.txt", b"v2-longer").unwrap();
    assert_eq!(broker.read(&cap, "durable.txt").unwrap(), b"v2-longer");
    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "no temp files should remain");
}

#[test]
fn create_new_rejects_existing() {
    let broker = LocalFsBroker::new();
    let (cap, _dir) = cap(FsMode::ReadWrite);
    broker.create_new(&cap, "once.txt", b"a").unwrap();
    match broker.create_new(&cap, "once.txt", b"b") {
        Err(FsError::AlreadyExists) => {}
        other => panic!("expected AlreadyExists, got {other:?}"),
    }
}

#[test]
fn remove_and_list_and_abs() {
    let broker = LocalFsBroker::new();
    let (cap, dir) = cap(FsMode::ReadWrite);
    broker.write_atomic(&cap, "d/a.txt", b"a").unwrap();
    broker.write_atomic(&cap, "d/b.txt", b"b").unwrap();
    let mut listed = broker.list_dir(&cap, "d").unwrap();
    listed.sort();
    assert_eq!(listed.len(), 2);
    assert_eq!(broker.abs_path(&cap, "d/a.txt").unwrap(), dir.path().join("d/a.txt"));
    broker.remove(&cap, "d/a.txt").unwrap();
    assert!(!broker.exists(&cap, "d/a.txt").unwrap());
}

#[test]
fn link_into_requires_writable_dst() {
    let broker = LocalFsBroker::new();
    let src_dir = tempfile::tempdir().unwrap();
    let dst_dir = tempfile::tempdir().unwrap();
    let src = FsCap::new_unchecked(src_dir.path(), FsRoot::Device, FsMode::ReadOnly);
    let dst_ro = FsCap::new_unchecked(dst_dir.path(), FsRoot::Device, FsMode::ReadOnly);
    let dst_rw = FsCap::new_unchecked(dst_dir.path(), FsRoot::Device, FsMode::ReadWrite);
    broker.write_atomic(
        &FsCap::new_unchecked(src_dir.path(), FsRoot::Device, FsMode::ReadWrite),
        "f.txt",
        b"shared",
    )
    .unwrap();
    match broker.link_into(&src, "f.txt", &dst_ro, "linked.txt") {
        Err(FsError::ReadOnlyViolation) => {}
        other => panic!("expected ReadOnlyViolation, got {other:?}"),
    }
    broker.link_into(&src, "f.txt", &dst_rw, "linked.txt").unwrap();
    assert_eq!(broker.read(&dst_rw, "linked.txt").unwrap(), b"shared");
}
