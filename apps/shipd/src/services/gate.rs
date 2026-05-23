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

/// Blast-radius class of an op — the human-vs-agent line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlastRadius {
    /// Read / own-scope. Harmless to Ship.
    C0Read,
    /// Mutates the caller's own workspace — the user's own risk.
    C1Workspace,
    /// Ship/cloud-dangerous (credential mint, billing). Human-operator only.
    C2Elevated,
}

impl BlastRadius {
    /// Parse the registry descriptor's string form. An unrecognized value is
    /// treated as `C2Elevated` — **fail closed**: a misdeclared op is gated to
    /// the human, never silently opened to agents.
    pub fn from_descriptor(s: &str) -> Self {
        match s {
            "C0_READ" => Self::C0Read,
            "C1_WORKSPACE" => Self::C1Workspace,
            "C2_ELEVATED" => Self::C2Elevated,
            _ => Self::C2Elevated,
        }
    }
}

/// A capability domain — a skill-group. `restricted` domains are
/// capability-defining (they hand out capability) and are never default-granted;
/// an agent reaches them only with an explicit grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Domain {
    pub group: &'static str,
    pub restricted: bool,
}

/// Classify a command name's domain from its `<domain>.<op>` prefix.
///
/// Restricted (capability-defining) groups:
/// - `ship-orchestration` — `agent.*` (dispatch / stop / steer: spawn & control actors)
/// - `ship-authoring`    — `create.*`, `view.*` (write skills / agents / views)
///
/// Ordinary workspace mutations (pipeline, project, mesh, git, annotations) are
/// the default group, open to any in-scope agent.
///
/// **An unrecognized prefix fails closed** to a restricted, ungrantable group:
/// a newly-added op domain that nobody has classified is gated to the operator
/// until someone consciously places it here. The `command_prefixes_are_all_classified`
/// conformance test fails the build if a `COMMANDS` entry lands in this bucket,
/// so the fail-closed path is a tripwire, not a silent denial in production.
pub fn classify(name: &str) -> Domain {
    match name.split('.').next().unwrap_or(name) {
        // restricted — capability-defining
        "agent" => Domain { group: "ship-orchestration", restricted: true },
        "create" | "view" => Domain { group: "ship-authoring", restricted: true },
        // default — ordinary workspace mutation
        "pipeline" | "project" | "mesh" | "git" | "annotation" | "annotations" | "feedback"
        | "rules" => {
            Domain { group: "ship-workspace", restricted: false }
        }
        // unknown — fail closed
        _ => Domain { group: "ship-unclassified", restricted: true },
    }
}

/// The gate's verdict. `Ask` (interactive confirmation, task #82) is a future
/// variant produced by a policy layer above this fn; the gate itself decides
/// only Allow/Deny today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny(&'static str),
}

/// The authenticated caller, reduced to exactly what the gate needs: are they
/// the operator (full trust), and which restricted skill-groups do they hold?
/// Default (non-restricted) groups are implicit — every agent has them — so
/// only restricted grants are listed.
#[derive(Debug, Clone)]
pub struct Caller {
    pub is_operator: bool,
    pub restricted_grants: Vec<String>,
}

/// Map `AuthContext` onto the gate's [`Caller`]. The restricted grants are the
/// real minted set (`auth.grants`), provisioned from the agent's role/skills at
/// session-register — no longer derived from `scopes`. The operator line still
/// reads `Scope::Full` (the operator marker, a separate axis from grants that
/// dies with the REST route-auth conversion in P4).
pub fn caller_from_auth(auth: &AuthContext) -> Caller {
    let is_operator = (matches!(auth.caller_kind, CallerKind::Cli) && auth.agent_id.is_none())
        || auth.has_scope(Scope::Full);
    Caller { is_operator, restricted_grants: auth.grants.clone() }
}

/// The decision. Pure — no I/O, no auth lookups.
///
/// ```text
/// operator                      -> Allow (full trust, all classes)
/// C2                            -> Deny  (agents never; operator handled above)
/// C0/C1, default domain         -> Allow (any in-scope agent)
/// C0/C1, restricted, granted    -> Allow
/// C0/C1, restricted, not granted-> Deny
/// ```
pub fn decide(blast: BlastRadius, domain: &Domain, caller: &Caller) -> Decision {
    if caller.is_operator {
        return Decision::Allow;
    }
    match blast {
        BlastRadius::C2Elevated => Decision::Deny("op requires a human-operator credential"),
        BlastRadius::C0Read | BlastRadius::C1Workspace => {
            if !domain.restricted {
                Decision::Allow
            } else if caller.restricted_grants.iter().any(|g| g == domain.group) {
                Decision::Allow
            } else {
                Decision::Deny("op requires a granted capability (restricted skill-group)")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caller(is_operator: bool, grants: &[&str]) -> Caller {
        Caller {
            is_operator,
            restricted_grants: grants.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn unknown_blast_string_fails_closed_to_c2() {
        assert_eq!(BlastRadius::from_descriptor("nonsense"), BlastRadius::C2Elevated);
        assert_eq!(BlastRadius::from_descriptor("C1_WORKSPACE"), BlastRadius::C1Workspace);
    }

    #[test]
    fn classify_marks_capability_defining_domains_restricted() {
        assert!(classify("agent.dispatch").restricted);
        assert_eq!(classify("agent.dispatch").group, "ship-orchestration");
        assert!(classify("view.tool").restricted);
        assert!(classify("create.surface").restricted);
        assert!(!classify("pipeline.step_done").restricted);
        assert!(!classify("git.commit").restricted);
        assert!(!classify("mesh.send").restricted);
    }

    #[test]
    fn operator_clears_every_class() {
        let op = caller(true, &[]);
        let restricted = classify("agent.dispatch");
        assert_eq!(decide(BlastRadius::C2Elevated, &restricted, &op), Decision::Allow);
        assert_eq!(decide(BlastRadius::C1Workspace, &restricted, &op), Decision::Allow);
    }

    #[test]
    fn agent_gets_default_c1_but_not_c2() {
        let worker = caller(false, &[]);
        let default = classify("pipeline.update_status");
        assert_eq!(decide(BlastRadius::C1Workspace, &default, &worker), Decision::Allow);
        // C2 is human-only regardless of grants.
        assert!(matches!(
            decide(BlastRadius::C2Elevated, &default, &worker),
            Decision::Deny(_)
        ));
    }

    #[test]
    fn command_prefixes_are_all_classified() {
        // Every registered command must classify to a real group — never the
        // fail-closed `ship-unclassified` bucket. If this fails, a new command
        // prefix was added without placing it in `classify` (default vs
        // restricted), which would silently gate (or ungate) it. Classify it.
        for cmd in crate::services::schema::COMMANDS {
            let domain = classify(cmd.name);
            assert_ne!(
                domain.group, "ship-unclassified",
                "command `{}` has an unclassified domain prefix — add it to gate::classify",
                cmd.name
            );
        }
    }

    #[test]
    fn unknown_prefix_fails_closed_for_agents() {
        let agent = caller(false, &[]);
        let unknown = classify("brandnew.op");
        assert!(unknown.restricted);
        assert!(matches!(
            decide(BlastRadius::C1Workspace, &unknown, &agent),
            Decision::Deny(_)
        ));
    }

    #[test]
    fn restricted_domain_needs_the_matching_grant() {
        let worker = caller(false, &[]);
        let commander = caller(false, &["ship-orchestration"]);
        let designer = caller(false, &["ship-authoring"]);
        let dispatch = classify("agent.dispatch");
        let author = classify("view.tool");

        // Worker can't dispatch or author.
        assert!(matches!(decide(BlastRadius::C1Workspace, &dispatch, &worker), Decision::Deny(_)));
        assert!(matches!(decide(BlastRadius::C1Workspace, &author, &worker), Decision::Deny(_)));
        // Commander dispatches but doesn't author.
        assert_eq!(decide(BlastRadius::C1Workspace, &dispatch, &commander), Decision::Allow);
        assert!(matches!(decide(BlastRadius::C1Workspace, &author, &commander), Decision::Deny(_)));
        // Designer authors but doesn't dispatch.
        assert_eq!(decide(BlastRadius::C1Workspace, &author, &designer), Decision::Allow);
        assert!(matches!(decide(BlastRadius::C1Workspace, &dispatch, &designer), Decision::Deny(_)));
    }
}
