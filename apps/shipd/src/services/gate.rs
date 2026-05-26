//! The one capability gate — a pure decision shared by every write dispatch.
//!
//! Spec: `.artifacts/auth-gate-synthesis.md`. The model is **caller × blast ×
//! skill-grant**, explainable in a paragraph:
//!
//! > Reads (C0) are open to anyone. Workspace writes (C1) are open to any
//! > in-scope agent. The ops that *hand out capability* — dispatching/steering
//! > agents, authoring skills/agents/views — live in **restricted skill-groups
//! > that aren't granted by default**, and no agent can confer more than it
//! > holds. Cloud/credential-dangerous ops (C2) are human-only.
//!
//! C1 enforcement is happy-path hardening that eliminates an accident class —
//! NOT a Tier-1 security boundary (a host agent shares the uid). The real
//! boundary is C2 (credential-location) + attenuation at mint (slice B).
//!
//! This module is pure: [`decide`] takes a policy + a [`Caller`] and returns a
//! [`Decision`]. Every write surface resolves through it via `command::call`,
//! so there is one gate, not one per transport.

use ship_substrate::events::CallerKind;

use crate::auth::AuthContext;
use crate::auth::scopes::Scope;

/// Blast-radius class of an op — the human-vs-agent line. The canonical
/// definition lives in `ship-ops-rpc` (next to the FDS policy reader that
/// produces it); re-exported so every `gate::BlastRadius` reference keeps
/// working. `from_descriptor` fails closed to `C2Elevated`.
pub use ship_ops_rpc::BlastRadius;

/// The **service** an op belongs to — its op-domain (`git.commit` → `git`), the
/// existing proto-service curation. This is the *scope* axis of authority: an
/// agent's grant is the set of services it's **elevated** in. Derived via
/// `ship_ops_rpc::policy::domain_of` — no second taxonomy.
pub fn service_of(name: &str) -> &str {
    ship_ops_rpc::policy::domain_of(name)
}

/// The gate's verdict. `Ask` (interactive confirmation, task #82) is a future
/// variant produced by a policy layer above this fn; the gate itself decides
/// only Allow/Deny today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny(&'static str),
}

/// The authenticated caller, reduced to what the gate needs: are they the
/// operator (full trust), and which **services they're elevated in** (the C2
/// set). The C1 baseline is implicit — every agent has workspace-write
/// everywhere — so only the elevated (C2) services are listed.
#[derive(Debug, Clone)]
pub struct Caller {
    pub is_operator: bool,
    pub elevated_services: Vec<String>,
}

/// Map `AuthContext` onto the gate's [`Caller`]. `auth.grants` carries the
/// agent's elevated services (the C2 set), provisioned per-agent (#85). The
/// operator line reads `Scope::Full` (the operator marker, a separate axis that
/// dies with the REST route-auth conversion in P4).
pub fn caller_from_auth(auth: &AuthContext) -> Caller {
    let is_operator = (matches!(auth.caller_kind, CallerKind::Cli) && auth.agent_id.is_none())
        || auth.has_scope(Scope::Full);
    Caller { is_operator, elevated_services: auth.grants.clone() }
}

/// The decision. Pure — no I/O, no auth lookups. `service` is the op's domain
/// (`service_of`); `blast` its risk tier.
///
/// ```text
/// operator                          -> Allow (full trust, all tiers)
/// C3 critical                       -> Deny  (operator/2FA only; never an agent)
/// C0 read                           -> Allow (observation is non-harmful, #95)
/// C1 workspace                      -> Allow (the operational baseline — any agent)
/// C2 elevated, service in grant     -> Allow
/// C2 elevated, service not granted  -> Deny
/// ```
///
/// The model is **service × blast-tier with a default-C1 ceiling**: every agent
/// gets C0/C1 everywhere (functions out of the box); C2 needs the op's service
/// in the agent's elevated set; C3 is operator-only.
pub fn decide(blast: BlastRadius, service: &str, caller: &Caller) -> Decision {
    if caller.is_operator {
        return Decision::Allow;
    }
    match blast {
        BlastRadius::C3Critical => Decision::Deny("op requires a human-operator credential (2FA)"),
        BlastRadius::C0Read | BlastRadius::C1Workspace => Decision::Allow,
        BlastRadius::C2Elevated => {
            if caller.elevated_services.iter().any(|s| s == service) {
                Decision::Allow
            } else {
                Decision::Deny("op requires elevation (C2) in this service — not granted")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caller(is_operator: bool, elevated: &[&str]) -> Caller {
        Caller {
            is_operator,
            elevated_services: elevated.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn unknown_blast_string_fails_closed_to_most_restrictive() {
        // Fail closed to the highest tier — a misdeclared op is operator-only.
        assert_eq!(BlastRadius::from_descriptor("nonsense"), BlastRadius::C3Critical);
        assert_eq!(BlastRadius::from_descriptor("C1_WORKSPACE"), BlastRadius::C1Workspace);
        assert_eq!(BlastRadius::from_descriptor("C0_SAFE"), BlastRadius::C0Read);
        // Ordering: the gate's `blast <= ceiling` relies on this.
        assert!(BlastRadius::C0Read < BlastRadius::C1Workspace);
        assert!(BlastRadius::C2Elevated < BlastRadius::C3Critical);
    }

    #[test]
    fn service_is_the_op_domain() {
        assert_eq!(service_of("git.commit"), "git");
        assert_eq!(service_of("pipelines.dispatch"), "pipelines");
    }

    #[test]
    fn operator_clears_every_tier() {
        let op = caller(true, &[]);
        assert_eq!(decide(BlastRadius::C3Critical, "account", &op), Decision::Allow);
        assert_eq!(decide(BlastRadius::C2Elevated, "pipelines", &op), Decision::Allow);
    }

    #[test]
    fn baseline_agent_gets_c0_c1_everywhere_not_c2_or_c3() {
        // The operational baseline: any agent reads + workspace-writes in any
        // service, out of the box, with no elevated services.
        let worker = caller(false, &[]);
        assert_eq!(decide(BlastRadius::C0Read, "settings", &worker), Decision::Allow);
        assert_eq!(decide(BlastRadius::C1Workspace, "git", &worker), Decision::Allow);
        assert_eq!(decide(BlastRadius::C1Workspace, "pipelines", &worker), Decision::Allow);
        // C2 needs elevation in that service; C3 is operator-only.
        assert!(matches!(decide(BlastRadius::C2Elevated, "pipelines", &worker), Decision::Deny(_)));
        assert!(matches!(decide(BlastRadius::C3Critical, "account", &worker), Decision::Deny(_)));
    }

    #[test]
    fn c2_allowed_only_in_elevated_services() {
        // A commander elevated in `pipelines` can run that service's C2 ops
        // (dispatch) but not C2 elsewhere — per-service scoping.
        let commander = caller(false, &["pipelines"]);
        assert_eq!(decide(BlastRadius::C2Elevated, "pipelines", &commander), Decision::Allow);
        assert!(matches!(decide(BlastRadius::C2Elevated, "settings", &commander), Decision::Deny(_)));
        // C1 everywhere still holds (baseline); C3 still operator-only even in
        // an elevated service.
        assert_eq!(decide(BlastRadius::C1Workspace, "settings", &commander), Decision::Allow);
        assert!(matches!(decide(BlastRadius::C3Critical, "pipelines", &commander), Decision::Deny(_)));
    }

    #[test]
    fn every_c2_command_service_is_grantable() {
        // Service membership only *gates* C2 ops — C0/C1 are allowed regardless
        // (baseline), C3 is always operator-only. So the invariant that matters:
        // every **C2** command's service is reachable by *some* role's elevated
        // set — otherwise it's permanently un-grantable (silently operator-only).
        // We check the grantable union (what `grants_for_role` confers), which
        // covers command-only authoring domains (`view`/`create`) the read-derived
        // registry doesn't carry.
        use ship_platform::auth::scopes::grants_for_role;
        let mut grantable: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for role in [Some("commander"), Some("designer"), Some("contributor")] {
            grantable.extend(grants_for_role(role));
        }
        // Operational domains (#95) are operator-surface, never agent-grantable.
        let operational = |s: &str| matches!(s, "actor" | "mesh" | "meta" | "permissions");
        for cmd in crate::services::schema::COMMANDS {
            if BlastRadius::from_descriptor(cmd.blast_radius) != BlastRadius::C2Elevated {
                continue;
            }
            let svc = service_of(cmd.name);
            assert!(
                grantable.contains(svc) || operational(svc),
                "C2 command `{}` service `{}` is not grantable by any role — \
                 add it to a role in grants_for_role or re-tier it",
                cmd.name,
                svc
            );
        }
    }
}
