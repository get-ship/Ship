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
    // The final argument is the payload TYPE PATH: the registration carries
    // its own schema validator (Phase 4a — deserialize-into-the-declared-type).
    // The string form ($type_name) stays as the stable identifier the
    // fixture/codegen tools key on.
    ($name:expr, [$($scope:expr),+ $(,)?], $retention:expr, $stability:expr, $type_name:expr, $payload:ty) => {
        inventory::submit! {
            EventRegistration {
                def: || {
                    fn validate(json: &str) -> Result<(), String> {
                        serde_json::from_str::<$payload>(json)
                            .map(|_| ())
                            .map_err(|e| e.to_string())
                    }
                    static DEF: EventDef = EventDef {
                        name: $name,
                        allowed_scopes: &[$($scope),+],
                        retention: $retention,
                        stability: $stability,
                        since_version: None,
                        payload_type_name: $type_name,
                        validate_payload: validate,
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
register_event!("session.started", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionStarted", crate::canonical_session::SessionStarted);
register_event!("session.completed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCompleted", crate::canonical_session::SessionCompleted);
register_event!("session.failed", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionFailed", crate::canonical_session::SessionFailed);
register_event!("session.cancelled", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionCancelled", crate::canonical_session::SessionCancelled);
register_event!("session.suspended", [EventScope::Space], Retention::Durable, Stability::Stable, "canonical_session::SessionSuspended", crate::canonical_session::SessionSuspended);
register_event!("session.turn.started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnStarted", crate::canonical_session::TurnStarted);
register_event!("session.turn.ended", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "canonical_session::TurnEnded", crate::canonical_session::TurnEnded);

// ── op.* (fine-grained) — Runtime-scoped, Ephemeral ────────────────────────────
// R2 hook conversion: the op.* family is the designated replacement for the
// legacy `hook.*` family. The hook→op conversion and hook.* deregistration are
// tracked in #25b and have NOT landed — these registrations are the additive
// contract only; hook ingestion still emits hook.* and nothing emits op.* today.
// Fine-grained (per tool-call) so Runtime + Ephemeral — same scope rule as
// `session.turn.*`, just one more level of granularity.
register_event!("op.tool_use_started", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseStarted", crate::op::OpToolUseStarted);
register_event!("op.tool_use_completed", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseCompleted", crate::op::OpToolUseCompleted);
register_event!("op.tool_use_denied", [EventScope::Runtime], Retention::Ephemeral, Stability::Stable, "op::OpToolUseDenied", crate::op::OpToolUseDenied);
