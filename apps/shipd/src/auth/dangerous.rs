//! The `--dangerous` relaxed-auth flag.
//!
//! When set, a **loopback** peer is treated as an authenticated capped client
//! with no token — the explicit, deliberately alarming override for
//! contributors developing against the daemon. The relaxation is compiled out
//! of release builds (see `middleware`), so a shipped daemon can never relax
//! auth regardless of this flag.

use std::sync::atomic::{AtomicBool, Ordering};

static DANGEROUS_MODE: AtomicBool = AtomicBool::new(false);

/// Enable/disable relaxed-auth mode. Set once at startup from `--dangerous`.
pub fn set_dangerous_mode(on: bool) {
    DANGEROUS_MODE.store(on, Ordering::SeqCst);
}

/// True when the daemon was started with `--dangerous`.
pub fn is_dangerous_mode() -> bool {
    DANGEROUS_MODE.load(Ordering::SeqCst)
}
