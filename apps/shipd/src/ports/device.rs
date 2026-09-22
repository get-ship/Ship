//! Daemon-side adapters for the L1 device service's injected seams.
//!
//! The device service (`ship_device`) owns window-management *mechanism* but
//! injects transport-/runtime-owned capabilities as ports. The daemon supplies
//! the concrete impls here:
//!
//!   - [`OwnedRegistryWindowSessions`] — mints / revokes the pid-bound C2
//!     window token against the daemon's in-memory `SessionRegistry`.
//!   - [`LiveSpaceRegistry`] — validates a window's target `space_id`
//!     against the daemon's *live* served-space identity.
//!   - [`SubstrateWindowActors`] — actor projection reads + actor-events
//!     writes for window lifecycle (the substrate path isolation no longer
//!     reaches into).

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Mutex;

use ship_device::ports::{SpaceRegistry, WindowActors, WindowRow, WindowSessions};
use ship_substrate::db::actor_events;
use ship_substrate::events::KernelRouter;
use ship_substrate::projections::actor::{self, ActorRow};

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

/// Substrate-backed [`WindowActors`]: projection reads + actor-events writes.
///
/// Holds the daemon's shared kernel router so window register/deregister
/// always deliver live mailbox notifications (no optional silent skip).
pub struct SubstrateWindowActors {
    kernel: Arc<Mutex<KernelRouter>>,
}

impl SubstrateWindowActors {
    pub fn new(kernel: Arc<Mutex<KernelRouter>>) -> Self {
        Self { kernel }
    }
}

fn map_row(r: ActorRow) -> WindowRow {
    WindowRow {
        id: r.id,
        kind: r.kind,
        space_id: r.space_id,
        pid: r.pid,
        live: r.live,
    }
}

impl WindowActors for SubstrateWindowActors {
    fn find_live_for_space(&self, space_id: &str) -> Result<Option<WindowRow>> {
        Ok(actor::find_live_window_for_space(space_id)?.map(map_row))
    }

    fn list_windows(&self) -> Result<Vec<WindowRow>> {
        Ok(actor::list_windows()?.into_iter().map(map_row).collect())
    }

    fn get_actor(&self, actor_id: &str) -> Result<Option<WindowRow>> {
        Ok(actor::get_actor(actor_id)?.map(map_row))
    }

    fn register_window(&self, actor_id: &str, space_id: &str, pid: u32) -> Result<()> {
        actor_events::register_window(actor_id, space_id, pid, Some(&self.kernel))
    }

    fn deregister_window(&self, actor_id: &str, space_id: &str) -> Result<()> {
        actor_events::deregister_window(actor_id, space_id, Some(&self.kernel))
    }

    fn new_actor_id(&self) -> String {
        ship_substrate::gen_ulid()
    }
}
