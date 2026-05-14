//! Tests for `daemon_stop_at` — the stop path targets the supervisor via
//! the flock-based pidfile only. The legacy `daemon.pid` fallback was
//! removed: pre-v1 has no installed state to protect.

use super::daemon_stop_at;
use super::super::pidfile::{self, PidfileRecord, PidfileRole};
use std::io::Write;
use tempfile::TempDir;

fn sample_supervisor_record(pid: u32) -> PidfileRecord {
    PidfileRecord {
        pid,
        port: 9315,
        version: "test-v".into(),
        commit_sha: "cafef00d".into(),
        started_at: chrono::Utc::now().to_rfc3339(),
        pidfile_path: String::new(),
        socket_path: String::new(),
        role: PidfileRole::Supervisor,
    }
}

/// Idempotent: stop on an already-stopped daemon is a no-op that exits 0.
#[test]
fn stop_is_noop_when_no_supervisor_and_no_pidfile() {
    let dir = TempDir::new().unwrap();
    daemon_stop_at(dir.path()).expect("stop on empty dir should succeed");
    assert!(!pidfile::pidfile_path(dir.path()).exists());
}

/// When the flock pidfile exists but its lock is not held (prior supervisor
/// crashed), stop is a no-op — there is no legacy fallback to consult.
#[test]
fn stop_is_noop_when_pidfile_lock_released() {
    let dir = TempDir::new().unwrap();

    let pidfile_path = pidfile::pidfile_path(dir.path());
    std::fs::create_dir_all(pidfile_path.parent().unwrap()).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&pidfile_path)
        .unwrap();
    let record = sample_supervisor_record(999_999);
    file.write_all(&serde_json::to_vec_pretty(&record).unwrap()).unwrap();
    file.write_all(b"\n").unwrap();
    drop(file);

    daemon_stop_at(dir.path()).expect("released pidfile must be a no-op");
}
