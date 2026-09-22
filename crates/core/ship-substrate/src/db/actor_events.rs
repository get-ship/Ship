//! Event-sourced actor state writes.
//!
//! Each public function builds an `EventEnvelope` and hands it to the
//! canonical write path: `SqliteEventStore::append` persists the event
//! row and applies the actor projection in a single transaction.
//! When a kernel router handle is supplied we additionally deliver to
//! actor mailboxes (sync, no re-persist).
//!
//! ADR GHihs2tn: all actor lifecycle events are elevated=1.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::db::block_on_anyhow;
use crate::events::EventEnvelope;
use crate::events::KernelRouter;
use crate::events::types::event_types;
use crate::events::types::{ActorCreated, ActorSlept, ActorStopped, ActorWoke};
use crate::persistence::event_store::SqliteEventStore;
use ship_types::event_registry::EventScope;

/// Open a Runtime-scope store at the default ship-home. actor.* lands in
/// runtime.db, matching the kernel sink's routing for the same family.
fn runtime_store() -> Result<SqliteEventStore> {
    SqliteEventStore::new_for_scope(
        &crate::project::get_global_dir()?,
        EventScope::Runtime,
        None,
    )
}

/// Best-effort live mailbox delivery after a durable append. A miss is
/// not a write failure — the event row + projection already committed —
/// but the silent loss of a websocket/HUD notification is invisible.
/// Log when no router was injected (composition-root slip) and when the
/// deliver step itself errors. Production call sites must pass `Some`.
fn deliver_live(
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
    stamped: &EventEnvelope,
    actor_id: &str,
    event_type: &str,
) {
    let Some(kr) = kernel else {
        tracing::warn!(
            actor_id = %actor_id,
            event_type = event_type,
            "actor mailbox: no KernelRouter injected — durable record committed but live delivery skipped"
        );
        return;
    };
    let deliver_result = block_on_anyhow({
        let kr = kr.clone();
        let stamped = stamped.clone();
        async move {
            kr.lock().await.deliver(&stamped);
            Ok(())
        }
    });
    if let Err(e) = deliver_result {
        tracing::warn!(
            actor_id = %actor_id,
            event_type = event_type,
            error = ?e,
            "actor mailbox: live deliver failed; durable record was committed but subscribers may miss this event"
        );
    }
}

fn emit_actor_event<P: serde::Serialize>(
    actor_id: &str,
    // Context only: actor.* is Runtime-scoped (registry + runtime.db home) and
    // the actor projection reads `space_id` from the PAYLOAD, not the envelope
    // scope. Keeping it off the envelope keeps the scope Runtime for enforce_scope.
    _space_id: Option<&str>,
    parent_actor_id: Option<&str>,
    event_type: &'static str,
    payload: &P,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    let mut envelope = EventEnvelope::new(event_type, actor_id, payload)?
        .with_actor_id(actor_id)
        .elevate();
    if let Some(parent) = parent_actor_id {
        envelope = envelope.with_parent_actor_id(parent);
    }

    // Enforced canonical write: stamp + enforce against the daemon-internal
    // binding, then persist (event row + actor projection in one tx). actor.*
    // is Runtime-scoped → route to runtime.db (the db the actor projection
    // reads); the `space_id` here is payload/context, not the event scope, so
    // the binding carries no space.
    let store = runtime_store()?;
    let binding = ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk);
    let stamped = store.append_enforced(&envelope, &binding)?;

    // Deliver to mailboxes when an explicit router handle is provided.
    // Persistence already happened above, so this is a delivery-only step.
    deliver_live(kernel, &stamped, actor_id, event_type);

    Ok(())
}

// ── public API ────────────────────────────────────────────────────────────────

pub fn emit_actor_created(
    actor_id: &str,
    payload: &ActorCreated,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    emit_actor_event(
        actor_id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_CREATED,
        payload,
        kernel,
    )
}

/// Convenience wrapper for pipeline / one-shot spawns: builds an `ActorCreated`
/// payload from raw fields. The dashboard treats the row identically to a
/// chat agent — `kind` (`pipeline-step` etc.) is what distinguishes it.
pub fn emit_actor_created_simple(
    actor_id: &str,
    kind: &str,
    agent_id: Option<&str>,
    harness: Option<&str>,
    transport: Option<&str>,
    parent_actor_id: Option<&str>,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    let payload = ActorCreated {
        kind: kind.to_string(),
        environment_type: "local".to_string(),
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
        None,
        parent_actor_id,
        event_types::ACTOR_CREATED,
        &payload,
        kernel,
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
pub fn register_window(
    actor_id: &str,
    space_id: &str,
    pid: u32,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    let payload = ActorCreated {
        kind: "window".to_string(), // ProcessKind::Window — projection treats kind as a string
        environment_type: "local".to_string(),
        space_id: Some(space_id.to_string()),
        pid: Some(pid),
        persistence: Some("long-lived".to_string()),
        ..Default::default()
    };
    emit_actor_created(actor_id, &payload, Some(space_id), None, kernel)
}

/// Deregister a window on close — emits `actor.stopped` so it leaves the live
/// set and `find_live_window_for_space` stops matching it.
pub fn deregister_window(
    actor_id: &str,
    space_id: &str,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    let payload = ActorStopped {
        reason: "window closed".to_string(),
        ..Default::default()
    };
    emit_actor_stopped(actor_id, &payload, Some(space_id), None, kernel)
}

/// Sweep window-kind actors and close the lifecycle of any whose recorded pid
/// is no longer alive.
///
/// A clean window close calls `deregister_window`; a *crash* (SIGKILL, an OS
/// close that never reached the daemon) leaves the row stuck non-terminal with
/// a dead pid. The projection's derived `live` field already reads false (it
/// pid-probes), so the launcher's single-instance query is honest — but the
/// event log never recorded the death, so `status` stays `created` forever.
/// This reaper emits `actor.stopped` for each such row so the lifecycle is
/// closed in the log, not merely masked by the derived flag.
///
/// CLEANUP ONLY — never respawns. The launcher owns (re)spawn; this only
/// reconciles the log with process reality. Returns the ids it reaped.
///
/// MUST run on a thread with no ambient Tokio runtime (the emit path's durable
/// append uses `block_on`); daemon callers offload via `spawn_blocking`.
pub fn reap_dead_windows(kernel: Option<&Arc<Mutex<KernelRouter>>>) -> Result<Vec<String>> {
    use crate::projections::actor::{ListActorsFilter, list_actors, pid_alive};

    // "window" == ProcessKind::Window.as_str(); the projection treats kind as a
    // string throughout, so we filter on the string (no supervisor dep here).
    let windows = list_actors(&ListActorsFilter {
        kind: Some("window".to_string()),
        limit: 256,
        ..Default::default()
    })?;

    let mut reaped = Vec::new();
    for w in windows {
        // Already closed in the log — nothing to reconcile (keeps the sweep
        // idempotent: a dead row is stopped exactly once).
        if w.status == "stopped" || w.status == "crashed" {
            continue;
        }
        // A window always records its pid at registration, so a row whose pid
        // is gone is a window that vanished without deregistering.
        let pid_dead = w.pid.map(|p| !pid_alive(p)).unwrap_or(false);
        if !pid_dead {
            continue;
        }
        let payload = ActorStopped {
            reason: "reaped: window process not found".to_string(),
            ..Default::default()
        };
        emit_actor_stopped(&w.id, &payload, w.space_id.as_deref(), None, kernel)?;
        reaped.push(w.id);
    }
    Ok(reaped)
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
pub fn persist_actor_created_simple(
    actor_id: &str,
    kind: &str,
    agent_id: Option<&str>,
    harness: Option<&str>,
    transport: Option<&str>,
    parent_actor_id: Option<&str>,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    let payload = ActorCreated {
        kind: kind.to_string(),
        environment_type: "local".to_string(),
        parent_actor_id: parent_actor_id.map(str::to_string),
        restart_count: 0,
        agent_id: agent_id.map(str::to_string),
        harness: harness.map(str::to_string),
        transport: transport.map(str::to_string),
        persistence: Some("one-shot".to_string()),
        ..Default::default()
    };
    // actor.created is Runtime-scoped (registry + runtime.db home). `space_id`
    // is carried in the payload (what the actor projection reads), not the
    // envelope scope, so the scope stays Runtime for enforce_scope.
    let mut envelope = EventEnvelope::new(event_types::ACTOR_CREATED, actor_id, &payload)?
        .with_actor_id(actor_id)
        .elevate();
    if let Some(parent) = parent_actor_id {
        envelope = envelope.with_parent_actor_id(parent);
    }

    // Durable enforced write — the part that must not be raced by a short-lived
    // child. Stamp + enforce against the daemon-internal (Runtime) binding.
    let binding = ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk);
    let stamped = runtime_store()?.append_enforced(&envelope, &binding)?;

    // Best-effort live notification when an explicit router is supplied.
    // `blocking_lock` (no runtime) keeps this cheap; a missed push just means
    // subscribers see the row on next read. Production call sites must pass
    // `Some` — log when they don't so a composition-root slip is visible.
    match kernel {
        Some(kr) => kr.blocking_lock().deliver(&stamped),
        None => tracing::warn!(
            actor_id = %actor_id,
            event_type = event_types::ACTOR_CREATED,
            "actor mailbox: no KernelRouter injected — durable record committed but live delivery skipped"
        ),
    }

    Ok(())
}

pub fn emit_actor_woke(
    id: &str,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    emit_actor_event(
        id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_WOKE,
        &ActorWoke {},
        kernel,
    )
}

pub fn emit_actor_slept(
    id: &str,
    idle_secs: u64,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    emit_actor_event(
        id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_SLEPT,
        &ActorSlept { idle_secs },
        kernel,
    )
}

pub fn emit_actor_stopped(
    id: &str,
    payload: &ActorStopped,
    space_id: Option<&str>,
    parent_actor_id: Option<&str>,
    kernel: Option<&Arc<Mutex<KernelRouter>>>,
) -> Result<()> {
    emit_actor_event(
        id,
        space_id,
        parent_actor_id,
        event_types::ACTOR_STOPPED,
        payload,
        kernel,
    )
}
