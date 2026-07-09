//! Runner router mechanism — the domain-free entry gate both `ship-agents`
//! (`service.agents` actor) and `ship-pipeline-runtime` (`orchestrator`)
//! hit before dispatching an envelope to domain policy.
//!
//! Two surfaces:
//! - [`terminal_guard`] + [`Admit`]: dedup-by-event-id (single sliding window
//!   held by [`EventDedup`]) AND the terminal-state guard (a late envelope
//!   for a job that already reached Merged/Failed/Cancelled is rejected,
//!   unless the event type is operator-shaped — cancels are exempt so the
//!   operator can still tear down a stuck job). The terminal check goes
//!   through a primitive `Fn(&str) -> Future<Output = bool>` provided by
//!   the caller — port-typed from `ProjectionReader::is_run_terminal` per
//!   census §4.3 so the runner never imports the projection trait.
//! - [`DISPATCH_GATE`] semaphore: bounds concurrent agent-session dispatch
//!   preps so one slow `worktree add + harness spawn + sandbox-up` doesn't
//!   stall every other event.
//! - [`extract_slug`] + [`is_operator_event`]: small envelope helpers.

use std::future::Future;
use std::pin::Pin;

use ship_substrate::events::EventEnvelope;
use ship_types::event_types as et;

use crate::dedup::{EventDedup, Seen};

/// Bounds concurrent agent-session dispatch preps.
pub static DISPATCH_GATE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(8);

#[derive(Debug, PartialEq, Eq)]
pub enum Admit {
    Proceed,
    Duplicate,
    Terminal,
}

/// Run the runner entry gate: dedup, then terminal-state guard.
///
/// Both checks are independent; the duplicate short-circuit is logged at
/// debug (replay noise), the terminal rejection is logged at warn (a
/// late envelope is operator-visible signal worth surfacing).
///
/// `is_terminal` is a closure returning the domain's terminal-state bool.
/// The closure frees the runner from importing `ProjectionReader`
/// (census §4.3 — port-typed to bool) and keeps `ship-runner` free of the
/// harness-driver trait surface.
pub async fn terminal_guard<F>(
    dedup: &EventDedup,
    is_terminal: F,
    envelope: &EventEnvelope,
) -> Admit
where
    F: Fn(&str) -> Pin<Box<dyn Future<Output = bool> + Send + 'static>>,
{
    if dedup.record(&envelope.id) == Seen::Duplicate {
        tracing::debug!(
            event_id = %envelope.id,
            event_type = %envelope.event_type,
            "job-dispatch: duplicate event — skipping"
        );
        return Admit::Duplicate;
    }

    if !is_operator_event(&envelope.event_type)
        && is_terminal(&envelope.entity_id).await
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

/// Operator-shaped event types bypass the terminal-state guard. Cancel
/// stays reachable for a stuck job.
pub fn is_operator_event(event_type: &str) -> bool {
    event_type == "pipeline.run.started" || event_type == et::PIPELINE_RUN_CANCELLED
}

/// Read the `slug` field from an envelope's `payload_json`. Returns `None`
/// if the payload isn't JSON or doesn't carry a string `slug`.
pub fn extract_slug(envelope: &EventEnvelope) -> Option<String> {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).ok()?;
    payload["slug"].as_str().map(str::to_string)
}
