//! The approval diff — the keystone decision of the grant arc.
//!
//! An agent's resolved skills compute a *requested* capability set (the
//! compiler's `requested_grants`); the operator-approved baseline is the
//! *provisioned* set (`provisioned_grants_for`). This module computes the gap
//! between them. The gap is what an install/escalation approval is asked to
//! confer — it is **never auto-granted**; the token mint stays on the approved
//! set (fail-closed). See `.artifacts/grant-resolution-arc-plan.md`.

use std::collections::BTreeSet;

/// Capabilities the agent REQUESTS that are not yet APPROVED — the pending
/// approval delta. Empty = the request is fully covered by the provisioned
/// baseline (nothing to surface). Non-empty = surface for operator approval
/// (install = first sight; escalation = later growth) and mint WITHOUT these.
///
/// Sorted + deduped. Order-independent in both inputs.
pub fn pending_grants(requested: &[String], approved: &[String]) -> Vec<String> {
    let approved: BTreeSet<&str> = approved.iter().map(String::as_str).collect();
    let pending: BTreeSet<String> = requested
        .iter()
        .filter(|r| !approved.contains(r.as_str()))
        .cloned()
        .collect();
    pending.into_iter().collect()
}

/// True when the agent requests at least one capability it isn't approved for —
/// i.e. there is a pending delta to surface. Drift signal: an agent that edits
/// its skills to request more flips this true (detected, never escalated).
pub fn has_pending(requested: &[String], approved: &[String]) -> bool {
    let approved: BTreeSet<&str> = approved.iter().map(String::as_str).collect();
    requested.iter().any(|r| !approved.contains(r.as_str()))
}
