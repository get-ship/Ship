//! Shipd implementations of the ship-pipeline-runtime port traits.
//!
//! Bridges `ApiState` and the daemon's `DispatchHandle` to the abstract ports
//! declared in `ship_pipeline_runtime::pipeline_ports`. Follows the same
//! pattern as `agent_event_ports.rs`.

use async_trait::async_trait;
use ship_pipeline_runtime::pipeline_ports::{
    PipelineDispatchPort, TurnSignalPort, WorktreesDirPort,
};
use ship_substrate::events::EventEnvelope;
use std::path::{Path, PathBuf};

use crate::ship_runtime::ApiState;

// ── PipelineDispatchPort ──────────────────────────────────────────────────────

pub struct PipelineDispatchAdapter {
    state: ApiState,
}

impl PipelineDispatchAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl PipelineDispatchPort for PipelineDispatchAdapter {
    async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        self.state.persist_and_route(envelope).await
    }

    fn ship_dir(&self) -> &Path {
        self.state.ship_dir()
    }
}

// ── TurnSignalPort ────────────────────────────────────────────────────────────
//
// With feature = "ship-code": signals the live orchestrator activity channel
// via the dispatch handle.
// Without: no-op (always returns false — no dispatch, nothing to signal).

#[cfg(feature = "ship-code")]
pub struct TurnSignalAdapter {
    state: ApiState,
}

#[cfg(feature = "ship-code")]
impl TurnSignalAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[cfg(feature = "ship-code")]
#[async_trait]
impl TurnSignalPort for TurnSignalAdapter {
    async fn signal_turn_ended(&self, slug: &str) -> bool {
        let Some(dh) = self.state.dispatch_handle() else {
            return false;
        };
        let txs = dh.activity_txs.lock().await;
        if let Some(tx) = txs.get(slug) {
            tx.send(ship_pipeline_runtime::orchestrator::StepActivity::TurnEnded)
                .is_ok()
        } else {
            false
        }
    }
}

// No-op stub when ship-code feature is off.
#[cfg(not(feature = "ship-code"))]
pub struct TurnSignalAdapter;

#[cfg(not(feature = "ship-code"))]
impl TurnSignalAdapter {
    pub fn new(_state: ApiState) -> Self {
        Self
    }
}

#[cfg(not(feature = "ship-code"))]
#[async_trait]
impl TurnSignalPort for TurnSignalAdapter {
    async fn signal_turn_ended(&self, _slug: &str) -> bool {
        false
    }
}

// ── WorktreesDirPort ──────────────────────────────────────────────────────────

pub struct WorktreesDirAdapter;

impl WorktreesDirPort for WorktreesDirAdapter {
    fn worktrees_dir(&self) -> PathBuf {
        crate::local_paths::worktrees_dir()
    }
}
