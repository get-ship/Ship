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
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
    event_type: &'static str,
    payload: &P,
) -> Result<()> {
    let mut envelope = EventEnvelope::new(event_type, actor_id, payload)?
        .with_context(space_id, None)
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
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(actor_id, space_id, parent_actor_id, event_types::ACTOR_CREATED, payload)
}

/// Convenience wrapper for pipeline / one-shot spawns: builds an `ActorCreated`
/// payload from raw fields. The dashboard treats the row identically to a
/// chat agent — `kind` (`pipeline-step` etc.) is what distinguishes it.
pub fn emit_actor_created_simple(
    actor_id: &str,
    kind: &str,
    space_id: Option<&str>,
    agent_id: Option<&str>,
    harness: Option<&str>,
    transport: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    let payload = ActorCreated {
        kind: kind.to_string(),
        environment_type: "local".to_string(),
        workspace_id: space_id.map(str::to_string),
        parent_actor_id: parent_actor_id.map(str::to_string),
        restart_count: 0,
        agent_id: agent_id.map(str::to_string),
        harness: harness.map(str::to_string),
        transport: transport.map(str::to_string),
        persistence: Some("one-shot".to_string()),
        ..Default::default()
    };
    emit_actor_event(
        actor_id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_CREATED,
        &payload,
    )
}

/// Register a Tauri shell window as a lifecycle actor (process-per-app).
///
/// Unlike agent actors (daemon-spawned → pid arrives later via `process.spawned`),
/// a window self-spawns and reports its own pid here — so the row is `live`
/// immediately and `find_live_window_for_space` can serve single-instance. A
/// *lifecycle* actor: identity + pid + lifecycle, no mailbox (the webview gets
/// events over its Connect stream). `space_id` goes in the payload (what the
/// actor projection reads), not just the envelope.
pub fn register_window(actor_id: &str, space_id: &str, pid: u32) -> Result<()> {
    let payload = ActorCreated {
        kind: "window".to_string(), // ActorKind::Window — projection treats kind as a string
        environment_type: "local".to_string(),
        space_id: Some(space_id.to_string()),
        pid: Some(pid),
        persistence: Some("long-lived".to_string()),
        ..Default::default()
    };
    emit_actor_created(actor_id, &payload, Some(space_id), None)
}

/// Deregister a window on close — emits `actor.stopped` so it leaves the live
/// set and `find_live_window_for_space` stops matching it.
pub fn deregister_window(actor_id: &str, space_id: &str) -> Result<()> {
    let payload = ActorStopped {
        reason: "window closed".to_string(),
        ..Default::default()
    };
    emit_actor_stopped(actor_id, &payload, Some(space_id), None)
}

/// Persist `actor.created` (event row + projection) and deliver the live
/// mailbox notification, all synchronously and without building a throwaway
/// async runtime.
///
/// MUST be called from a thread with no ambient Tokio runtime (a bare
/// `std::thread`), not inline on an async worker:
///   - the durable append's `block_on` would hit `block_in_place`, which
///     panics on a current-thread runtime (the default for `#[tokio::test]`);
///   - the live deliver uses the router's `blocking_lock`, which panics if a
///     runtime is active on the calling thread.
///
/// Callers dispatching from async (e.g. `RoutingExecutor::spawn`) offload this
/// to a `std::thread` and await its completion, so the dashboard row is
/// guaranteed before a short-lived child can exit. Using `blocking_lock`
/// instead of `block_on_anyhow` for the deliver avoids spinning a fresh
/// multi-thread runtime per call — that path exhausted the thread table under
/// parallel dispatch and broke unrelated process/PTY spawns.
pub fn persist_actor_created_simple(
    actor_id: &str,
    kind: &str,
    space_id: Option<&str>,
    agent_id: Option<&str>,
    harness: Option<&str>,
    transport: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    let payload = ActorCreated {
        kind: kind.to_string(),
        environment_type: "local".to_string(),
        workspace_id: space_id.map(str::to_string),
        parent_actor_id: parent_actor_id.map(str::to_string),
        restart_count: 0,
        agent_id: agent_id.map(str::to_string),
        harness: harness.map(str::to_string),
        transport: transport.map(str::to_string),
        persistence: Some("one-shot".to_string()),
        ..Default::default()
    };
    let mut envelope = EventEnvelope::new(event_types::ACTOR_CREATED, actor_id, &payload)?
        .with_context(space_id, None)
        .with_actor_id(actor_id)
        .elevate();
    if let Some(parent) = parent_actor_id {
        envelope = envelope.with_parent_actor_id(parent);
    }

    // Durable write — the part that must not be raced by a short-lived child.
    SqliteEventStore::new()?.append(&envelope)?;

    // Best-effort live notification. `blocking_lock` (no runtime) keeps this
    // cheap; a missed push just means subscribers see the row on next read.
    if let Some(kr) = crate::events::kernel_router() {
        kr.blocking_lock().deliver(&envelope);
    }

    Ok(())
}

pub fn emit_actor_woke(
    id: &str,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(id, space_id, parent_actor_id, event_types::ACTOR_WOKE, &ActorWoke {})
}

pub fn emit_actor_slept(
    id: &str,
    idle_secs: u64,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(
        id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_SLEPT,
        &ActorSlept { idle_secs },
    )
}

pub fn emit_actor_stopped(
    id: &str,
    payload: &ActorStopped,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(
        id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_STOPPED,
        payload,
    )
}

pub fn emit_actor_crashed(
    id: &str,
    error: &str,
    restart_count: u32,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
) -> Result<()> {
    emit_actor_event(
        id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_CRASHED,
        &ActorCrashed { error: error.to_string(), restart_count },
    )
}
