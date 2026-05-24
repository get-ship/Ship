//! Read op policy (capability + blast radius) out of the embedded
//! `FileDescriptorSet` via buffa reflection.
//!
//! Proto is the single source for op shapes AND policy (decision #67 a): each
//! rpc carries `option (ship.ops.v1.capability)` + `option blast_radius`
//! (`policy.proto`). buffa's normal codegen drops custom options from the
//! generated structs, so the runtime reads them back from the raw descriptor
//! set — `FILE_DESCRIPTOR_SET` is compiled with `--include_imports` and keeps
//! the options. This module is that reader; the gate consumes it instead of a
//! hand-authored Rust table.

use buffa::Message;
use buffa::extension::codecs::{EnumI32, StringCodec};
use buffa::extension::Extension;
use buffa::ExtensionSet;
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use ship_ops::BlastRadius;

/// `policy.proto` declares these extensions on `google.protobuf.MethodOptions`.
const EXTENDEE: &str = "google.protobuf.MethodOptions";
const CAPABILITY: Extension<StringCodec> = Extension::new(50001, EXTENDEE);
const BLAST_RADIUS: Extension<EnumI32> = Extension::new(50002, EXTENDEE);

/// The gate-relevant policy declared on one rpc method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodPolicy {
    /// Proto rpc name, e.g. `"SkillsCreate"` (PascalCase of the op name).
    pub method: String,
    /// Required capability, e.g. `"ship.skills.write"`. Empty if undeclared.
    pub capability: String,
    /// Blast-radius class driving the gate.
    pub blast: BlastRadius,
}

/// Map the proto `BlastRadius` enum number to the gate's class. Unspecified (0)
/// or unknown → `C2Elevated` (**fail closed**: a misdeclared op is gated to the
/// human). Reconciles the proto name (`C0_SAFE`) with the gate's `C0Read`.
fn blast_from_proto(n: Option<i32>) -> BlastRadius {
    match n {
        Some(1) => BlastRadius::C0Read,      // BLAST_RADIUS_C0_SAFE
        Some(2) => BlastRadius::C1Workspace, // BLAST_RADIUS_C1_WORKSPACE
        _ => BlastRadius::C2Elevated,        // C2_ELEVATED, UNSPECIFIED, unknown
    }
}

/// Read every rpc method's policy from the embedded descriptor set.
///
/// Panics only if the embedded `FILE_DESCRIPTOR_SET` fails to decode — that is a
/// build-time corruption, not a runtime condition.
pub fn read_policies() -> Vec<MethodPolicy> {
    let fds = FileDescriptorSet::decode_from_slice(crate::FILE_DESCRIPTOR_SET)
        .expect("embedded FILE_DESCRIPTOR_SET decodes");
    let mut out = Vec::new();
    for file in &fds.file {
        for service in &file.service {
            for method in &service.method {
                let Some(name) = method.name.clone() else { continue };
                // MessageField derefs to the (default-if-absent) MethodOptions;
                // an rpc with no custom options yields None for both reads.
                let opts = &*method.options;
                let capability = opts.extension(&CAPABILITY).unwrap_or_default();
                let blast = blast_from_proto(opts.extension(&BLAST_RADIUS));
                out.push(MethodPolicy { method: name, capability, blast });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(method: &str) -> MethodPolicy {
        read_policies()
            .into_iter()
            .find(|p| p.method == method)
            .unwrap_or_else(|| panic!("method `{method}` not in descriptor set"))
    }

    #[test]
    fn reads_capability_and_blast_from_the_descriptor_set() {
        // Authored in commands.gen.proto today; this proves reflection recovers
        // the custom options buffa drops from the generated structs.
        let create = policy("RulesCreate");
        assert_eq!(create.capability, "ship.rules.write");
        assert_eq!(create.blast, BlastRadius::C1Workspace);
    }

    #[test]
    fn permissions_writes_are_c2_in_the_proto() {
        let p = policy("PermissionsCreate");
        assert_eq!(p.capability, "ship.permissions.write");
        assert_eq!(p.blast, BlastRadius::C2Elevated);
    }

    #[test]
    fn every_method_carries_a_capability() {
        // No rpc may ship without a declared capability — that would be an
        // ungated surface. (Reads + commands both carry one.)
        for p in read_policies() {
            assert!(!p.capability.is_empty(), "rpc `{}` has no capability option", p.method);
        }
    }
}
