//! Agent-def domain consumer — the owner of the durable `agent.def.updated`.
//!
//! The `ship-config` watcher emits a single, generic, **transient** signal
//! for every changed config file: `config.file_changed` (Runtime +
//! `Retention::Ephemeral`). The signal carries `{domain, path, content_hash}` —
//! the watched dir's name, the file's rel path, and a SHA-256 of its bytes
//! (empty when removed, the deleted-marker the read-only index drops its row
//! on). The watcher does not know what a `domain` means; that knowledge is
//! the domain's job.
//!
//! This module is the AGENTS domain's job. The consumer subscribes to the
//! shared `agent_def_kernel` `KernelLog` broadcast, filters for
//! `config.file_changed` events whose `domain == "agents"`, and emits the
//! **durable** Space-scoped `agent.def.updated` (registered in agent-def,
//! where it is now emitted). The kernel's `route` honors `Retention`:
//! `config.file_changed` is skipped (Ephemeral, live-only), the
//! agent-emitted `agent.def.updated` is persisted (Durable, the row the
//! read-only def index and the per-agent watch consumers key off).
//!
//! The result is zero duplicate durable rows: the watcher no longer names
//! the agent event, the domain owns the durable emission, and the
//! broadcast is the single in-memory channel the producer + consumer
//! share. See `.artifacts/transient-events-design.md` (job/transient-events)
//! for the spike rationale.
//!
//! ## Out of scope (Phase 2)
//!
//! The same pattern applies to the other coarse `<domain>.changed` events
//! (`skill.changed`, `view.changed`, `pipeline.changed`, `rule.changed`,
//! `collection.changed`) and their fine-grained domain-event successors.
//! This module proves the mechanism on `agent.def.updated` only.

use std::sync::Arc;

use ship_substrate::events::EmitContext;
use ship_substrate::ports::KernelLog;
use ship_types::SpaceId;
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::{AgentDefUpdated, ConfigFileChanged};
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;

/// The watched-dir name this consumer filters for. Constant — the agent-def
/// domain owns only the `agents` dir.
const AGENTS_DOMAIN: &str = "agents";

/// Spawn the long-lived domain consumer: subscribe to the shared
/// `agent_def_kernel` broadcast, filter `config.file_changed` events whose
/// `domain == "agents"`, and emit one durable `agent.def.updated` per
/// match. Returns the join handle; the task exits when the broadcast
/// channel closes (daemon shutdown).
///
/// One consumer is mounted per space. The `space: SpaceId` is the binding
/// the durable emit rides — `agent.def.updated` is Space-scoped, so the
/// kernel routes the row to the space's db. The Runtime
/// `config.file_changed` envelope carries no space, so the consumer is the
/// authoritative link: every transient signal it observes lands as a
/// durable row in the space this consumer was constructed for.
pub fn spawn_agent_def_consumer(kernel: Arc<KernelLog>, space: SpaceId) -> JoinHandle<()> {
    let mut rx = kernel.subscribe();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Err(e) = handle_event(&kernel, &space, &event).await {
                        tracing::warn!(
                            "agent-def consumer: handle error for {}: {e}",
                            event.entity_id
                        );
                    }
                }
                Err(RecvError::Lagged(n)) => {
                    // A burst overran the broadcast buffer. The next boot
                    // rebuilds the read-only index from the def files, so
                    // a windowed drop is recoverable; log + continue.
                    tracing::warn!("agent-def consumer: lagged by {n} events, continuing");
                }
                Err(RecvError::Closed) => break,
            }
        }
    })
}

/// Handle a single event. Returns `Ok(())` on the ignore-paths (no domain
/// match, wrong event type, malformed payload) so the consumer's loop
/// never panics on a bad envelope.
async fn handle_event(
    kernel: &Arc<KernelLog>,
    space: &SpaceId,
    event: &EventEnvelope,
) -> anyhow::Result<()> {
    if event.event_type != event_types::CONFIG_FILE_CHANGED {
        return Ok(());
    }
    let payload: ConfigFileChanged = match serde_json::from_str(&event.payload_json) {
        Ok(p) => p,
        Err(e) => {
            // A malformed payload from the watcher is the watcher's bug,
            // not ours — log and skip. The transient signal will not be
            // re-delivered.
            tracing::warn!("agent-def consumer: malformed config.file_changed payload: {e}");
            return Ok(());
        }
    };
    if payload.domain != AGENTS_DOMAIN {
        return Ok(());
    }
    // Build the durable Space-scoped envelope. The `event_types` constant
    // resolves the registered scope (Space) at `EventEnvelope::new`; the
    // substrate wrapper's `enforce_scope` stamps the `scope_key` from the
    // EmitContext's binding when we emit below.
    let agent_payload = AgentDefUpdated {
        path: payload.path.clone(),
        content_hash: payload.content_hash.clone(),
    };
    let envelope = EventEnvelope::new(
        event_types::AGENT_DEF_UPDATED,
        &payload.path,
        &agent_payload,
    )?;
    let ctx = EmitContext::daemon_internal(Some(space.clone()));
    kernel.emit(envelope, &ctx).await?;
    Ok(())
}
