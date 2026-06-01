//! Decision types — the verdict vocabulary of the policy primitive.
//!
//! `ScopeViolation` is the fail-closed error returned by `enforce_scope` (ring
//! 2). `Decision`/`AskReason` are the verb-gate (ring 1) verdict types; the
//! `decide` function that produces them lands when `BlastRadius` moves here
//! from `gate.rs`/`ship-ops-rpc` (deferred to a later phase, north-star §2.6).

use thiserror::Error;

/// A scope-enforcement rejection. Returned by `enforce_scope` when an envelope
/// cannot be routed to a single, binding-authorized scope.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScopeViolation {
    /// A Space event carried a space key that does not match the binding's
    /// space — the caller tried to write into another space's log.
    #[error("cross-space emit: binding is space {expected:?}, event keyed {got:?}")]
    CrossSpace {
        expected: String,
        got: String,
    },
    /// A space-unbound caller (`binding.space_id == None`) tried to emit a
    /// Space event. Fail-closed: an unbound caller cannot emit Space data.
    #[error("space-unbound caller cannot emit a Space-scoped event")]
    NoSpaceForSpaceEvent,
}

/// The verb-gate verdict (ring 1). Allow outright, prompt the operator, or deny.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Ask(AskReason),
    Deny(&'static str),
}

/// Why a verb-gate decision is `Ask` — the prompt context shown to the operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AskReason {
    /// The service/operation the principal is invoking.
    pub service: String,
    /// Human-readable summary of what is being requested.
    pub summary: String,
}
