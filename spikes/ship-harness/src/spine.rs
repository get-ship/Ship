//! The verification spine — the differentiator. "Nothing gets blindly accepted."
//!
//! Every consequential tool call flows through `run()`:
//!
//!     Permit (pre)  ->  Execute  ->  Verify (post)
//!
//! This is the `Verify<Permit<Execute>>` decorator from the design, collapsed
//! into one chokepoint so the loop has a single place where outcomes are gated.
//! rig executes tools as the model asks; claw gates on *permission*; Ship gates
//! on *capability + blast radius*. NONE of them verify that the result was real.
//! That post-exec gate is what this spike exists to demonstrate.

use std::fmt;

/// Mirrors Ship's blast-radius tiers (proto/ship/ops/v1/policy.proto:12-25).
/// In production the *permit* check defers to Ship's gate over the peer-cred
/// socket; here we model the tier locally so the shape is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Blast {
    C0Safe,     // reads / own-scope
    C1Space,    // workspace write
    C2Elevated, // dispatch / authoring
    // operator-only — an agent session must NEVER construct this. The dead-code
    // warning is the invariant holding: no agent path reaches C3.
    #[allow(dead_code)]
    C3Critical,
}

/// What the spine is allowed to do, set per-session. Anything above this tier
/// is denied at the permit gate before the side effect runs.
#[derive(Clone, Copy, Debug)]
pub struct Authority {
    pub ceiling: Blast,
}

#[derive(Debug)]
pub enum SpineError {
    /// Permit gate rejected the call (policy / approval / blast ceiling).
    Denied(String),
    /// The side effect itself failed.
    Execute(String),
    /// Side effect "succeeded" but the post-exec check could not confirm the
    /// claimed outcome. This is the case the other harnesses don't have.
    Unverified(String),
}

impl fmt::Display for SpineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpineError::Denied(m) => write!(f, "DENIED: {m}"),
            SpineError::Execute(m) => write!(f, "EXECUTE FAILED: {m}"),
            SpineError::Unverified(m) => write!(f, "UNVERIFIED (refused, not accepted): {m}"),
        }
    }
}
impl std::error::Error for SpineError {}

/// Run a side effect through the three gates.
///
/// * `tier`     — blast radius this call needs.
/// * `auth`     — the session's authority.
/// * `execute`  — the actual side effect (sync closure for spike simplicity).
/// * `verify`   — adversarial post-check: given the execute output, return
///                `Ok(accepted_string)` or `Err(reason)`. An `Err` here means
///                the result is REFUSED and surfaced to the model as a failure,
///                never silently folded into context.
pub fn run<T>(
    gate: &crate::gate::Gate,
    label: &str,
    tier: Blast,
    execute: impl FnOnce() -> Result<T, String>,
    verify: impl FnOnce(&T) -> Result<String, String>,
) -> Result<String, SpineError> {
    // 1. PERMIT — policy + approval + blast ceiling, all in the Gate.
    if let Err(reason) = gate.permit(label, tier) {
        eprintln!("  spine[{label}] permit {tier:?} -> DENIED: {reason}");
        return Err(SpineError::Denied(reason));
    }
    eprintln!("  spine[{label}] permit {tier:?} ok");
    // 2. EXECUTE
    let out = match execute() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("  spine[{label}] execute FAILED: {e}");
            return Err(SpineError::Execute(e));
        }
    };
    // 3. VERIFY — nothing blindly accepted
    match verify(&out) {
        Ok(accepted) => {
            eprintln!("  spine[{label}] verify ok");
            Ok(accepted)
        }
        Err(e) => {
            eprintln!("  spine[{label}] verify REFUSED: {e}");
            Err(SpineError::Unverified(e))
        }
    }
}
