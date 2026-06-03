//! Injected seams the space service can't own in L1.
//!
//! The space service owns space *product logic* (info composition, the config
//! overlay, the window control surface). Two capabilities it depends on are
//! transport-/runtime-owned and therefore injected as ports:
//!
//!   - [`SpaceEvents`] — the per-space-setting emit seam. The override write is
//!     event-sourced (`space.setting.changed` riding the caller's space on the
//!     envelope), so it goes through the kernel. The daemon owns the kernel +
//!     the caller's emit context; it supplies the concrete impl. The service
//!     stays free of `SharedKernel` plumbing and is testable with a recording
//!     fake.
//!   - [`WindowSessions`] — the session mint/revoke seam. Minting a pid-bound
//!     C2 window token and revoking it are operations on the daemon's
//!     in-memory `SessionRegistry`, which lives in shipd. The daemon supplies
//!     the impl; the service decides *when* to mint/revoke.
//!
//! Both ports are `async` (the kernel emit + the registry lock are async) and
//! object-safe so the daemon can hand the service a `&dyn Port`.

use anyhow::Result;
use async_trait::async_trait;

/// Emit a per-space `space.setting.changed` event for `(space_id, key)` with
/// `value_json` (already validated as JSON by the caller). The implementor
/// stamps the caller's emit context + the space onto the envelope and routes it
/// through the kernel. Idempotent at the projection (UPSERT on `(space, key)`).
#[async_trait]
pub trait SpaceEvents: Send + Sync {
    async fn emit_setting_changed(
        &self,
        space_id: &str,
        key: &str,
        value_json: &str,
    ) -> Result<()>;
}

/// Mint / revoke the pid-bound C2 session token a space window presents on
/// every Connect call. The daemon owns the `SessionRegistry`; this is the seam
/// the service drives so register/close own the *decision* without owning the
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
