//! Daemon-side adapters for the L1 space service's injected seams.
//!
//! The space service (`ship_platform::services::space`) owns space product
//! logic but injects two transport-/runtime-owned capabilities as ports. The
//! daemon supplies the concrete impls here:
//!
//!   - [`KernelSpaceEvents`] — emits `space.setting.changed` through the kernel,
//!     stamping the caller's emit context + the space onto the envelope (the
//!     space rides `with_context`, not the payload).
//!   - [`RegistryWindowSessions`] — mints / revokes the pid-bound C2 window
//!     token against the daemon's in-memory `SessionRegistry`.

use anyhow::{anyhow, Result};
use async_trait::async_trait;

use ship_platform::services::space::ports::{SpaceEvents, WindowSessions};
use ship_substrate::events::{EmitContext, EventEnvelope};
use ship_types::event_types::SPACE_SETTING_CHANGED;
use ship_types::payload::SpaceSettingChanged;

use crate::auth::SessionRegistry;
use crate::ship_runtime::SharedKernel;

/// Kernel-backed `space.setting.changed` emitter. Holds the shared kernel + the
/// caller's emit context (so the override carries the caller's identity/session,
/// not just the space).
pub struct KernelSpaceEvents<'a> {
    pub kernel: &'a SharedKernel,
    pub emit_ctx: EmitContext,
}

#[async_trait]
impl SpaceEvents for KernelSpaceEvents<'_> {
    async fn emit_setting_changed(&self, space_id: &str, key: &str, value_json: &str) -> Result<()> {
        let payload = SpaceSettingChanged { key: key.to_string(), value_json: value_json.to_string() };
        let envelope = EventEnvelope::new(SPACE_SETTING_CHANGED, key, &payload)
            .map_err(|e| anyhow!(e.to_string()))?
            .with_context(Some(space_id), self.emit_ctx.session_id.as_deref());
        self.kernel
            .lock()
            .await
            .emit(envelope, &self.emit_ctx)
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }
}

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
