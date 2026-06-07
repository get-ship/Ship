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

pub struct TurnSignalAdapter {
    state: ApiState,
}

impl TurnSignalAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

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

// ── WorktreesDirPort ──────────────────────────────────────────────────────────

pub struct WorktreesDirAdapter;

impl WorktreesDirPort for WorktreesDirAdapter {
    fn worktrees_dir(&self) -> PathBuf {
        ship_supervisor::helpers::worktrees_dir()
    }
}
