//! Test-only helpers for environment isolation.
//!
//! Several tests mutate `SHIP_HOME` to point at a tempdir. Without
//! coordination they race each other (parallel test runner) and leak
//! state into later tests. `isolated_ship_home` serializes via
//! `SHIP_HOME_LOCK` and restores the prior value on drop.
//!
//! Module-level `#[cfg(test)]` is applied at the declaring site
//! (`main.rs`); no inner `#![cfg(test)]` needed.

use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard};

pub static SHIP_HOME_LOCK: Mutex<()> = Mutex::new(());

/// Holds the lock + the previous `SHIP_HOME` so it can be restored on
/// drop. Ensures one test never sees another test's tempdir.
pub struct ShipHomeGuard {
    _guard: MutexGuard<'static, ()>,
    previous: Option<OsString>,
}

impl Drop for ShipHomeGuard {
    fn drop(&mut self) {
        // SAFETY: serialized via SHIP_HOME_LOCK; held until drop.
        unsafe {
            match self.previous.take() {
                Some(v) => std::env::set_var("SHIP_HOME", v),
                None => std::env::remove_var("SHIP_HOME"),
            }
        }
    }
}

/// Pin `SHIP_HOME` to `path` for the lifetime of the returned guard.
pub fn isolated_ship_home(path: &std::path::Path) -> ShipHomeGuard {
    let guard = SHIP_HOME_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let previous = std::env::var_os("SHIP_HOME");
    // SAFETY: serialized via SHIP_HOME_LOCK; held by guard until drop.
    unsafe { std::env::set_var("SHIP_HOME", path) };
    ShipHomeGuard { _guard: guard, previous }
}
