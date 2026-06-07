use super::pidfile::{self, PidfileRecord, PidfileRole};
use super::supervision::{select_with, AdapterKind, SupervisionAdapter};
use super::supervision_pidfile::PidfileAdapter;
use super::supervision_systemd::{is_managing_unit_for_tests, SystemdAdapter};
use tempfile::TempDir;

fn sample_record(pid: u32) -> PidfileRecord {
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

fn record_with_version(pid: u32, version: &str) -> PidfileRecord {
    let mut r = sample_record(pid);
    r.version = version.into();
    r
}

#[test]
fn select_returns_pidfile_when_systemd_not_managing() {
    let adapter = select_with(false);
    assert_eq!(adapter.name(), AdapterKind::Pidfile);
}

#[test]
fn select_returns_systemd_when_systemd_is_managing() {
    let adapter = select_with(true);
    assert_eq!(adapter.name(), AdapterKind::Systemd);
}

#[test]
fn pidfile_adapter_status_when_no_pidfile() {
    let tmp = TempDir::new().unwrap();
    let status = PidfileAdapter.status(tmp.path()).unwrap();
    assert_eq!(status.adapter, AdapterKind::Pidfile);
    assert!(!status.running);
    assert!(status.pid.is_none());
    assert!(status.version.is_none());
    assert!(status.uptime_secs.is_none());
}

#[test]
fn pidfile_adapter_status_when_lock_held() {
    let tmp = TempDir::new().unwrap();
    let guard = match pidfile::acquire(tmp.path(), sample_record(4242)).unwrap() {
        pidfile::AcquireResult::Fresh(g) => g,
        _ => panic!("expected Fresh"),
    };

    let status = PidfileAdapter.status(tmp.path()).unwrap();
    assert_eq!(status.adapter, AdapterKind::Pidfile);
    assert!(status.running);
    assert_eq!(status.pid, Some(4242));
    assert_eq!(status.version.as_deref(), Some("test-v"));
    assert!(status.uptime_secs.is_some(), "started_at is RFC3339, uptime must compute");
    let keys: Vec<&str> = status.extras.iter().map(|(k, _)| k.as_str()).collect();
    assert!(keys.contains(&"started_at"));
    assert!(keys.contains(&"role"));

    drop(guard);
}

#[test]
fn pidfile_adapter_status_when_stale_file_no_lock() {
    let tmp = TempDir::new().unwrap();
    let path = pidfile::pidfile_path(tmp.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_vec_pretty(&sample_record(9001)).unwrap()).unwrap();

    let status = PidfileAdapter.status(tmp.path()).unwrap();
    assert!(!status.running, "no lock held -> not running");
    assert_eq!(status.pid, Some(9001));
    let lock_note = status
        .extras
        .iter()
        .find(|(k, _)| k == "lock")
        .map(|(_, v)| v.as_str());
    assert_eq!(lock_note, Some("released (stale pidfile)"));
}

#[test]
fn systemd_adapter_is_managing_returns_false_when_unit_missing() {
    // A name the test environment is guaranteed not to have registered.
    let managing = is_managing_unit_for_tests("ship-no-such-unit-xyz.service");
    assert!(
        !managing,
        "is-enabled must return false for nonexistent unit (or when systemctl absent)"
    );
}

#[test]
fn systemd_adapter_name_reports_systemd() {
    assert_eq!(SystemdAdapter.name(), AdapterKind::Systemd);
}

#[test]
fn version_check_skipped_when_no_live_daemon() {
    let tmp = TempDir::new().unwrap();
    // No pidfile exists → guard is a no-op, regardless of installed version.
    super::commands::check_version_mismatch(tmp.path()).expect("no pidfile → ok");
}

#[test]
fn version_check_skipped_when_pidfile_stale() {
    // Pidfile on disk but no lock held — supervisor already died. A
    // subsequent `start` should proceed; the new supervisor will take
    // over the stale pidfile. This mirrors prior pidfile semantics.
    let tmp = TempDir::new().unwrap();
    let path = pidfile::pidfile_path(tmp.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&record_with_version(9001, "0.99.0")).unwrap(),
    )
    .unwrap();

    super::commands::check_version_mismatch(tmp.path())
        .expect("stale pidfile (no lock) → ok, takeover is legal");
}

#[test]
fn version_mismatch_refuses_start() {
    let tmp = TempDir::new().unwrap();
    // Acquire the lock with a deliberately foreign version. The guard
    // must refuse and name both versions in the error.
    let rec = record_with_version(4242, "0.99.0-fake");
    let guard = match pidfile::acquire(tmp.path(), rec).unwrap() {
        pidfile::AcquireResult::Fresh(g) => g,
        _ => panic!("expected Fresh"),
    };

    let err = super::commands::check_version_mismatch(tmp.path())
        .expect_err("version mismatch must Err");
    let msg = format!("{err}");
    assert!(msg.contains("version mismatch"), "msg: {msg}");
    assert!(msg.contains("0.99.0-fake"), "msg should cite running version: {msg}");
    assert!(msg.contains(env!("CARGO_PKG_VERSION")), "msg should cite installed version: {msg}");
    assert!(msg.contains("ship daemon restart"), "msg should suggest remediation: {msg}");

    drop(guard);
}

#[test]
fn version_match_allows_start() {
    let tmp = TempDir::new().unwrap();
    let rec = record_with_version(4242, env!("CARGO_PKG_VERSION"));
    let guard = match pidfile::acquire(tmp.path(), rec).unwrap() {
        pidfile::AcquireResult::Fresh(g) => g,
        _ => panic!("expected Fresh"),
    };

    super::commands::check_version_mismatch(tmp.path())
        .expect("matching version → ok");

    drop(guard);
}

/// End-to-end spawn of the pidfile adapter is not run by default — it
/// would fork a real shipd supervisor against the user's `~/.ship/`.
/// Kept ignored so anyone debugging routing can opt in with
/// `cargo test -- --ignored pidfile_adapter_start_stop_round_trip`.
#[test]
#[ignore]
fn pidfile_adapter_start_stop_round_trip() {
    use super::supervision::StartContext;
    let ctx = StartContext {
        host: "127.0.0.1".to_string(),
        dev: true,
        global_dir: ship_substrate::ShipHome::from_env().root().to_path_buf(),
    };
    PidfileAdapter.start(&ctx).expect("start");
    PidfileAdapter.stop().expect("stop");
}
