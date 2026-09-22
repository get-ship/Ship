//! Emit-context + time helpers for the space service.
use ship_substrate::events::validator::EmitContext;

/// Emit context for the Runtime-scoped `space.*` events (`space.opened`,
/// `space.seen`). These carry NO space-envelope binding — the `spaces`
/// projection is the device-wide record (read daemon-wide, no space in
/// hand), so the events route to `runtime.db`. The daemon-internal mint
/// carries the device-id scope key for the Runtime tier.
pub(crate) fn runtime_emit_ctx() -> EmitContext {
    EmitContext::daemon_internal(None)
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}
