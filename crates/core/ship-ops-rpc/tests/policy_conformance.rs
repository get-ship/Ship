//! Security conformance for the op-policy registry (task #314, blast tiers #73).
//!
//! The command-path gate resolves an op's policy by op name through
//! `ship_ops_rpc::policy::policy_for` — the SINGLE op-name→policy derivation,
//! keyed by the canonical `op_name(service, method)`. Before #314 the daemon
//! gate used a separate `pascal()` derivation (`permissions.create` →
//! `PermissionsCreate`) that never matched the per-domain services' bare method
//! names (`Create`) after #78 deleted the PascalCase op-monolith proto — so
//! `policy_for` returned `None` for every C2 command and the gate fell OPEN.
//!
//! These tests pin the closed state: every C2 op resolves to operator-authority,
//! and an unknown op resolves to `None` (the gate inverts that to deny).

use ship_ops_rpc::policy::{is_non_domain_service, op_name, policy_for, read_policies};
use ship_ops_rpc::BlastRadius;

/// Every op the descriptor classifies `C2_ELEVATED` (operator-authority — the
/// highest agent-reachable blast radius) MUST resolve through the op-name keyed
/// `policy_for`, and resolve back to C2. An op that does not resolve returns
/// `None`, which the command-path gate would treat as ungated (fail OPEN) — the
/// exact #314 gap. This test fails listing every orphaned C2 op.
#[test]
fn every_c2_op_resolves_to_operator_authority() {
    let mut orphans = Vec::new();
    let mut mistiered = Vec::new();
    for p in read_policies() {
        if is_non_domain_service(&p.service) || p.blast != BlastRadius::C2Elevated {
            continue;
        }
        let op = op_name(&p.service, &p.method);
        match policy_for(&op) {
            None => orphans.push(op),
            Some(resolved) if resolved.blast != BlastRadius::C2Elevated => {
                mistiered.push(format!("{op} -> {:?}", resolved.blast));
            }
            Some(_) => {}
        }
    }
    assert!(
        orphans.is_empty(),
        "C2 ops orphaned from the policy map (gate fails OPEN for these): {orphans:?}"
    );
    assert!(mistiered.is_empty(), "C2 ops that resolved to the wrong tier: {mistiered:?}");
}

/// The fail-closed contract the gate relies on: an op with no proto policy
/// resolves to `None`. The gate inverts this to DENY (never allow-on-None).
#[test]
fn unknown_op_has_no_policy() {
    assert!(policy_for("nonsense.does_not_exist").is_none());
    assert!(policy_for("permissions.not_a_real_op").is_none());
}

/// The canonical C2 ops named in the #314 security gap each resolve to
/// operator-authority through the op-name lookup the gate uses.
#[test]
fn named_c2_orphans_now_resolve() {
    for op in [
        "permissions.create",
        "permissions.delete",
        "mcp.create",
        "mcp.delete",
        "skills.create",
        // agents-arc E6: the canonical service is plural `AgentsService`, so the
        // proto op-names are `agents.*` (re-aligned with registry/SDK/CLI).
        "agents.create",
        "agents.delete",
        "agents.dispatch",
        "agents.stop",
    ] {
        let p = policy_for(op).unwrap_or_else(|| panic!("`{op}` must resolve to a policy"));
        assert_eq!(p.blast, BlastRadius::C2Elevated, "`{op}` must be C2");
    }
}

/// `agents.grant` hands out authority — it is C3 (operator-only), a tier ABOVE
/// C2. The plural op-name (agents-arc E6) resolves against the AgentsService
/// descriptor.
#[test]
fn agent_grant_resolves_to_critical() {
    let p = policy_for("agents.grant").expect("agents.grant must resolve");
    assert_eq!(p.blast, BlastRadius::C3Critical);
}
