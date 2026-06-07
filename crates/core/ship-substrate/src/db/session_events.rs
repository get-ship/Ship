//! Typed session event emission.
//!
//! Each function builds an `EventEnvelope` and hands it to the canonical
//! write path: `SqliteEventStore::append` persists the event row and
//! applies the session projection in a single transaction. When a kernel
//! router is up we additionally deliver to actor mailboxes (sync, no
//! re-persist).
//!
//! `session.progress` is non-elevated; otherwise the same path.
//!
//! ADR GHihs2tn: all session lifecycle transitions must emit typed events.

use anyhow::Result;

use crate::db::block_on_anyhow;
use crate::events::types::event_types;
use crate::events::types::{
    SessionAttached, SessionEnded, SessionProgress, SessionStarted,
};
use crate::events::EventEnvelope;
use crate::persistence::event_store::SqliteEventStore;

// ── helpers ──────────────────────────────────────────────────────────────────

fn session_envelope<P: serde::Serialize>(
    event_type: &str,
    session_id: &str,
    workspace_id: &str,
    payload: &P,
) -> Result<EventEnvelope> {
    // session.* is Runtime-scoped (registry + runtime.db home). The workspace
    // linkage rides on `parent_actor_id` + the payload's `workspace_id` (what the
    // session projection reads), NOT the envelope scope — so we attach the
    // session id as context but keep the scope Runtime for enforce_scope.
    Ok(EventEnvelope::new(event_type, session_id, payload)?
        .with_context(None, Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(workspace_id)
        .elevate())
}

/// Canonical write: event row + session projection apply in one tx via
/// `SqliteEventStore::append`. Mailbox delivery is best-effort if a
/// kernel router is up.
fn emit_session(envelope: EventEnvelope, _workspace_id: &str) -> Result<EventEnvelope> {
    // session.* is Runtime-scoped → write runtime.db (the session projection's db).
    let store = SqliteEventStore::new_for_scope(
        &crate::project::get_global_dir()?,
        ship_types::event_registry::EventScope::Runtime,
        None,
    )?;
    // Enforced write: stamp author + scope key from the daemon-internal binding,
    // then persist (event row + session projection in one tx). session.* is
    // Runtime-scoped, so the binding carries no space.
    let binding = ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk);
    let stamped = store.append_enforced(&envelope, &binding)?;

    if let Some(kr) = crate::events::kernel_router() {
        let _ = block_on_anyhow(async {
            kr.lock().await.deliver(&stamped);
            Ok(())
        });
    }

    Ok(stamped)
}

// ── public API ────────────────────────────────────────────────────────────────

pub fn insert_session_with_started_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionStarted,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(
        event_types::SESSION_STARTED,
        session_id,
        workspace_id,
        payload,
    )?;
    emit_session(envelope, workspace_id)
}

pub fn insert_session_attached_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionAttached,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(
        event_types::SESSION_ATTACHED,
        session_id,
        workspace_id,
        payload,
    )?;
    emit_session(envelope, workspace_id)
}

/// Emit `session.progress` — elevated when escalation is set.
pub fn insert_session_progress_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionProgress,
    elevated: bool,
) -> Result<()> {
    let mut envelope = EventEnvelope::new(event_types::SESSION_PROGRESS, session_id, payload)?
        .with_context(None, Some(session_id))
        .with_actor_id(session_id)
        .with_parent_actor_id(workspace_id);
    if elevated {
        envelope = envelope.elevate();
    }

    emit_session(envelope, workspace_id)?;
    Ok(())
}

pub fn update_session_with_ended_event(
    session_id: &str,
    workspace_id: &str,
    payload: &SessionEnded,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(
        event_types::SESSION_ENDED,
        session_id,
        workspace_id,
        payload,
    )?;
    emit_session(envelope, workspace_id)
}


/// Emit a session drain/tool-count event transactionally.
pub fn emit_session_drain_event<P: serde::Serialize>(
    event_type: &str,
    session_id: &str,
    workspace_id: &str,
    payload: &P,
) -> Result<EventEnvelope> {
    let envelope = session_envelope(event_type, session_id, workspace_id, payload)?;
    emit_session(envelope, workspace_id)
}
