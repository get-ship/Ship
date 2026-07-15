//! The `--unsafe-skip-auth` relaxed-auth flag.
//!
//! When set, a **loopback** peer is treated as an authenticated capped client
//! with no token — the explicit, deliberately alarming override for
//! contributors developing against the daemon. The relaxation is compiled out
//! of release builds (see `middleware`), so a shipped daemon can never relax
//! auth regardless of this flag.

use std::sync::atomic::{AtomicBool, Ordering};

static UNSAFE_SKIP_AUTH_MODE: AtomicBool = AtomicBool::new(false);

/// Enable/disable relaxed-auth mode. Set once at startup from `--unsafe-skip-auth`.
pub fn set_unsafe_skip_auth_mode(on: bool) {
    UNSAFE_SKIP_AUTH_MODE.store(on, Ordering::SeqCst);
}

/// True when the daemon was started with `--unsafe-skip-auth`.
pub fn is_unsafe_skip_auth_mode() -> bool {
    UNSAFE_SKIP_AUTH_MODE.load(Ordering::SeqCst)
}
