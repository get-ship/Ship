//! Dispatch entry gate — the domain-free primitive every agent-session
//! dispatch path hits before handing an envelope to domain policy.
//!
//! Two layers:
//! - [`dedup`] — sliding-window event-id deduplication. Mailbox redelivery
//!   and replay races must not advance state twice for the same envelope.
//! - [`router`] — terminal-state guard, operator-event exemption, and the
//!   dispatch-gate semaphore constructor ([`new_dispatch_gate`]) that
//!   bounds concurrent agent-session dispatch preps. The semaphore is
//!   owned by the composition root and injected downward — not ambient.
//!
//! Lives in `ship-agents` (the only real consumer family). The composition
//! root (shipd) constructs the gate and injects it via [`crate::ports::AgentPorts`].

pub mod dedup;
pub mod router;

pub use dedup::{EventDedup, Seen};
pub use router::{
    Admit, DISPATCH_GATE_PERMITS, extract_slug, is_operator_event, new_dispatch_gate,
    terminal_guard,
};
