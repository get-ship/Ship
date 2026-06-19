//! Global `~/.ship` directory and per-test isolation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow};

use super::SHIP_DIR_NAME;

const TEST_GLOBAL_DIR_PREFIX: &str = "ship-test-global-";

static TEST_GLOBAL_CLEANUP_PATH: OnceLock<PathBuf> = OnceLock::new();
#[cfg(unix)]
static TEST_GLOBAL_CLEANUP_REGISTERED: OnceLock<()> = OnceLock::new();

#[cfg(unix)]
unsafe extern "C" {
    fn atexit(cb: extern "C" fn()) -> i32;
    fn kill(pid: i32, sig: i32) -> i32;
}

/// Returns the global config directory (~/.ship)
pub fn get_global_dir() -> Result<PathBuf> {
    let is_test_ctx = cfg!(test)
        || env::var_os("SHIP_TEST").is_some()
        || std::env::current_exe()
            .ok()
            .as_deref()
            .is_some_and(is_likely_rust_test_binary);

    // Auto-isolation takes priority — prevents env-var races in parallel tests.
    if let Some(test_dir) = auto_test_global_dir() {
        fs::create_dir_all(&test_dir).with_context(|| {
            format!(
                "Failed to create auto-isolated test global dir at {}",
                test_dir.display()
            )
        })?;
        return Ok(test_dir);
    }

    if let Some(env_path) = crate::env::read(crate::env::Var::Home) {
        let path = PathBuf::from(env_path.trim());
        if !path.as_os_str().is_empty() {
            // Guard: reject production paths from test code. Developers must
            // either unset SHIP_HOME or point it at a temp directory.
            #[allow(clippy::panic, reason = "intentional fail-loud guard: a test must never write into ~/.ship")]
            if is_test_ctx && !path.starts_with(std::env::temp_dir()) {
                panic!(
                    "Test attempted to use production database via SHIP_HOME={}. \
                     Set SHIP_HOME to a temp directory in test setup.",
                    path.display()
                );
            }
            return Ok(path);
        }
    }

    // Safety net: if auto_test_global_dir() returned None but we are still in a
    // test context, refuse to return the production path.
    #[cfg(debug_assertions)]
    #[allow(clippy::panic, reason = "intentional fail-loud guard: test without isolation must not touch ~/.ship")]
    if is_test_ctx {
        panic!(
            "Test binary detected without explicit DB isolation. \
             Use `KernelRouter::new(ship_dir)` with an explicit path or set SHIP_HOME \
             to a temp directory in test setup."
        );
    }

    home::home_dir()
        .map(|h| h.join(SHIP_DIR_NAME))
        .ok_or_else(|| anyhow!("Could not find home directory"))
}

fn auto_test_global_dir() -> Option<PathBuf> {
    if std::env::var_os("SHIP_DISABLE_AUTO_TEST_GLOBAL_DIR").is_some() {
        return None;
    }

    let exe = std::env::current_exe().ok()?;
    if !is_likely_rust_test_binary(&exe) {
        return None;
    }

    thread_local! {
        static TEST_GLOBAL_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
    }
    let dir = TEST_GLOBAL_DIR.with(|cell| {
        let mut slot = cell.borrow_mut();
        slot.get_or_insert_with(|| {
            let thread_suffix = format!("{:?}", std::thread::current().id())
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>();
            std::env::temp_dir()
                .join(format!(
                    "{}{}-{}",
                    TEST_GLOBAL_DIR_PREFIX,
                    std::process::id(),
                    thread_suffix
                ))
                .join(SHIP_DIR_NAME)
        })
        .clone()
    });
    register_test_global_cleanup(&dir);
    cleanup_stale_test_global_dirs();
    Some(dir)
}

pub(crate) fn is_likely_rust_test_binary(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    // Match both standard target dirs and custom CARGO_TARGET_DIR layouts
    let in_test_deps = path_str.contains("/debug/deps/")
        || path_str.contains("/release/deps/")
        || path_str.contains("\\debug\\deps\\")
        || path_str.contains("\\release\\deps\\");
    if !in_test_deps {
        return false;
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains('-'))
}

#[cfg(unix)]
extern "C" fn cleanup_test_global_dir_on_exit() {
    // Remove all per-thread test dirs for this PID.
    let prefix = format!("{}{}-", TEST_GLOBAL_DIR_PREFIX, std::process::id());
    if let Ok(entries) = fs::read_dir(std::env::temp_dir()) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && name.starts_with(&prefix)
                && entry.path().is_dir()
            {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }
}

fn register_test_global_cleanup(path: &Path) {
    let _ = TEST_GLOBAL_CLEANUP_PATH.set(path.to_path_buf());
    #[cfg(unix)]
    if TEST_GLOBAL_CLEANUP_REGISTERED.get().is_none() {
        // SAFETY: registering a process-exit callback once is safe here; callback
        // does best-effort cleanup and ignores failures.
        let _ = unsafe { atexit(cleanup_test_global_dir_on_exit) };
        let _ = TEST_GLOBAL_CLEANUP_REGISTERED.set(());
    }
}

fn cleanup_stale_test_global_dirs() {
    let Ok(entries) = fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(pid) = parse_test_global_dir_pid(name) else {
            continue;
        };
        if pid == std::process::id() {
            continue;
        }
        if is_process_alive(pid) {
            continue;
        }
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

pub(crate) fn parse_test_global_dir_pid(name: &str) -> Option<u32> {
    name.strip_prefix(TEST_GLOBAL_DIR_PREFIX)?
        .split('-')
        .next()?
        .parse()
        .ok()
}

#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) is a read-only existence/permission probe.
    let rc = unsafe { kill(pid as i32, 0) };
    if rc == 0 {
        return true;
    }
    // ESRCH (3) => process does not exist.
    std::io::Error::last_os_error().raw_os_error() != Some(3)
}

#[cfg(not(unix))]
fn is_process_alive(_pid: u32) -> bool {
    false
}
