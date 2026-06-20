//! Emit-context + time helpers for the space service.
use ship_substrate::events::validator::EmitContext;
use ship_types::SpaceId;

/// Emit context for Runtime-scoped registry events (`project.registered` /
/// `project.seen` / `project.switched`). These carry NO space — the `projects`
/// table is the machine-wide registry (the source the space-id resolver itself
/// consults), so they route to runtime.db. The daemon-internal mint carries the
/// device-id scope key for the Runtime tier.
pub(crate) fn runtime_emit_ctx() -> EmitContext {
    EmitContext::daemon_internal(None)
}

/// Emit context for Space-scoped events. The space rides the envelope (not the
/// payload); the daemon-internal mint carries the space scope key so the
/// kernel routes the event to `spaces/<space>/space.db` and fail-closes a
/// space-unbound caller. Used by `project.switched` (Space scope).
pub(crate) fn space_emit_ctx(space_id: &str) -> EmitContext {
    EmitContext::daemon_internal(Some(SpaceId::new(space_id)))
}

pub(crate) fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}
