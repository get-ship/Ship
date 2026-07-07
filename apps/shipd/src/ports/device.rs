//! Daemon-side adapters for the L1 device service's injected seams.
//!
//! The device service (`ship_device`) owns window-management *mechanism* but
//! injects two transport-/runtime-owned capabilities as ports. The daemon
//! supplies the concrete impls here:
//!
//!   - [`RegistryWindowSessions`] — mints / revokes the pid-bound C2 window
//!     token against the daemon's in-memory `SessionRegistry`. The mint/revoke
//!     mechanism the retired space-window seam used, now sole-owned here against
//!     the device crate's `WindowSessions` trait.
//!   - [`ProjectsSpaceRegistry`] — validates a window's target space against the
//!     `projects` projection table (Runtime-scoped, written by ship-space).

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use ship_device::ports::{SpaceRegistry, WindowSessions};

use crate::auth::SessionRegistry;


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
            .map_err(|e| anyhow!(e.to_string()))?;
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

/// `projects`-projection-backed space-existence check. A window's `register`
/// validates its explicit target against this before binding — a window can't
/// conjure a space.
pub struct ProjectsSpaceRegistry;

impl SpaceRegistry for ProjectsSpaceRegistry {
    fn space_exists(&self, space_id: &str) -> Result<bool> {
        Ok(ship_substrate::db::projects::get(space_id)
            .map_err(|e| anyhow!(e.to_string()))?
            .is_some())
    }
}
