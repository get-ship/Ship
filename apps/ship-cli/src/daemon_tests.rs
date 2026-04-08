use super::{read_pid, write_pid};
use tempfile::TempDir;

#[test]
fn test_write_and_read_pid() {
    let dir = TempDir::new().unwrap();
    let pid_path = dir.path().join("daemon.pid");

    write_pid(&pid_path, 12345).unwrap();
    assert!(pid_path.exists());

    let pid = read_pid(&pid_path).unwrap();
    assert_eq!(pid, Some(12345));
}

#[test]
fn test_read_pid_missing_file() {
    let dir = TempDir::new().unwrap();
    let pid_path = dir.path().join("daemon.pid");
    let pid = read_pid(&pid_path).unwrap();
    assert_eq!(pid, None);
}

#[test]
fn test_read_pid_invalid_content() {
    let dir = TempDir::new().unwrap();
    let pid_path = dir.path().join("daemon.pid");
    std::fs::write(&pid_path, "not-a-number\n").unwrap();
    assert!(read_pid(&pid_path).is_err());
}

#[test]
fn test_write_pid_creates_parent_dirs() {
    let dir = TempDir::new().unwrap();
    let pid_path = dir.path().join("nested").join("dirs").join("daemon.pid");
    write_pid(&pid_path, 99).unwrap();
    let pid = read_pid(&pid_path).unwrap();
    assert_eq!(pid, Some(99));
}
