use super::*;
use std::fs;
use tempfile::tempdir;

fn setup() -> (tempfile::TempDir, PathBuf, String) {
    let tmp = tempdir().unwrap();
    let trash_root = tmp.path().join("trash");
    let ws = "test-workspace".to_string();
    (tmp, trash_root, ws)
}

#[test]
fn trash_file_copies_to_trash_dir() {
    let (tmp, trash_root, ws) = setup();
    let file = tmp.path().join("original.txt");
    fs::write(&file, "hello").unwrap();

    let dest = trash_file_at(&trash_root, &ws, &file).unwrap();
    assert!(dest.exists());
    assert_eq!(fs::read_to_string(&dest).unwrap(), "hello");
    // Original still exists (trash copies, doesn't move).
    assert!(file.exists());
}

#[test]
fn trash_nonexistent_file_fails() {
    let (_tmp, trash_root, ws) = setup();
    let result = trash_file_at(&trash_root, &ws, Path::new("/nonexistent/file.txt"));
    assert!(result.is_err());
}

#[test]
fn list_trash_returns_entries_newest_first() {
    let (tmp, trash_root, ws) = setup();

    let f1 = tmp.path().join("a.txt");
    fs::write(&f1, "aaa").unwrap();
    trash_file_at(&trash_root, &ws, &f1).unwrap();

    let f2 = tmp.path().join("b.txt");
    fs::write(&f2, "bbb").unwrap();
    trash_file_at(&trash_root, &ws, &f2).unwrap();

    let entries = list_trash_at(&trash_root, &ws).unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries[0].trashed_at >= entries[1].trashed_at);
}

#[test]
fn list_trash_empty_workspace() {
    let (_tmp, trash_root, ws) = setup();
    let entries = list_trash_at(&trash_root, &ws).unwrap();
    assert!(entries.is_empty());
}

#[test]
fn restore_file_to_original_location() {
    let (tmp, trash_root, ws) = setup();
    let file = tmp.path().join("restore-me.txt");
    fs::write(&file, "restore content").unwrap();
    trash_file_at(&trash_root, &ws, &file).unwrap();

    // Delete original.
    fs::remove_file(&file).unwrap();
    assert!(!file.exists());

    let restored = restore_file_at(&trash_root, &ws, "restore-me.txt", None).unwrap();
    assert_eq!(restored, file);
    assert_eq!(fs::read_to_string(&file).unwrap(), "restore content");
}

#[test]
fn restore_file_to_custom_dest() {
    let (tmp, trash_root, ws) = setup();
    let file = tmp.path().join("src.txt");
    fs::write(&file, "data").unwrap();
    trash_file_at(&trash_root, &ws, &file).unwrap();

    let custom_dest = tmp.path().join("restored/elsewhere.txt");
    let restored =
        restore_file_at(&trash_root, &ws, "src.txt", Some(&custom_dest)).unwrap();
    assert_eq!(restored, custom_dest);
    assert_eq!(fs::read_to_string(&custom_dest).unwrap(), "data");
}

#[test]
fn restore_nonexistent_name_fails() {
    let (_tmp, trash_root, ws) = setup();
    let result = restore_file_at(&trash_root, &ws, "nope.txt", None);
    assert!(result.is_err());
}

#[test]
fn purge_old_trash_removes_expired() {
    let (tmp, trash_root, ws) = setup();

    // Create a fake old trash entry.
    let old_dir = trash_root.join(&ws).join("20200101-000000.000");
    fs::create_dir_all(&old_dir).unwrap();
    fs::write(old_dir.join("old.txt"), "old").unwrap();
    fs::write(
        old_dir.join(".manifest"),
        r#"{"original_path":"/tmp/old.txt","trashed_at":"2020-01-01T00:00:00Z"}"#,
    )
    .unwrap();

    // Create a fresh entry.
    let file = tmp.path().join("fresh.txt");
    fs::write(&file, "fresh").unwrap();
    trash_file_at(&trash_root, &ws, &file).unwrap();

    let removed = purge_old_trash_at(&trash_root, Duration::from_secs(86400)).unwrap();
    assert_eq!(removed, 1);
    assert!(!old_dir.exists());

    // Fresh entry still exists.
    let entries = list_trash_at(&trash_root, &ws).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn purge_no_trash_dir_returns_zero() {
    let (_tmp, trash_root, _ws) = setup();
    let removed = purge_old_trash_at(&trash_root, Duration::from_secs(86400)).unwrap();
    assert_eq!(removed, 0);
}
