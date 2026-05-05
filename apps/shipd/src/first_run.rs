//! First-run UX — opens the setup wizard in the user's default browser
//! immediately after the daemon's HTTP listener binds on a fresh install.
//!
//! "Fresh install" means no `data.db` in the ship dir yet (the user-content
//! database is created only once we have real state to persist). On any
//! subsequent start the helper is a no-op.
//!
//! Suppressible via `SHIP_NO_BROWSER=1` for headless CI / service contexts.

use std::path::Path;

/// Public entry point: if this is a first-run and browser open isn't
/// suppressed, spawn the platform's URL opener pointing at `/setup`.
///
/// Never errors — failing to launch a browser is a UX nit, not a reason to
/// block the daemon. Any failure is logged and discarded.
pub fn maybe_open_setup(ship_dir: &Path, port: u16) {
    if std::env::var_os("SHIP_NO_BROWSER").is_some() {
        tracing::debug!("first-run browser suppressed by SHIP_NO_BROWSER");
        return;
    }
    if !is_first_run(ship_dir) {
        return;
    }
    let url = format!("http://127.0.0.1:{port}/setup");
    tracing::info!(url = %url, "first-run: opening setup wizard in browser");
    spawn_open(&url);
}

/// True when the user-content DB does not exist yet. We deliberately ignore
/// `platform.db` — that one is rewritten on every daemon boot (PID/port
/// bookkeeping) and its presence doesn't mean the user ever completed setup.
fn is_first_run(ship_dir: &Path) -> bool {
    !ship_substrate::db::data_db_path(ship_dir).exists()
}

#[cfg(target_os = "linux")]
fn spawn_open(url: &str) {
    spawn_detached("xdg-open", &[url]);
}

#[cfg(target_os = "macos")]
fn spawn_open(url: &str) {
    spawn_detached("open", &[url]);
}

#[cfg(target_os = "windows")]
fn spawn_open(url: &str) {
    // `start` is a cmd builtin, not an executable — invoke via cmd /C.
    // The empty "" is cmd's window-title slot so a URL with spaces doesn't
    // get misinterpreted as one.
    spawn_detached("cmd", &["/C", "start", "", url]);
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn spawn_open(_url: &str) {
    tracing::debug!("first-run: browser open not supported on this platform");
}

fn spawn_detached(program: &str, args: &[&str]) {
    let mut cmd = std::process::Command::new(program);
    cmd.args(args);
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    match cmd.spawn() {
        Ok(mut child) => {
            // Detach — we don't want a zombie if the opener exits after us.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => {
            tracing::warn!(program, error = %e, "first-run: failed to spawn browser opener");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn is_first_run_true_when_data_db_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(is_first_run(tmp.path()));
    }

    #[test]
    fn is_first_run_false_when_data_db_exists() {
        let tmp = TempDir::new().unwrap();
        let path = ship_substrate::db::data_db_path(tmp.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"").unwrap();
        assert!(!is_first_run(tmp.path()));
    }

    #[test]
    fn maybe_open_setup_is_noop_when_suppressed() {
        let tmp = TempDir::new().unwrap();
        // SAFETY: scoped to this test; SHIP_NO_BROWSER is a boolean flag, no
        // other thread in this crate depends on its exact value.
        unsafe { std::env::set_var("SHIP_NO_BROWSER", "1") };
        // Must not panic or block.
        maybe_open_setup(tmp.path(), 9315);
        unsafe { std::env::remove_var("SHIP_NO_BROWSER") };
    }
}
