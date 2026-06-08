//! The gate — faithful mirror of shipd's real gate (`apps/shipd/src/auth/gate.rs`,
//! `decide`).
//!
//! Pure decision logic, no I/O. The single enforcement point. In production this
//! lives in the daemon and is reached through the ops router (RPC-as-IPC); here it
//! is mirrored so the shape is exercised. A JS sandbox escape cannot bypass it
//! because the real one runs behind the gateway/daemon, not in the isolate.
//!
//!   operator        -> Allow (all tiers)
//!   C0 / C1         -> Allow (baseline for any authenticated agent)
//!   C2 Elevated     -> Allow only if caller granted the op's SERVICE domain
//!   C3 Critical     -> operator-only (human 2FA); an agent never clears it
//!
//! ⚠️ MIRROR-DRIFT (tracked): this hand-written `decide` MUST stay
//! tier-for-tier identical to shipd's `auth::gate::decide`. The two are
//! independent copies today; when the gateway routes to the real in-process gate
//! (RPC-as-IPC, the Phase B seam), this mirror collapses into a thin client and
//! the drift risk goes away. Until then, any change to shipd's decision policy
//! must be reflected here (and vice-versa).

use crate::auth::AuthContext;

/// Blast radius, mirroring proto/ship/ops/v1/policy.proto. Intrinsic to each op;
/// never claimed by spec or request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Blast {
    C0Safe,
    C1Space,
    C2Elevated,
    C3Critical,
}

#[derive(Debug)]
pub enum Decision {
    Allow,
    Deny(String),
}

/// Extract the service domain from a dotted op name: "collections.write" -> "collections".
pub fn service_of(op: &str) -> &str {
    op.split('.').next().unwrap_or(op)
}

/// The one decision. Mirrors shipd's `decide(blast, service, caller)`.
pub fn decide(blast: Blast, service: &str, auth: &AuthContext) -> Decision {
    if auth.operator_cred {
        return Decision::Allow;
    }
    match blast {
        Blast::C3Critical => {
            Decision::Deny("op requires a human-operator credential (2FA)".into())
        }
        Blast::C0Safe | Blast::C1Space => Decision::Allow,
        Blast::C2Elevated => {
            if auth.grants.iter().any(|s| s == service) {
                Decision::Allow
            } else {
                Decision::Deny(format!(
                    "op requires elevation (C2) in service `{service}` — not granted"
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthContext;

    fn agent(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }
    fn operator() -> AuthContext {
        let mut a = agent(&[]);
        a.operator_cred = true;
        a
    }
    fn is_allow(d: Decision) -> bool { matches!(d, Decision::Allow) }
    fn is_deny(d: Decision) -> bool { matches!(d, Decision::Deny(_)) }

    // Reads/writes are baseline — granted to any agent, whatever its grants.
    #[test]
    fn baseline_tiers_need_no_grant() {
        assert!(is_allow(decide(Blast::C0Safe, "any-service", &agent(&[]))));
        assert!(is_allow(decide(Blast::C1Space, "any-service", &agent(&[]))));
    }

    // C2 is gated on the SERVICE being granted — not on identity, not on a tier claim.
    #[test]
    fn c2_requires_the_named_service_grant() {
        assert!(is_deny(decide(Blast::C2Elevated, "svcA", &agent(&["svcB"]))));
        assert!(is_allow(decide(Blast::C2Elevated, "svcA", &agent(&["svcA"]))));
    }

    // C3 is operator-only; grants never reach it.
    #[test]
    fn c3_is_operator_only_never_granted() {
        assert!(is_deny(decide(Blast::C3Critical, "svc", &agent(&["svc"]))));
        assert!(is_allow(decide(Blast::C3Critical, "svc", &operator())));
    }

    // Operator clears every tier.
    #[test]
    fn operator_clears_all_tiers() {
        for b in [Blast::C0Safe, Blast::C1Space, Blast::C2Elevated, Blast::C3Critical] {
            assert!(is_allow(decide(b, "svc", &operator())));
        }
    }

    // service_of derives the domain from the dotted op name.
    #[test]
    fn service_is_the_first_segment() {
        assert_eq!(service_of("svc.method.deep"), "svc");
        assert_eq!(service_of("bare"), "bare");
    }
}
