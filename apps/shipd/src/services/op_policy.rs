//! Op gate policy sourced from proto (the single source — decision #67 a) via
//! the `ship-ops-rpc` FileDescriptorSet reflection, keyed by op name.
//!
//! This is a thin re-export of `ship_ops_rpc::policy::policy_for` — the SINGLE
//! op-name→policy derivation, keyed by the canonical `op_name(service, method)`
//! that `ship ops call`, gen-skills, and the coverage gate all use. The gate
//! consumed a local `pascal()` map before, but `pascal("permissions.create")`
//! ("PermissionsCreate") never matched the per-domain services' bare method
//! names ("Create") after #78 deleted the PascalCase op-monolith proto — so
//! `policy_for` returned `None` for every C2 command and the gate fell OPEN
//! (task #314). Delegating to the one op-name lookup closes that gap and keeps a
//! single source: no parallel name mangling lives here.

pub use ship_ops_rpc::policy::{policy_for, MethodPolicy};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::gate::BlastRadius;
    use crate::services::schema::COMMANDS;

    /// Where a command IS on a per-domain proto service, the proto-sourced
    /// policy must match the hand-declared `COMMANDS` blast + capability —
    /// proving the op-name lookup the gate uses reflects the catalog without
    /// drift. Commands not yet on a proto service (`actor.*`, `mesh.*`,
    /// `project.*`, `create.surface`, …) are gated by the `COMMANDS.blast_radius`
    /// fallback in `command::blast_for`; they're covered there, not here.
    #[test]
    fn reflected_policy_matches_declared_commands() {
        let mut checked = 0;
        for cmd in COMMANDS {
            let Some(pol) = policy_for(cmd.name) else { continue };
            assert_eq!(
                pol.blast,
                BlastRadius::from_descriptor(cmd.blast_radius),
                "blast mismatch for `{}`",
                cmd.name
            );
            assert_eq!(
                pol.capability, cmd.permission,
                "capability mismatch for `{}`",
                cmd.name
            );
            checked += 1;
        }
        assert!(checked > 0, "expected at least some commands on proto services");
    }

    /// SECURITY (#314): every C2 command that DOES carry a proto policy resolves
    /// to operator-authority through the op-name lookup the gate uses (never to a
    /// lower tier or absent). This is the shipd-side mirror of the registry
    /// conformance test `every_c2_op_resolves_to_operator_authority`. C2 commands
    /// without a proto service are covered by `command::every_c2_command_is_gated`.
    #[test]
    fn proto_backed_c2_commands_resolve_to_operator_authority() {
        for cmd in COMMANDS {
            if BlastRadius::from_descriptor(cmd.blast_radius) != BlastRadius::C2Elevated {
                continue;
            }
            if let Some(p) = policy_for(cmd.name) {
                assert_eq!(
                    p.blast,
                    BlastRadius::C2Elevated,
                    "C2 command `{}` resolved to {:?}",
                    cmd.name,
                    p.blast
                );
            }
        }
    }

    /// The fail-closed contract: an unknown / unpoliced op has no proto policy.
    /// The gate's `blast_for` returns `None` for a truly unknown name (denied by
    /// fall-through to `unknown_command`), and a tier for every known command.
    #[test]
    fn unknown_op_has_no_proto_policy() {
        assert!(policy_for("permissions.not_a_real_op").is_none());
        assert!(policy_for("nonsense.nope").is_none());
    }
}
