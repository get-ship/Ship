use super::pid_io::{read_pid, write_pid};
use super::status::{format_relative_time, format_uptime};
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

#[test]
fn format_uptime_buckets() {
    assert_eq!(format_uptime(45), "45s");
    assert_eq!(format_uptime(125), "2m 5s");
    assert_eq!(format_uptime(3725), "1h 2m");
    assert_eq!(format_uptime(86400 * 2 + 3600), "2d 1h");
}

#[test]
fn format_relative_time_seconds() {
    let t = (chrono::Utc::now() - chrono::Duration::seconds(8)).to_rfc3339();
    let s = format_relative_time(&t);
    assert!(s.ends_with(" ago"));
    assert!(s.contains('s'));
}

#[test]
fn format_relative_time_minutes() {
    let t = (chrono::Utc::now() - chrono::Duration::minutes(12)).to_rfc3339();
    let s = format_relative_time(&t);
    assert!(s.ends_with("m ago"));
}

#[test]
fn format_relative_time_invalid_input() {
    assert_eq!(format_relative_time("not-a-date"), "not-a-date");
}
