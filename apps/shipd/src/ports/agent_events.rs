//! Shipd implementation of the `AgentDispatchPort` declared in ship-agents —
//! bridges shipd's `ApiState` to the dispatch core without importing
//! `ApiState` into the domain crate.

use async_trait::async_trait;
use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
use ship_substrate::events::EventEnvelope;

use crate::ship_runtime::ApiState;

// ── AgentDispatchPort ─────────────────────────────────────────────────────────

/// Adapter bridging shipd's `ApiState` to the `AgentDispatchPort` declared in
/// ship-agents. Provides the three daemon-specific capabilities the dispatch
/// core needs without importing `ApiState` into the domain crate.
pub struct AgentDispatchAdapter {
    state: ApiState,
}

impl AgentDispatchAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl AgentDispatchPort for AgentDispatchAdapter {
    async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        self.state.persist_and_route(envelope).await
    }

    fn daemon_origin(&self) -> String {
        self.state.daemon_origin()
    }
    #[cfg(feature = "bridge")]
    fn worktrees_dir(&self) -> Option<std::path::PathBuf> {
        bridge::helpers::worktrees_dir().ok()
    }
}
