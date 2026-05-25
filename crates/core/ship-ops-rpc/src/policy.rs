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
use buffa::extension::codecs::{Bool, EnumI32, StringCodec};
use buffa::extension::Extension;
use buffa::ExtensionSet;
use buffa_descriptor::generated::descriptor::FileDescriptorSet;

/// Blast-radius class — the human-vs-agent line the gate enforces, carried as a
/// proto method option. The canonical home is here (next to the FDS reader that
/// produces it); shipd's gate re-exports it. `from_descriptor` fails closed to
/// `C2Elevated` — a misdeclared op is gated to the human, never opened to agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum BlastRadius {
    #[serde(rename = "C0_READ")]
    C0Read,
    #[serde(rename = "C1_WORKSPACE")]
    C1Workspace,
    #[serde(rename = "C2_ELEVATED")]
    C2Elevated,
}

impl BlastRadius {
    pub fn from_descriptor(s: &str) -> Self {
        match s {
            "C0_READ" => Self::C0Read,
            "C1_WORKSPACE" => Self::C1Workspace,
            "C2_ELEVATED" => Self::C2Elevated,
            _ => Self::C2Elevated,
        }
    }

    pub fn as_descriptor(self) -> &'static str {
        match self {
            Self::C0Read => "C0_READ",
            Self::C1Workspace => "C1_WORKSPACE",
            Self::C2Elevated => "C2_ELEVATED",
        }
    }
}

/// `policy.proto` declares these extensions on `google.protobuf.MethodOptions`.
const EXTENDEE: &str = "google.protobuf.MethodOptions";
const CAPABILITY: Extension<StringCodec> = Extension::new(50001, EXTENDEE);
const BLAST_RADIUS: Extension<EnumI32> = Extension::new(50002, EXTENDEE);
const SENSITIVE: Extension<Bool> = Extension::new(50003, EXTENDEE);

/// The gate-relevant policy declared on one rpc method. This is the single
/// decode of the descriptor set — both the gate (keyed by op name) and the
/// Connect-layer introspection map (keyed by wire-path) are built from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodPolicy {
    /// Proto package, e.g. `"ship.ops.v1"`.
    pub package: String,
    /// Proto service name, e.g. `"CommandOpsService"`.
    pub service: String,
    /// Proto rpc name, e.g. `"SkillsCreate"` (PascalCase of the op name).
    pub method: String,
    /// Required capability, e.g. `"ship.skills.write"`. Empty if undeclared.
    pub capability: String,
    /// Blast-radius class driving the gate.
    pub blast: BlastRadius,
    /// Content-exposure flag (redaction / log hygiene); orthogonal to blast.
    pub sensitive: bool,
}

impl MethodPolicy {
    /// The Connect wire-path body, `"<package>.<service>/<method>"`.
    pub fn wire_path(&self) -> String {
        format!("{}.{}/{}", self.package, self.service, self.method)
    }
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
        let package = file.package.clone().unwrap_or_default();
        for service in &file.service {
            let Some(service_name) = service.name.clone() else { continue };
            for method in &service.method {
                let Some(name) = method.name.clone() else { continue };
                // MessageField derefs to the (default-if-absent) MethodOptions;
                // an rpc with no custom options yields None for each read.
                let opts = &*method.options;
                let capability = opts.extension(&CAPABILITY).unwrap_or_default();
                let blast = blast_from_proto(opts.extension(&BLAST_RADIUS));
                let sensitive = opts.extension(&SENSITIVE).unwrap_or(false);
                out.push(MethodPolicy {
                    package: package.clone(),
                    service: service_name.clone(),
                    method: name,
                    capability,
                    blast,
                    sensitive,
                });
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
    fn isolation_reads_are_c0_isolation_read() {
        // The isolation read surface (status/recommendations/preview/verify)
        // migrated off the MCP tool + REST onto IsolationService. Pin the gate
        // contract: pure C0 reads under `ship.isolation.read`.
        let iso: Vec<_> = read_policies()
            .into_iter()
            .filter(|p| p.service == "IsolationService")
            .collect();
        for m in ["Status", "Recommendations", "Preview", "Verify"] {
            let p = iso
                .iter()
                .find(|p| p.method == m)
                .unwrap_or_else(|| panic!("IsolationService.{m} not in descriptor"));
            assert_eq!(p.capability, "ship.isolation.read", "{m} capability");
            assert_eq!(p.blast, BlastRadius::C0Read, "{m} must be C0");
        }
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
