//! Kernel-emit plumbing for the runner mechanism.
//!
//! Two layers:
//!   1. [`EventSink`] — the injected kernel-emit *port*. The runner mechanism
//!      depends on this trait, never on a concrete [`KernelRouter`]. The L0
//!      concrete impl ([`KernelEventSink`]) is provided at construction.
//!   2. [`persist_and_route`] — shared envelope-persist helper used by domain
//!      emit modules (ship-agents, pipeline-defs, …). Domain-typed
//!      convenience emitters live with their owning domains; payload STRUCTS
//!      stay under `events::{process,executor,pipeline}`.

use crate::events::{CallerKind, EmitContext, EventEnvelope};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Kernel-emit port ───────────────────────────────────────────────────────

/// Port: emit an event envelope. Re-exported from [`ship_types::EventSink`].
pub use ship_types::EventSink;

/// The L0 concrete [`EventSink`]: wraps the substrate [`KernelRouter`] and
/// drives it through [`persist_and_route`]. Built by the host (daemon) and
/// handed to the runner mechanism as `Arc<dyn EventSink>`.
#[derive(Clone)]
pub struct KernelEventSink {
    kernel: Arc<Mutex<crate::events::KernelRouter>>,
}

impl KernelEventSink {
    pub fn new(kernel: Arc<Mutex<crate::events::KernelRouter>>) -> Self {
        Self { kernel }
    }
}

#[async_trait]
impl EventSink for KernelEventSink {
    async fn emit(&self, envelope: EventEnvelope) {
        let slug = envelope.space_id().map(str::to_string).unwrap_or_default();
        persist_and_route(&self.kernel, envelope, &slug).await;
    }
}

// ── Routing ─────────────────────────────────────────────────────────────────

/// Persist an event and deliver it through the kernel.
///
/// Thin wrapper around `KernelRouter::emit` that logs failures at ERROR
/// with slug/event_type/entity_id context. The unit return type matches
/// the call-site contract (most callers don't propagate); when callers are
/// rewritten to propagate, they should call `kernel.emit` directly and
/// delete their use of this helper.
pub async fn persist_and_route(
    kernel: &Arc<Mutex<crate::events::KernelRouter>>,
    envelope: EventEnvelope,
    slug: &str,
) {
    let event_type = envelope.event_type.clone();
    let entity_id = envelope.entity_id.clone();

    // Relay: carry the envelope's own resolved space through the binding,
    // preserving its session. Daemon-internal mint (trusted relay).
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(
            envelope.space_id().map(ship_types::SpaceId::new),
            agent_schema::Authority::Sdk,
        ),
        session_id: envelope.session_id.clone(),
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::error!(
            slug, event_type = %event_type, entity_id = %entity_id,
            "persist_and_route: kernel emit failed: {e}"
        );
    }
}
