//! The `service.agents` actor — sole handler of `agent.session.started` and
//! `agent.session.cancelled`.
//!
//! Mirrors the orchestrator's `start_orchestrator` but for the agent-session
//! domain only. It subscribes to `agent.session.*`, runs the L0 router gate
//! (dedup + terminal-state guard) before its policy match, then dispatches:
//!   - `agent.session.started` → [`crate::dispatch::handle_agent_session_started`]
//!     (detached off the loop under the shared L0 `DISPATCH_GATE`);
//!   - `agent.session.cancelled` → [`crate::cancel::handle_agent_cancelled`].
//!
//! The no-double-spawn invariant: orchestrator deleted its `agent.session.*`
//! arms, so this actor is the SOLE actor that acts on `agent.session.started`.
//! The two actors' separate dedup instances therefore never double-admit a
//! spawn. orchestrator keeps subscribing to `agent.session.completed/failed`
//! ONLY to advance composed pipeline sessions — it no longer spawns/cancels.
//!
//! `process.*` is in `write_namespaces` because the cancel kill path emits
//! `process.killed`; the kernel validator rejects emits outside the actor's
//! write namespaces.

use std::sync::Arc;

use ship_code::runner::router::{terminal_guard, Admit};
use ship_substrate::events::EventEnvelope;
use tokio::sync::Mutex;

use crate::cancel::handle_agent_cancelled;
use crate::dispatch::handle_agent_session_started;
use crate::ports::AgentPorts;

type Kernel = Arc<Mutex<ship_substrate::events::KernelRouter>>;

/// Bootstrap the agents actor: spawn its mailbox and process
/// `agent.session.{started,cancelled}` events until the mailbox closes.
///
/// Called once during daemon startup (composition root). Returns when the
/// mailbox closes.
pub async fn start_agents(kernel: Kernel, ports: Arc<AgentPorts>) {
    let actor_id = "service.agents".to_string();
    let config = ship_substrate::events::ActorConfig {
        namespace: actor_id.clone(),
        // process.* is REQUIRED — the cancel kill path emits process.killed and
        // the kernel validator rejects emits outside write_namespaces.
        write_namespaces: vec!["agent.session.".to_string(), "process.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["agent.session.".to_string()],
        see_directed_events: false,
    };

    let mailbox = {
        let mut k = kernel.lock().await;
        match k.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => mb,
            Err(e) => {
                tracing::warn!("agent-session-dispatch: failed to spawn actor: {e}");
                return;
            }
        }
    };

    let mut mb = mailbox;
    while let Some(envelope) = mb.recv().await {
        handle_agents_event(&kernel, &envelope, &ports).await;
    }
    tracing::info!("agent-session-dispatch: mailbox closed");
}

/// Handle a single `agent.session.*` event. Runs the L0 entry gate (event-id
/// dedup + terminal-state guard) before the policy match.
pub async fn handle_agents_event(kernel: &Kernel, envelope: &EventEnvelope, ports: &AgentPorts) {
    // Runner-mechanism entry gate (L0 `ship_runner::router`): dedup by event id
    // AND the terminal-state guard. The terminal check goes through the
    // ProjectionReader port (primitive `bool`) so this crate never names the
    // projection status enum.
    match terminal_guard(&ports.event_dedup, &*ports.projections, envelope).await {
        Admit::Proceed => {}
        Admit::Duplicate | Admit::Terminal => return,
    }

    match envelope.event_type.as_str() {
        // The agent-session dispatch primitive — owns the spawn / register /
        // exit-handler-attach atom. Detaches the slow prep itself (under
        // DISPATCH_GATE), so it returns to the loop immediately.
        "agent.session.started" => {
            handle_agent_session_started(kernel, envelope, ports).await;
        }
        "agent.session.cancelled" => {
            handle_agent_cancelled(kernel, envelope, ports).await;
        }
        _ => {}
    }
}
