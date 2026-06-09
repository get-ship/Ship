//! Event registrations for the canonical session family.
//!
//! This module registers the 7 canonical session events with the global
//! event registry using the `inventory::submit!` mechanism, making them
//! available to downstream code that calls `event_registry::registry()`.
//!
//! The `register_event!` macro is copied from ship-substrate's payload_registry.rs
//! because it is not exported; it must be re-defined locally here.

use ship_types::event_registry::{EventDef, EventRegistration, EventScope, Retention, Stability};

/// Register one event type with its **allowed-set** of scopes (Model C).
///
/// The scope argument is a square-bracketed list of one-or-more `EventScope`s.
/// A size-1 set (the common case) is the single-scope model: the lone element
/// is both the primary (stamped at `EventEnvelope::new`) and the only scope the
/// emission may ride. Poly-scopic events list every scope they may ride; the
/// emission's binding picks one and `enforce_scope` validates membership +
/// authorization.
macro_rules! register_event {
    // The scopes are passed as a bracketed, comma-separated list of one-or-more
    // `EventScope`s (the allowed-SET, Model C). Captured element-by-element so
    // the array is rebuilt inside the macro — `&[ ... ]` then unsizes to the
    // `&'static [EventScope]` field. A size-1 set is the single-scope model.
    ($name:expr, [$($scope:expr),+ $(,)?], $retention:expr, $stability:expr, $type_name:expr) => {
        inventory::submit! {
            EventRegistration {
                def: || {
                    static DEF: EventDef = EventDef {
                        name: $name,
                        allowed_scopes: &[$($scope),+],
                        retention: $retention,
                        stability: $stability,
                        since_version: None,
                        payload_type_name: $type_name,
                    };
                    &DEF
                },
            }
        }
    };
}

// ── session.* / session.turn.* (canonical — jobs: canonical-session-events +
// workbench-teardown) ──────────────────────────────────────────────────
// The CANONICAL agent-session family (see `crate::canonical_session`).
// The complete ship-shaped session vocabulary: 5 coarse-grain events
// (session.started + 4 terminals) + 2 medium-grain turn events. Scope
// rule: session.* (coarse) → Space/Durable, session.turn.* (medium) →
// Runtime/Ephemeral. The workbench family is RETIRED (R6) — no
// `SessionStarted` workbench payload struct, no `session.attached` etc.
register_event!("session.started", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionStarted");
register_event!("session.completed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCompleted");
register_event!("session.failed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionFailed");
register_event!("session.cancelled", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCancelled");
register_event!("session.suspended", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionSuspended");
register_event!("session.turn.started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnStarted");
register_event!("session.turn.ended", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnEnded");
