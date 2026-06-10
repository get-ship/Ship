//! The `service.agents` actor — sole handler of `dispatch.requested` and
//! `session.cancelled`.
//!
//! Mirrors the orchestrator's `start_orchestrator` but for the agent-session
//! domain only. It subscribes to `dispatch.*` (spawn trigger) and `session.*`
//! (cancel teardown), runs the L0 router gate (dedup + terminal-state guard)
//! before its policy match, then dispatches:
//!   - `dispatch.requested` → [`crate::dispatch::handle_dispatch_requested`]
//!     (detached off the loop under the shared L0 `DISPATCH_GATE`);
//!   - `session.cancelled` → [`crate::cancel::handle_agent_cancelled`].
//!
//! The no-double-spawn invariant: orchestrator deleted its `dispatch.*`
//! arms, so this actor is the SOLE actor that acts on `dispatch.requested`.
//! The two actors' separate dedup instances therefore never double-admit a
//! spawn. orchestrator keeps subscribing to `session.completed/failed`
//! ONLY to advance composed pipeline sessions — it no longer spawns/cancels.
//!
//! `process.*` is in `write_namespaces` because the cancel kill path emits
//! `process.killed`; the kernel validator rejects emits outside the actor's
//! write namespaces. `session.*` is in `write_namespaces` because the dispatch
//! path emits `session.failed` on prep errors.

use std::sync::Arc;

#[cfg(feature = "harness-drivers")]
use ship_code::runner::router::{terminal_guard, Admit};
use ship_substrate::events::EventEnvelope;
use tokio::sync::Mutex;

#[cfg(feature = "harness-drivers")]
use crate::cancel::handle_agent_cancelled;
use crate::dispatch::handle_dispatch_requested;
use crate::ports::AgentPorts;

type Kernel = Arc<Mutex<ship_substrate::events::KernelRouter>>;

/// Bootstrap the agents actor: spawn its mailbox and process
/// `dispatch.requested` and `session.cancelled` events until the mailbox closes.
///
/// Called once during daemon startup (composition root). Returns when the
/// mailbox closes.
pub async fn start_agents(kernel: Kernel, ports: Arc<AgentPorts>) {
    let actor_id = "service.agents".to_string();
    let config = ship_substrate::events::ActorConfig {
        namespace: actor_id.clone(),
        // process.* — cancel kill path emits process.killed.
        // session.* — dispatch path emits session.failed on prep errors.
        write_namespaces: vec!["process.".to_string(), "session.".to_string()],
        read_namespaces: vec![],
        // dispatch.* — the spawn trigger (dispatch.requested).
        // session.* — the cancel teardown (session.cancelled).
        subscribe_namespaces: vec!["dispatch.".to_string(), "session.".to_string()],
        see_directed_events: false,
    };

    let mailbox = {
        let mut k = kernel.lock().await;
        match k.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => mb,
            Err(e) => {
                tracing::warn!("service.agents: failed to spawn actor: {e}");
                return;
            }
        }
    };

    let mut mb = mailbox;
    while let Some(envelope) = mb.recv().await {
        handle_agents_event(&kernel, &envelope, &ports).await;
    }
    tracing::info!("service.agents: mailbox closed");
}

/// Handle a single `dispatch.*` or `session.*` event. Runs the L0 entry gate
/// (event-id dedup + terminal-state guard) before the policy match.
///
/// With `harness-drivers` OFF, the terminal-state guard and the `cancelled`
/// arm are absent (no process to kill, no dedup registry). The floki dispatch
/// path in `handle_dispatch_requested` remains active.
pub async fn handle_agents_event(kernel: &Kernel, envelope: &EventEnvelope, ports: &AgentPorts) {
    // Runner-mechanism entry gate (L0 `ship_runner::router`): dedup by event id
    // AND the terminal-state guard. The terminal check goes through the
    // ProjectionReader port (primitive `bool`) so this crate never names the
    // projection status enum.
    //
    // Gate: terminal_guard + EventDedup are ship_code-backed; with
    // harness-drivers OFF these are absent.
    #[cfg(feature = "harness-drivers")]
    match terminal_guard(&ports.event_dedup, &*ports.projections, envelope).await {
        Admit::Proceed => {}
        Admit::Duplicate | Admit::Terminal => return,
    }

    match envelope.event_type.as_str() {
        // The dispatch.requested event — the spawn trigger. Owns the spawn /
        // register / exit-handler-attach atom. Detaches the slow prep itself
        // (under DISPATCH_GATE), so it returns to the loop immediately.
        "dispatch.requested" => {
            handle_dispatch_requested(kernel, envelope, ports).await;
        }
        // With harness-drivers OFF: no process to stop, no process.killed to
        // emit — the cancelled arm is a no-op.
        #[cfg(feature = "harness-drivers")]
        "session.cancelled" => {
            handle_agent_cancelled(kernel, envelope, ports).await;
        }
        _ => {}
    }
}
