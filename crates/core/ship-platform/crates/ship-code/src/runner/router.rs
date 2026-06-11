//! Router mechanism — the domain-free entry-gate for the L1 orchestrator loop.
//! Moved from ship-runner (L0) to ship-code (L1).

use ship_substrate::events::EventEnvelope;
use crate::dispatch_ports::ProjectionReader;
use ship_types::event_types as et;
use std::sync::Arc;

use super::dedup::{EventDedup, Seen};

/// Bounds concurrent agent-session dispatch preps.
pub static DISPATCH_GATE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(8);

#[derive(Clone)]
pub struct RouterGate {
    dedup: Arc<EventDedup>,
    projections: Arc<dyn ProjectionReader>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Admit {
    Proceed,
    Duplicate,
    Terminal,
}

impl RouterGate {
    pub fn new(dedup: Arc<EventDedup>, projections: Arc<dyn ProjectionReader>) -> Self {
        Self { dedup, projections }
    }

    pub async fn admit(&self, envelope: &EventEnvelope) -> Admit {
        if self.dedup.record(&envelope.id) == Seen::Duplicate {
            tracing::debug!(
                event_id = %envelope.id,
                event_type = %envelope.event_type,
                "job-dispatch: duplicate event — skipping"
            );
            return Admit::Duplicate;
        }

        if !is_operator_event(&envelope.event_type)
            && self.projections.is_run_terminal(&envelope.entity_id).await
        {
            tracing::warn!(
                entity_id = %envelope.entity_id,
                event_type = %envelope.event_type,
                "job-dispatch: ignoring event for terminal job"
            );
            return Admit::Terminal;
        }

        Admit::Proceed
    }
}

pub async fn terminal_guard(
    dedup: &EventDedup,
    projections: &dyn ProjectionReader,
    envelope: &EventEnvelope,
) -> Admit {
    if dedup.record(&envelope.id) == Seen::Duplicate {
        tracing::debug!(
            event_id = %envelope.id,
            event_type = %envelope.event_type,
            "job-dispatch: duplicate event — skipping"
        );
        return Admit::Duplicate;
    }

    if !is_operator_event(&envelope.event_type)
        && projections.is_run_terminal(&envelope.entity_id).await
    {
        tracing::warn!(
            entity_id = %envelope.entity_id,
            event_type = %envelope.event_type,
            "job-dispatch: ignoring event for terminal job"
        );
        return Admit::Terminal;
    }

    Admit::Proceed
}

pub fn is_operator_event(event_type: &str) -> bool {
    event_type == "pipeline.run.started" || event_type == et::PIPELINE_RUN_CANCELLED
}

pub fn extract_slug(envelope: &EventEnvelope) -> Option<String> {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).ok()?;
    payload["slug"].as_str().map(str::to_string)
}
