use super::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn sample_record(pid: u32) -> PidfileRecord {
    PidfileRecord {
        pid,
        port: 9315,
        version: "test".into(),
        commit_sha: "deadbeef".into(),
        started_at: "2026-04-16T00:00:00Z".into(),
        pidfile_path: String::new(),
        socket_path: String::new(),
        role: PidfileRole::Supervisor,
    }
}

#[test]
fn acquire_succeeds_when_no_pidfile() {
    let tmp = TempDir::new().unwrap();
    let guard = acquire(tmp.path(), sample_record(42))
        .unwrap()
        .expect("should acquire");

    // File exists and contains our record.
    let (record, lock_held) = read(tmp.path()).unwrap().expect("should read");
    assert_eq!(record.pid, 42);
    assert_eq!(record.port, 9315);
    assert_eq!(record.role, PidfileRole::Supervisor);
    assert_eq!(record.pidfile_path, pidfile_path(tmp.path()).display().to_string());
    assert!(lock_held, "lock should be held while guard alive");

    drop(guard);
}

#[test]
fn acquire_returns_none_when_lock_held() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().to_path_buf();

    // Hold the lock on a background thread; signal when ready.
    let (ready_tx, ready_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let holder_path = path.clone();
    let holder = thread::spawn(move || {
        let guard = acquire(&holder_path, sample_record(1001))
            .unwrap()
            .expect("holder should acquire");
        ready_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        drop(guard);
    });

    ready_rx.recv().unwrap();

    // Second acquire should see the lock held.
    let result = acquire(&path, sample_record(2002)).unwrap();
    assert!(result.is_none(), "second acquire must return None when lock held");

    // The record on disk still belongs to the holder.
    let (record, lock_held) = read(&path).unwrap().expect("should read");
    assert_eq!(record.pid, 1001);
    assert!(lock_held);

    release_tx.send(()).unwrap();
    holder.join().unwrap();
}

#[test]
fn acquire_takes_over_stale_pidfile() {
    let tmp = TempDir::new().unwrap();
    let path = pidfile_path(tmp.path());

    // Write a pidfile as if a prior process had crashed — no lock held,
    // but file remains.
    std::fs::create_dir_all(tmp.path()).unwrap();
    let stale = sample_record(9999);
    std::fs::write(&path, serde_json::to_vec_pretty(&stale).unwrap()).unwrap();

    // Acquire should succeed and overwrite the stale contents.
    let guard = acquire(tmp.path(), sample_record(1234))
        .unwrap()
        .expect("should take over stale pidfile");

    let (record, lock_held) = read(tmp.path()).unwrap().expect("should read");
    assert_eq!(record.pid, 1234, "stale pid must be overwritten");
    assert!(lock_held);

    drop(guard);
}

#[test]
fn read_returns_none_when_no_file() {
    let tmp = TempDir::new().unwrap();
    assert!(read(tmp.path()).unwrap().is_none());
}

#[test]
fn read_returns_record_and_lock_status_when_lock_released() {
    let tmp = TempDir::new().unwrap();
    let path = pidfile_path(tmp.path());

    // Write a record without holding the lock.
    std::fs::create_dir_all(tmp.path()).unwrap();
    let rec = sample_record(7777);
    std::fs::write(&path, serde_json::to_vec_pretty(&rec).unwrap()).unwrap();

    let (record, lock_held) = read(tmp.path()).unwrap().expect("should read");
    assert_eq!(record.pid, 7777);
    assert!(!lock_held, "lock should not be held when nobody acquired");
}

#[test]
fn read_reports_lock_held_while_guard_alive() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().to_path_buf();

    let (ready_tx, ready_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let holder_path = path.clone();
    let holder = thread::spawn(move || {
        let guard = acquire(&holder_path, sample_record(5555))
            .unwrap()
            .expect("holder should acquire");
        ready_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        drop(guard);
    });
    ready_rx.recv().unwrap();

    let (record, lock_held) = read(&path).unwrap().expect("should read");
    assert_eq!(record.pid, 5555);
    assert!(lock_held);

    release_tx.send(()).unwrap();
    holder.join().unwrap();
}

#[test]
fn drop_guard_releases_lock() {
    let tmp = TempDir::new().unwrap();

    let guard = acquire(tmp.path(), sample_record(111))
        .unwrap()
        .expect("first acquire");
    drop(guard);

    // Give the OS a moment to release.
    thread::sleep(Duration::from_millis(10));

    let guard2 = acquire(tmp.path(), sample_record(222))
        .unwrap()
        .expect("second acquire after drop");
    let (record, _) = read(tmp.path()).unwrap().expect("should read");
    assert_eq!(record.pid, 222);
    drop(guard2);
}

#[test]
fn drop_guard_removes_file() {
    let tmp = TempDir::new().unwrap();
    let path = pidfile_path(tmp.path());

    let guard = acquire(tmp.path(), sample_record(333)).unwrap().unwrap();
    assert!(path.exists());
    drop(guard);
    assert!(!path.exists(), "drop should remove the pidfile");
}
