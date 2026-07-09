//! Daemon-side adapters for the L1 device service's injected seams.
//!
//! The device service (`ship_device`) owns window-management *mechanism* but
//! injects two transport-/runtime-owned capabilities as ports. The daemon
//! supplies the concrete impls here:
//!
//!   - [`OwnedRegistryWindowSessions`] — mints / revokes the pid-bound C2
//!     window token against the daemon's in-memory `SessionRegistry`. The
//!     mint/revoke mechanism the retired space-window seam used, now
//!     sole-owned here against the device crate's `WindowSessions` trait.
//!   - [`LiveSpaceRegistry`] — validates a window's target `space_id`
//!     against the daemon's *live* served-space identity (the mint-once
//!     `space_id` the runtime assigned at boot through
//!     `ship_space::identity::ensure_identity_for_root`), not against the
//!     full Runtime-scoped `spaces` projection. A window cannot conjure a
//!     space the daemon never opened.

use anyhow::{Result, anyhow};
use async_trait::async_trait;

use ship_device::ports::{SpaceRegistry, WindowSessions};

use crate::auth::SessionRegistry;
use crate::ship_runtime::ApiState;


/// Owned (cloned) registry — use this when the sessions adapter needs to be
/// stored in an `Arc` for a crate-owned wire (the borrowed variant can't be
/// 'static). The `SessionRegistry` is an `Arc<Mutex<_>>` so clone is cheap.
pub struct OwnedRegistryWindowSessions {
    registry: SessionRegistry,
}

impl OwnedRegistryWindowSessions {
    pub fn new(registry: SessionRegistry) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl WindowSessions for OwnedRegistryWindowSessions {
    async fn mint_window_session(&self, space_id: &str, pid: u32) -> Result<String> {
        let token = ship_auth::scopes::generate_session_token()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let session = ship_auth::scopes::space_window_session(space_id, pid, token.clone());
        self.registry.lock().await.insert(token.clone(), session);
        Ok(token)
    }

    async fn revoke_window_session(&self, space_id: &str, pid: u32) -> bool {
        let mut reg = self.registry.lock().await;
        let before = reg.len();
        reg.retain(|_, s| !(s.space_id.as_deref() == Some(space_id) && s.pid == Some(pid)));
        before != reg.len()
    }
}

/// `live-served-space`-backed space-existence check. A window's `register`
/// validates its explicit target against the daemon's own
/// `served_space_id` (the mint-once identity the runtime cached at boot).
/// A window cannot conjure a space the daemon never opened — "the launch is
/// the launch" (multi-window model: there's no enumeration surface for
/// windows to pick from; a window either names the served space or it
/// doesn't bind).
pub struct LiveSpaceRegistry {
    state: ApiState,
}

impl LiveSpaceRegistry {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

impl SpaceRegistry for LiveSpaceRegistry {
    fn space_exists(&self, space_id: &str) -> Result<bool> {
        // The "live state" is whatever the runtime serves right now:
        // `served_space_id` is the mint-once identity the runtime assigned
        // through `ship_space::identity::ensure_identity_for_root`. A probe
        // that names anything else (including spaces from prior lifetimes
        // that the runtime no longer serves) is false here — the daemon's
        // responsibility is its own served space, not the device-wide
        // `spaces` projection. (A future "list and pick a space" model
        // reaches for the projection directly via `ship_space::spaces::list`,
        // not through this seam.)
        Ok(self.state.active_space_id().as_deref() == Some(space_id))
    }
}
