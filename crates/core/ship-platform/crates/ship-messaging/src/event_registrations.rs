//! Event registrations for the MESSAGING domain.
//!
//! Registers the human.response event with the global event registry
//! using the hoisted `register_event!` macro from `ship_types`.
//!
//! Keystone-A Phase A bundle 4b (2026-06-10): the MESSAGING domain (human-gateway)
//! owns its event at L1. This registration moved from
//! `ship_substrate::events::payload_registry`.
//!
//! Payload struct (HumanResponse) stays in ship_types::payload — it has no
//! external type consumers beyond the L0 registration (only xtask string references),
//! but lives safely in the L0 leaf-types crate without layering concerns.

use ship_types::event_registry::{EventScope, Retention, Stability};
use ship_types::register_event;

// ── human.response ──────────────────────────────────────────────────────────
// Poly-scopic (Model C): a human reply may unblock space work (Space), arrive at
// the runtime ingress before its run is correlated (Runtime), or address the
// account (Account). The webhook ingress emits under Runtime; downstream
// correlation can re-emit under the resolved Space.
register_event!("human.response", [EventScope::Space, EventScope::Runtime, EventScope::Account], Retention::Durable, Stability::Stable, "HumanResponse", ship_types::payload::HumanResponse);
