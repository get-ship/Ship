//! Dispatch entry gate — the domain-free primitive every dispatch path
//! (the orchestrator, the `service.agents` actor) hits before handing an
//! envelope to domain policy.
//!
//! Two layers:
//! - [`dedup`] — sliding-window event-id deduplication. Mailbox redelivery
//!   and replay races must not advance state twice for the same envelope.
//! - [`router`] — terminal-state guard, operator-event exemption, and the
//!   [`DISPATCH_GATE`] semaphore that bounds concurrent agent-session
//!   dispatch preps.
//!
//! All of this is L0 (substrate), so both L1 callers depend on it directly
//! without a cargo cycle.

pub mod dedup;
pub mod router;

pub use dedup::{EventDedup, Seen};
pub use router::{Admit, DISPATCH_GATE, extract_slug, is_operator_event, terminal_guard};
