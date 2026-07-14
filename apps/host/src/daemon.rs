//! Daemon lifecycle — the shell supervises the *existing* `ship` daemon rather
//! than embedding a second copy. One daemon implementation, one IPC handshake;
//! the shell is just another (operator) client. Discovery/handshake reading is
//! reused from `ship-substrate` so the shell never re-expresses those layouts.

use serde::Serialize;
use std::time::Duration;

/// What the webview needs to point its Connect client at the right loopback
/// endpoint and to render daemon status. No token here — the token reaches the
/// webview through the operator-session bootstrap, not this status call.
#[derive(Debug, Clone, Serialize)]
pub struct DaemonInfo {
    /// True when a handshake exists AND its recorded pid is alive.
    pub running: bool,
    /// Loopback port the Connect client should target, when known.
    pub port: Option<u16>,
    pub pid: Option<u32>,
    /// Host socket path, present only when the daemon has bound it (agent/CLI
    /// transport — the shell itself uses loopback, not this).
    pub socket: Option<String>,
    /// IPC directory the daemon serving the shell's cwd reads/writes.
    pub dir: String,
}

fn from_handle(h: ship_substrate::home::DaemonHandle) -> DaemonInfo {
    let sock = h.dir.join("ship.sock");
    DaemonInfo {
        running: h.is_alive(),
        port: Some(h.port),
        pid: h.pid,
        socket: sock.exists().then(|| sock.display().to_string()),
        dir: h.dir.display().to_string(),
    }
}

fn cwd() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Reap a spawned child on a detached thread so it doesn't become a zombie of
/// the shell process. A `Child` that's dropped without `wait`/`try_wait` leaves
/// a defunct entry in the process table on Unix until the parent exits; here the
/// parent (the long-lived shell) may never exit, so we `wait` explicitly. The
/// thread blocks only until the child does exit, then ends.
pub(crate) fn reap(mut child: std::process::Child) {
    std::thread::spawn(move || {
        let _ = child.wait();
    });
}

/// Current daemon status for the shell's working directory. Read-only; never
/// starts anything.
#[tauri::command]
pub fn daemon_status() -> DaemonInfo {
    let here = cwd();
    match ship_substrate::home::discover_running_daemon(&here) {
        Some(h) => from_handle(h),
        None => DaemonInfo {
            running: false,
            port: None,
            pid: None,
            socket: None,
            dir: ship_substrate::home::daemon_dir_for(&here).display().to_string(),
        },
    }
}

/// Canonical binary name of the ship CLI the host shells out to. We look for
/// this next to the host executable first (the install bundle ships `ship`
/// alongside `ship-host` so a launched desktop app — where `PATH` rarely
/// includes `/usr/local/bin` or the user's bin dir — can still find it), then
/// fall back to `PATH`. `.exe` is appended on Windows; bare on Unix.
#[cfg(windows)]
const SHIP_BINARY_NAME: &str = "ship.exe";
#[cfg(not(windows))]
const SHIP_BINARY_NAME: &str = "ship";

/// Resolve the `ship` CLI path: host-executable directory first, then nothing
/// (caller falls back to `Command::new("ship")` which searches `PATH`). Pure
/// — takes the directory to look in as an argument rather than reading
/// `current_exe()` itself, so the resolver is unit-testable without touching
/// the process state.
///
/// Returns `Some(path)` only if the candidate file actually exists on disk;
/// an absent host-side `ship` is not an error, it just means PATH is the
/// right next stop. A dangling symlink counts as missing (`is_file` follows
/// symlinks and reports false when the target is gone).
fn resolve_ship_in(host_exe_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let candidate = host_exe_dir.join(SHIP_BINARY_NAME);
    candidate.is_file().then_some(candidate)
}

/// Build the `Command` that runs `ship <args>`: prefers the host-side `ship`
/// shipped next to this binary, falls back to `PATH` resolution. Used by
/// `daemon_start` so the launched desktop app (whose `PATH` usually doesn't
/// include `/usr/local/bin`) can still bring up the daemon.
fn ship_command(host_exe_dir: &std::path::Path, args: &[&str]) -> std::process::Command {
    match resolve_ship_in(host_exe_dir) {
        Some(p) => {
            let mut c = std::process::Command::new(p);
            c.args(args);
            c
        }
        None => {
            let mut c = std::process::Command::new(SHIP_BINARY_NAME);
            c.args(args);
            c
        }
    }
}

/// Start the daemon by delegating to the `ship` CLI (`ship daemon start`), then
/// poll the handshake until it's alive or we time out (~5s). Delegating keeps a
/// single daemon-launch path; the shell does not reimplement supervision.
///
/// The CLI is resolved from the host executable's directory first (the
/// bundled install ships `ship` next to `ship-host`); PATH is the fallback
/// when the host-side binary isn't present — covers `cargo run` in dev and
/// source-tree invocations.
#[tauri::command]
pub fn daemon_start() -> Result<DaemonInfo, String> {
    if let Some(h) = ship_substrate::home::discover_running_daemon(&cwd()) {
        if h.is_alive() {
            return Ok(from_handle(h));
        }
    }
    let host_exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let child = ship_command(&host_exe_dir, &["daemon", "start"])
        .spawn()
        .map_err(|e| format!("spawning `ship daemon start`: {e}"))?;
    // Optimistic re-check: the `ship daemon start` CLI is short-lived and may
    // have already exited before we got here (e.g. it was already running
    // and just printed status). Bail out early with the live handle rather
    // than blocking the full poll loop.
    if let Some(h) = ship_substrate::home::discover_running_daemon(&cwd()) {
        if h.is_alive() {
            reap(child);
            return Ok(from_handle(h));
        }
    }
    // `ship daemon start` is short-lived — it launches/hands off the daemon and
    // exits. Reap it on a detached thread so it doesn't linger as a zombie of
    // the shell process (an un-waited `Child` accumulates a defunct entry on
    // Unix). The reaper outlives this call; the daemon itself is independent.
    reap(child);

    for _ in 0..50 {
        if let Some(h) = ship_substrate::home::discover_running_daemon(&cwd()) {
            if h.is_alive() {
                return Ok(from_handle(h));
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("daemon did not report a live handshake within 5s".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::home::DaemonHandle;
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// Process-wide lock for the daemon_start error-path test. The test
    /// mutates `PATH` and the cwd — both process-global — so the mutex
    /// serialises its body against any concurrent run of the same test.
    /// No other test in the crate mutates process-global state (the
    /// launcher tests inject their inputs), so the lock is local to this
    /// module.
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn handle_in(dir: &std::path::Path, pid: Option<u32>) -> DaemonHandle {
        DaemonHandle {
            dir: dir.to_path_buf(),
            port: 9315,
            token: "tok".into(),
            pid,
            workspace: None,
            build_sha: None,
        }
    }
    #[test]
    fn from_handle_maps_fields_and_alive_pid() {
        let tmp = TempDir::new().unwrap();
        // Our own pid is alive by construction (we're running this test).
        let info = from_handle(handle_in(tmp.path(), Some(std::process::id())));
        assert!(info.running, "live pid should map to running=true");
        assert_eq!(info.port, Some(9315));
        assert_eq!(info.pid, Some(std::process::id()));
        assert_eq!(info.dir, tmp.path().display().to_string());
    }

    #[test]
    fn from_handle_running_false_without_pid() {
        let tmp = TempDir::new().unwrap();
        let info = from_handle(handle_in(tmp.path(), None));
        assert!(!info.running, "no pid → not running");
        assert_eq!(info.pid, None);
    }

    #[test]
    fn from_handle_socket_reflects_ship_sock_presence() {
        let tmp = TempDir::new().unwrap();
        // Absent until the daemon binds it.
        assert!(from_handle(handle_in(tmp.path(), None)).socket.is_none());
        // Present once `ship.sock` exists in the daemon dir.
        let sock = tmp.path().join("ship.sock");
        std::fs::write(&sock, b"").unwrap();
        assert_eq!(
            from_handle(handle_in(tmp.path(), None)).socket,
            Some(sock.display().to_string())
        );
    }

    /// Run a single-threaded gated block of daemon_start error-path tests.
    /// The test is *serial* — it mutates `PATH` and the cwd, which are
    /// process-global. A static `Mutex` makes cargo's parallel test runner
    /// serialize this test against any concurrent run of itself. The mutex is
    /// taken for the whole body and held until the very last assertion;
    /// cargo's scheduler sees the guard on the way in and waits.
    ///
    /// The happy path of `daemon_start` is *not* exercised here: it requires
    /// a real `ship` binary that brings up a daemon, which is what
    /// `tests/contract` and the launch smoke test cover. Unit coverage
    /// stays on the two error contracts:
    ///   1. `ship` missing on PATH → surface the spawn error
    ///   2. `ship` exits 0 but no handshake ever appears → 5s timeout Err
    #[test]
    fn daemon_start_error_paths() {
        let _guard = TEST_ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let cwd = TempDir::new().unwrap();
        let prev_cwd = std::env::current_dir().ok();
        let prev_path = std::env::var_os("PATH");
        std::env::set_current_dir(cwd.path()).unwrap();

        // ---- (1) ship missing on PATH ------------------------------------
        let empty_path = TempDir::new().unwrap();
        std::env::set_var("PATH", empty_path.path());
        let err = daemon_start()
            .expect_err("expected daemon_start to fail when no `ship` is on PATH");
        assert!(
            err.to_lowercase().contains("ship") || err.to_lowercase().contains("spawn"),
            "error must name `ship` or `spawn`; got: {err:?}"
        );

        // ---- (2) ship exists but no handshake ever appears ---------------
        // A no-op `ship` shim exits 0 immediately. `daemon_start` reaps it
        // and then polls for a handshake that never arrives. The 5s
        // timeout contract is the load-bearing one: the function must
        // return an Err after the timeout, never panic. The test takes
        // ~5s; that's the cost of a meaningful test for this path.
        let bin_dir = TempDir::new().unwrap();
        let ship_shim = bin_dir.path().join("ship");
        std::fs::write(&ship_shim, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&ship_shim, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        std::env::set_var("PATH", bin_dir.path());
        let result = std::panic::catch_unwind(|| daemon_start());
        let outcome =
            result.expect("daemon_start must not panic when handshake never arrives");
        assert!(outcome.is_err(), "expected timeout Err, got {outcome:?}");
        let err = outcome.unwrap_err();
        assert!(
            err.contains("5s") || err.contains("handshake"),
            "error should mention the timeout/handshake, got: {err:?}"
        );

        // Restore.
        if let Some(prev) = prev_cwd {
            std::env::set_current_dir(prev).unwrap();
        }
        match prev_path {
            Some(v) => std::env::set_var("PATH", v),
            None => std::env::remove_var("PATH"),
        }
    }

    // -- ship resolver -----------------------------------------------------
    //
    // The host shell can't rely on `PATH` to find `ship` (a launched desktop
    // app's PATH usually doesn't include the user's bin dir). The resolver
    // looks in the host executable's directory first and falls back to PATH.
    // These tests pin the three branches:
    //   1. host-side `ship` present → Some(absolute path)
    //   2. host-side `ship` missing → None
    //   3. host-side `ship` is a directory / dangling symlink → None
    // and verify the `ship_command` builder wires whichever branch through to
    // `std::process::Command`'s program path.

    #[test]
    fn resolve_ship_in_finds_ship_next_to_host() {
        let tmp = TempDir::new().unwrap();
        let ship = tmp.path().join(SHIP_BINARY_NAME);
        std::fs::write(&ship, b"").unwrap();
        assert_eq!(resolve_ship_in(tmp.path()), Some(ship));
    }

    #[test]
    fn resolve_ship_in_returns_none_when_absent() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(resolve_ship_in(tmp.path()), None);
    }

    #[test]
    fn resolve_ship_in_rejects_directory_and_dangling_symlink() {
        let tmp = TempDir::new().unwrap();
        // A directory with the right name is NOT a file — resolver must skip
        // it. The install bundle ships a binary, never a directory named
        // `ship`.
        std::fs::create_dir(tmp.path().join(SHIP_BINARY_NAME)).unwrap();
        assert_eq!(
            resolve_ship_in(tmp.path()),
            None,
            "a directory named `ship` must not match the resolver"
        );
        // Remove the dir, drop a dangling symlink — `is_file()` follows the
        // symlink and reports false when the target is gone.
        std::fs::remove_dir(tmp.path().join(SHIP_BINARY_NAME)).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("nowhere-target", tmp.path().join(SHIP_BINARY_NAME)).unwrap();
        #[cfg(not(unix))]
        std::fs::write(tmp.path().join(SHIP_BINARY_NAME), b"").ok();
        assert_eq!(
            resolve_ship_in(tmp.path()),
            None,
            "a dangling symlink must not match the resolver"
        );
    }

    #[test]
    fn ship_command_uses_host_side_when_present() {
        let tmp = TempDir::new().unwrap();
        let ship = tmp.path().join(SHIP_BINARY_NAME);
        std::fs::write(&ship, b"").unwrap();
        let cmd = ship_command(tmp.path(), &["daemon", "start"]);
        // Resolved to the host-side binary, not the bare name on PATH.
        assert_eq!(cmd.get_program(), ship.as_os_str());
    }

    #[test]
    fn ship_command_falls_back_to_bare_name_on_path() {
        let tmp = TempDir::new().unwrap();
        // No file dropped in `tmp` — resolver returns None.
        let cmd = ship_command(tmp.path(), &["daemon", "start"]);
        // Falls back to `Command::new("ship")` — the program is the literal
        // binary name; PATH resolution happens at spawn time inside the OS.
        assert_eq!(cmd.get_program(), std::ffi::OsStr::new(SHIP_BINARY_NAME));
        // Args still threaded through.
        let args: Vec<&std::ffi::OsStr> = cmd.get_args().collect();
        assert_eq!(
            args,
            vec![std::ffi::OsStr::new("daemon"), std::ffi::OsStr::new("start")]
        );
    }

    // -- end ship resolver --------------------------------------------------
}
