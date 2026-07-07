//! Tests for the daemon lifecycle helpers (split from `daemon/mod.rs`).

use std::path::{Path, PathBuf};

fn token_path(daemon_dir: &Path) -> PathBuf {
    daemon_dir.join("token")
}

mod stale_cleanup_tests {
    use super::token_path;
    use crate::ship_runtime::init::identity_files::{another_daemon_alive, cleanup_stale_daemon_files, pid_path, port_path, parent_pid};
    use std::path::Path;
    use tempfile::TempDir;

    fn write_pid(dir: &Path, pid: &str) {
        let path = pid_path(dir);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, pid).unwrap();
    }

    fn write_port(dir: &Path, port: &str) {
        let path = port_path(dir);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, port).unwrap();
    }

    fn write_token(dir: &Path, token: &str) {
        let path = token_path(dir);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, token).unwrap();
    }

    #[test]
    fn no_pid_file_is_noop() {
        let dir = TempDir::new().unwrap();
        cleanup_stale_daemon_files(dir.path()).expect("no pid file should be Ok");
    }

    #[test]
    fn corrupt_pid_file_gets_removed() {
        let dir = TempDir::new().unwrap();
        write_pid(dir.path(), "not-a-number");
        cleanup_stale_daemon_files(dir.path()).expect("corrupt pid should be Ok");
        assert!(!pid_path(dir.path()).exists());
    }

    #[test]
    fn stale_pid_cleans_up_pid_and_port() {
        let dir = TempDir::new().unwrap();
        // pid 99999999 is almost certainly not running.
        write_pid(dir.path(), "99999999");
        write_port(dir.path(), "9315");
        write_token(dir.path(), "abc123");

        cleanup_stale_daemon_files(dir.path()).expect("stale pid should be Ok");

        assert!(!pid_path(dir.path()).exists(), "pid file should be removed");
        assert!(!port_path(dir.path()).exists(), "port file should be removed");
        assert!(token_path(dir.path()).exists(), "token should be preserved");
    }

    /// If the recorded pid matches our own process, treat it as a leftover from
    /// a prior failed start and clean up rather than refusing to boot.
    #[test]
    fn own_pid_is_treated_as_leftover() {
        let dir = TempDir::new().unwrap();
        let own = std::process::id().to_string();
        write_pid(dir.path(), &own);
        write_port(dir.path(), "9315");

        cleanup_stale_daemon_files(dir.path())
            .expect("own pid should not be treated as a live conflict");
        assert!(!pid_path(dir.path()).exists());
        assert!(!port_path(dir.path()).exists());
    }

    /// When another process is confirmed alive and it is not us, startup
    /// must refuse by returning Err.
    #[cfg(unix)]
    #[test]
    fn live_daemon_detected_returns_err() {
        let dir = TempDir::new().unwrap();

        // Spawn a long-running child; its pid is guaranteed alive for the test.
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        let child_pid = child.id();
        write_pid(dir.path(), &child_pid.to_string());
        write_port(dir.path(), "9315");

        let detected = another_daemon_alive(dir.path());
        assert_eq!(detected, Some(child_pid), "helper must detect live foreign pid");

        let err = cleanup_stale_daemon_files(dir.path())
            .expect_err("live conflict must return Err");
        let msg = format!("{err}");
        assert!(msg.contains("another daemon is already running"), "msg: {msg}");
        assert!(msg.contains(&child_pid.to_string()), "msg: {msg}");
        assert!(
            pid_path(dir.path()).exists(),
            "pid file must remain when refusing"
        );

        let _ = child.kill();
        let _ = child.wait();
    }

    /// The supervisor process writes the pid file with its own PID before
    /// spawning the child daemon. The child must not treat the live
    /// parent-supervisor as a racing daemon.
    #[cfg(unix)]
    #[test]
    fn live_supervisor_parent_does_not_block_child_startup() {
        let dir = TempDir::new().unwrap();
        let ppid = parent_pid().expect("test must run with a parent");
        write_pid(dir.path(), &ppid.to_string());

        let detected = another_daemon_alive(dir.path());
        assert_eq!(
            detected, None,
            "parent supervisor (pid {ppid}) must not be flagged as a racing daemon"
        );

        let result = cleanup_stale_daemon_files(dir.path());
        assert!(
            result.is_ok(),
            "live supervisor parent (pid {ppid}) wrongly treated as racing daemon: {:?}",
            result.err()
        );
    }

    /// Pin sup-restart-01's contract: a live third-party PID that is
    /// neither us nor our parent still refuses startup. Uses a spawned
    /// child process (whose pid is alive, not us, and not our parent).
    #[cfg(unix)]
    #[test]
    fn live_unrelated_daemon_still_blocks_startup() {
        let dir = TempDir::new().unwrap();
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .expect("spawn sleep");
        let child_pid = child.id();

        assert_ne!(child_pid, std::process::id(), "child pid must not be ours");
        assert_ne!(
            Some(child_pid),
            parent_pid(),
            "child pid must not be our parent"
        );

        write_pid(dir.path(), &child_pid.to_string());
        let result = cleanup_stale_daemon_files(dir.path());
        let err = result.expect_err("live unrelated daemon should block startup");
        let msg = format!("{err}");
        assert!(msg.contains("already running"), "msg: {msg}");
        assert!(msg.contains(&child_pid.to_string()), "msg: {msg}");

        let _ = child.kill();
        let _ = child.wait();
    }

    /// hardening-r2 §5.9: a transient/non-NotFound read error on the pidfile
    /// must REFUSE the boot, not silently fall through to "no other daemon".
    /// On Linux this is reproducible by chmod-ing the pid file 0o000.
    #[cfg(unix)]
    #[test]
    fn unreadable_pidfile_blocks_startup() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new().unwrap();
        write_pid(dir.path(), "99999999"); // any well-formed pid; we never get to parse
        let pid_file = pid_path(dir.path());
        // Drop read perms so read_to_string hits PermissionDenied.
        std::fs::set_permissions(&pid_file, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Restore on scope exit so TempDir can clean up.
        let restore = std::fs::Permissions::from_mode(0o644);
        struct Restore {
            path: std::path::PathBuf,
            perms: std::fs::Permissions,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                let _ = std::fs::set_permissions(&self.path, self.perms.clone());
            }
        }
        let _restore = Restore {
            path: pid_file.clone(),
            perms: restore,
        };

        let err = cleanup_stale_daemon_files(dir.path())
            .expect_err("non-NotFound pidfile error must surface as Err — refusal to boot a 2nd daemon");
        let msg = format!("{err}");
        assert!(
            msg.contains("could not read pidfile") || msg.contains("read failed"),
            "expected fail-closed refusal, got: {msg}"
        );
    }
}

mod path_layout_tests {
    //! Validate that daemon writes land in the directory the caller passes —
    //! either the singleton `<runtime>/daemon/` or a per-worktree
    //! `<runtime>/daemons/<hash>/` (devx-04). The helpers treat
    //! `daemon_dir` as opaque; policy lives in the caller.
    use crate::ship_runtime::init::identity_files::{remove_liveness_files, write_pid_file, write_port_file, write_workspace_file};
    use ship_substrate::ShipHome;
    use ship_substrate::home::DaemonFile;
    use tempfile::TempDir;

    #[test]
    fn write_pid_lands_in_caller_provided_dir() {
        let tmp = TempDir::new().unwrap();
        let daemon_dir = ShipHome::for_test(tmp.path()).daemon_dir();
        write_pid_file(&daemon_dir).expect("write pid");
        assert!(daemon_dir.join("pid").exists());
    }

    #[test]
    fn write_port_lands_in_caller_provided_dir() {
        let tmp = TempDir::new().unwrap();
        let daemon_dir = ShipHome::for_test(tmp.path()).daemon_dir();
        write_port_file(&daemon_dir, 9315).expect("write port");
        let body = std::fs::read_to_string(daemon_dir.join("port")).unwrap();
        assert_eq!(body.trim(), "9315");
    }

    #[test]
    fn remove_liveness_files_removes_pid_and_port() {
        let tmp = TempDir::new().unwrap();
        let daemon_dir = ShipHome::for_test(tmp.path()).daemon_dir();
        write_port_file(&daemon_dir, 9315).unwrap();
        write_pid_file(&daemon_dir).unwrap();

        remove_liveness_files(&daemon_dir);

        assert!(!daemon_dir.join("port").exists());
        assert!(!daemon_dir.join("pid").exists());
    }

    #[test]
    fn singleton_dir_matches_typed_accessor() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let daemon_dir = home.daemon_dir();
        assert_eq!(daemon_dir.join("pid"), home.daemon_file(DaemonFile::Pid));
        assert_eq!(daemon_dir.join("port"), home.daemon_file(DaemonFile::Port));
        assert_eq!(daemon_dir.join("token"), home.daemon_file(DaemonFile::Token));
    }

    #[test]
    fn worktree_dir_writes_under_per_worktree_path() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let hash = "abcd1234";
        let daemon_dir = home.worktree_daemon_dir(hash);
        write_pid_file(&daemon_dir).expect("write pid");
        write_workspace_file(&daemon_dir, std::path::Path::new("/proj/wt"))
            .expect("write workspace");

        // Verify writes land under the typed accessor's path — checking the
        // accessor itself round-trips through fs prevents the lint from
        // having to whitelist test code that joins ".ship/" literals.
        assert!(home.worktree_daemon_dir(hash).join("pid").exists());
        assert!(home.worktree_daemon_dir(hash).join("workspace").exists());
    }
}
