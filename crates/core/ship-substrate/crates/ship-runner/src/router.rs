//! Router mechanism — the domain-free entry-gate the L1 orchestrator loop
//! runs before its policy match.
//!
//! The historic `handle_orchestrator_event` mixed two concerns:
//!   - **mechanism** (here): dedup by event id, the terminal-state guard, the
//!     dispatch-concurrency bound, and the two pure envelope helpers
//!     (`is_operator_event`, `extract_slug`);
//!   - **policy** (stays in L1): the per-event-type `match` that loads
//!     projections, advances the DAG, runs gates, etc.
//!
//! This module owns the mechanism. It depends only on the runner ports
//! ([`ProjectionReader`]) plus substrate event types and the local dedup.

use ship_substrate::events::EventEnvelope;
use ship_substrate::services::dispatch_ports::ProjectionReader;
use ship_types::event_types as et;
use std::sync::Arc;

use super::dedup::{EventDedup, Seen};

/// Bounds concurrent agent-session dispatch preps so a burst doesn't spawn
/// a thundering herd of harness processes at once. 8 is a starting point —
/// instrument dispatch latency under load and tune if needed.
///
/// The L1 loop acquires a permit before detaching a slow `agent.session.started`
/// prep so the mailbox returns immediately without unbounded fan-out.
pub static DISPATCH_GATE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(8);

/// The runner's event entry-gate. Bundles the two pre-checks the mechanism
/// owns — event-id dedup and the terminal-state guard — so the L1 router loop
/// runs one call before its policy match instead of re-implementing both.
#[derive(Clone)]
pub struct RouterGate {
    dedup: Arc<EventDedup>,
    projections: Arc<dyn ProjectionReader>,
}

/// Outcome of [`RouterGate::admit`].
#[derive(Debug, PartialEq, Eq)]
pub enum Admit {
    /// First sight of this event for a live run — the caller should process it.
    Proceed,
    /// A previously-seen event id (mailbox redelivery, replay race) — skip.
    Duplicate,
    /// The run is terminal (failed / merged / cancelled) and the event is not
    /// an operator signal — skip so a late pipeline event can't resurrect it.
    Terminal,
}

impl RouterGate {
    pub fn new(dedup: Arc<EventDedup>, projections: Arc<dyn ProjectionReader>) -> Self {
        Self { dedup, projections }
    }

    /// Run the mechanism's pre-checks for one envelope.
    ///
    /// 1. **Dedup by event id** — prevents a replayed envelope (mailbox
    ///    redelivery, startup replay racing live delivery, test harness
    ///    replays) from advancing the pipeline twice.
    /// 2. **Terminal-state guard** — once a run is Failed/Merged/Cancelled the
    ///    router ignores subsequent pipeline events for it. Operator events
    ///    ([`is_operator_event`]) are exempt: those are signals FROM the
    ///    operator, not pipeline progression. The terminal check goes through
    ///    the [`ProjectionReader`] port (primitive `bool`) so the runner never
    ///    names the projection status enum here.
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

/// Run the two mechanism pre-checks without holding a [`RouterGate`].
///
/// Convenience for callers that already hold the dedup + projections handles
/// directly (the L1 `OrchestratorContext`). Equivalent to
/// `RouterGate::new(dedup, projections).admit(envelope)`.
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

/// Events originating from the operator (or system policy), not from
/// pipeline progression. These are exempt from the terminal-state guard
/// so operators can still issue signals about a finalized job.
pub fn is_operator_event(event_type: &str) -> bool {
    event_type == "pipeline.run.started"
        || event_type == "pipeline.run.updated"
        || event_type == et::PIPELINE_RUN_CANCELLED
}

/// Extract the slug from an event envelope payload.
pub fn extract_slug(envelope: &EventEnvelope) -> Option<String> {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).ok()?;
    payload["slug"].as_str().map(str::to_string)
}
