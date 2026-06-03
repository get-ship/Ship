//! Injected seams the device service can't own in L1.
//!
//! The device service owns the window-management *mechanism* (the focus-vs-spawn
//! decision, the register/close lifecycle, consulting the actor projection for
//! live windows). Two capabilities it depends on are transport-/runtime-owned
//! and therefore injected as ports:
//!
//!   - [`WindowSessions`] — the session mint/revoke seam. Minting a pid-bound
//!     C2 window token and revoking it are operations on the daemon's in-memory
//!     `SessionRegistry`, which lives in shipd. The daemon supplies the impl;
//!     the service decides *when* to mint/revoke.
//!   - [`SpaceRegistry`] — space-existence validation. A window can open onto
//!     any REGISTERED space (it can't conjure one), but the registry of known
//!     spaces is the `projects` projection, an L1 read model the device crate
//!     does not own. The daemon supplies the impl; the service decides *whether*
//!     a target is valid.
//!
//! Both ports are object-safe so the daemon can hand the service a `&dyn Port`.
//! `WindowSessions` is `async` (the registry lock is async).

use anyhow::Result;
use async_trait::async_trait;

/// Mint / revoke the pid-bound C2 session token a window presents on every
/// Connect call. The daemon owns the `SessionRegistry`; this is the seam the
/// service drives so register/close own the *decision* without owning the
/// registry.
#[async_trait]
pub trait WindowSessions: Send + Sync {
    /// Mint a pid-bound, C2-capped session for a window onto `space_id` and
    /// insert it into the registry. Returns the bearer token the webview uses.
    async fn mint_window_session(&self, space_id: &str, pid: u32) -> Result<String>;

    /// Revoke every session whose pid-bound identity matches `(space_id, pid)`
    /// — the window's token. Returns true if anything was dropped (an instant
    /// authority cut). A window is single-instance per space, so this targets
    /// exactly that window's token.
    async fn revoke_window_session(&self, space_id: &str, pid: u32) -> bool;
}

/// Resolve whether a space id is REGISTERED. A window's `register` validates its
/// target against this before binding — a window can't conjure a space. The
/// daemon backs this with the `projects` projection read.
pub trait SpaceRegistry: Send + Sync {
    /// True if `space_id` names a registered space. `Err` surfaces a lookup
    /// failure (the service maps it to a service error rather than treating an
    /// unreachable registry as "space absent").
    fn space_exists(&self, space_id: &str) -> Result<bool>;
}
