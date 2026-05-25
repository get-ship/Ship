//! Op gate policy sourced from proto (the single source — decision #67 a) via
//! the `ship-ops-rpc` FileDescriptorSet reflection, keyed by op name.
//!
//! This replaces reading blast/capability off the hand-authored
//! `schema::COMMANDS` table. The `reflected_policy_matches_declared_commands`
//! conformance test proves the proto-sourced policy is byte-identical to the
//! Rust-declared one, so the gate switches source with zero behaviour change —
//! the first consumer flipped onto proto-as-SSOT.

use std::collections::HashMap;
use std::sync::LazyLock;

use ship_ops_rpc::policy::{read_policies, MethodPolicy};

/// op name (`skills.create`) → proto rpc method name (`SkillsCreate`). Mirrors
/// `xtask::gen_ops::pascal` — the function that produced the proto method names,
/// so this is how an op name addresses its reflected policy.
fn pascal(op: &str) -> String {
    op.split(['.', '_'])
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

/// rpc method name → its declared policy, read once from the descriptor set.
static BY_METHOD: LazyLock<HashMap<String, MethodPolicy>> =
    LazyLock::new(|| read_policies().into_iter().map(|p| (p.method.clone(), p)).collect());

/// The gate policy declared for an op, by op name. `None` for an unknown op.
pub fn policy_for(op_name: &str) -> Option<&'static MethodPolicy> {
    BY_METHOD.get(&pascal(op_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::gate::BlastRadius;
    use crate::services::schema::COMMANDS;

    /// Proves the proto-sourced policy equals the hand-declared `COMMANDS`
    /// policy for every command — so flipping the gate's source from the Rust
    /// table to reflection is observably a no-op. Guards the emit→embed→reflect
    /// pipeline against drift.
    #[test]
    fn reflected_policy_matches_declared_commands() {
        for cmd in COMMANDS {
            let pol = policy_for(cmd.name)
                .unwrap_or_else(|| panic!("no reflected policy for command `{}`", cmd.name));
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
        }
    }
}
