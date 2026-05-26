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

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn request_fully_covered_has_no_pending() {
        let requested = v(&["ship-orchestration"]);
        let approved = v(&["ship-orchestration", "ship-authoring"]);
        assert!(pending_grants(&requested, &approved).is_empty());
        assert!(!has_pending(&requested, &approved));
    }

    #[test]
    fn unapproved_request_surfaces_as_pending() {
        let requested = v(&["ship-orchestration", "ship-authoring"]);
        let approved = v(&["ship-orchestration"]);
        assert_eq!(pending_grants(&requested, &approved), v(&["ship-authoring"]));
        assert!(has_pending(&requested, &approved));
    }

    #[test]
    fn no_approval_yet_means_all_requested_are_pending() {
        let requested = v(&["ship-authoring", "ship-orchestration"]);
        assert_eq!(
            pending_grants(&requested, &[]),
            v(&["ship-authoring", "ship-orchestration"]) // sorted
        );
    }

    #[test]
    fn empty_request_is_never_pending() {
        assert!(pending_grants(&[], &v(&["ship-orchestration"])).is_empty());
        assert!(!has_pending(&[], &[]));
    }

    #[test]
    fn order_and_dupes_do_not_matter() {
        let requested = v(&["b", "a", "b"]);
        let approved = v(&["a"]);
        assert_eq!(pending_grants(&requested, &approved), v(&["b"]));
    }
}
