//! Event-sourced actor state writes.
//!
//! Each public function builds an `EventEnvelope` and hands it to the
//! canonical write path: `SqliteEventStore::append` persists the event
//! row and applies the actor projection in a single transaction.
//! When a kernel router is up we additionally deliver to actor mailboxes
//! (sync, no re-persist).
//!
//! ADR GHihs2tn: all actor lifecycle events are elevated=1.

use anyhow::Result;

use crate::db::block_on_anyhow;
use crate::events::types::event_types;
use crate::events::types::{ActorCrashed, ActorCreated, ActorSlept, ActorStopped, ActorWoke};
use crate::events::EventEnvelope;
use crate::persistence::event_store::{EventStore, SqliteEventStore};

fn emit_actor_event<P: serde::Serialize>(
    actor_id: &str,
    project_id: Option<&str>,
    parent_actor_id: Option<&str>,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let mut envelope = EventEnvelope::new(event_type, actor_id, payload)?
        .with_context(project_id, None)
        .with_actor_id(actor_id)
        .elevate();
    if let Some(parent) = parent_actor_id {
        envelope = envelope.with_parent_actor_id(parent);
    }

    // Canonical write: event row + actor projection apply in one tx.
    let store = SqliteEventStore::new()?;
    store.append(&envelope)?;

    // Deliver to mailboxes if a kernel router is running. Persistence
    // already happened above, so this is a delivery-only step.
    if let Some(kr) = crate::events::kernel_router() {
        let _ = block_on_anyhow(async {
            kr.lock().await.deliver(&envelope);
            Ok(())
        });
    }

    Ok(())
}

// ── public API ────────────────────────────────────────────────────────────────

pub fn emit_actor_created(
    actor_id: &str,
    payload: &ActorCreated,
    project_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(actor_id, project_id, parent_actor_id, event_types::ACTOR_CREATED, payload)
}

/// Convenience wrapper for pipeline / one-shot spawns: builds an `ActorCreated`
/// payload from raw fields. The dashboard treats the row identically to a
/// chat agent — `kind` (`pipeline-step` etc.) is what distinguishes it.
pub fn emit_actor_created_simple(
    actor_id: &str,
    kind: &str,
    project_id: Option<&str>,
    agent_id: Option<&str>,
    provider: Option<&str>,
    transport: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    let payload = ActorCreated {
        kind: kind.to_string(),
        environment_type: "local".to_string(),
        workspace_id: project_id.map(str::to_string),
        parent_actor_id: parent_actor_id.map(str::to_string),
        restart_count: 0,
        agent_id: agent_id.map(str::to_string),
        provider: provider.map(str::to_string),
        transport: transport.map(str::to_string),
        persistence: Some("one-shot".to_string()),
        ..Default::default()
    };
    emit_actor_event(
        actor_id,
        project_id,
        parent_actor_id,
        event_types::ACTOR_CREATED,
        &payload,
    )
}

pub fn emit_actor_woke(
    id: &str,
    project_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(id, project_id, parent_actor_id, event_types::ACTOR_WOKE, &ActorWoke {})
}

pub fn emit_actor_slept(
    id: &str,
    idle_secs: u64,
    project_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(
        id,
        project_id,
        parent_actor_id,
        event_types::ACTOR_SLEPT,
        &ActorSlept { idle_secs },
    )
}

pub fn emit_actor_stopped(
    id: &str,
    payload: &ActorStopped,
    project_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(
        id,
        project_id,
        parent_actor_id,
        event_types::ACTOR_STOPPED,
        payload,
    )
}

pub fn emit_actor_crashed(
    id: &str,
    error: &str,
    restart_count: u32,
    project_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(
        id,
        project_id,
        parent_actor_id,
        event_types::ACTOR_CRASHED,
        &ActorCrashed { error: error.to_string(), restart_count },
    )
}
