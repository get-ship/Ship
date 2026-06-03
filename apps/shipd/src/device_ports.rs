//! Daemon-side adapters for the L1 device service's injected seams.
//!
//! The device service (`ship_device`) owns window-management *mechanism* but
//! injects two transport-/runtime-owned capabilities as ports. The daemon
//! supplies the concrete impls here:
//!
//!   - [`RegistryWindowSessions`] — mints / revokes the pid-bound C2 window
//!     token against the daemon's in-memory `SessionRegistry`. Same mint/revoke
//!     mechanism as the legacy space-window seam (`space_ports`), re-targeted at
//!     the device crate's `WindowSessions` trait.
//!   - [`ProjectsSpaceRegistry`] — validates a window's target space against the
//!     `projects` projection (an L1 read model the device crate does not own).

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use ship_device::ports::{SpaceRegistry, WindowSessions};

use crate::auth::SessionRegistry;

/// Session-registry-backed window token mint/revoke. Mints a pid-bound, C2
/// session via `space_window_session` and inserts it; revokes by dropping every
/// session matching `(space, pid)`.
pub struct RegistryWindowSessions<'a> {
    pub registry: &'a SessionRegistry,
}

#[async_trait]
impl WindowSessions for RegistryWindowSessions<'_> {
    async fn mint_window_session(&self, space_id: &str, pid: u32) -> Result<String> {
        let token = ship_platform::auth::scopes::generate_session_token()
            .map_err(|e| anyhow!(e.to_string()))?;
        let session =
            ship_platform::auth::scopes::space_window_session(space_id, pid, token.clone());
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
        Ok(ship_platform::projections::projects::get(space_id)
            .map_err(|e| anyhow!(e.to_string()))?
            .is_some())
    }
}
