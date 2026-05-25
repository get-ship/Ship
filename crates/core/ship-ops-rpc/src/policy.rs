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
const DESCRIPTION: Extension<StringCodec> = Extension::new(50004, EXTENDEE);

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
    /// Simple (package-stripped) request message type, e.g. `"CreateRuleRequest"`.
    pub input_type: String,
    /// Simple (package-stripped) response message type, e.g. `"ScopedMutation"`.
    /// `"JsonResponse"` marks an op not yet on a typed proto shape — no fixture
    /// to validate. The generators + the contract gate key off this.
    pub output_type: String,
    /// One-line human description (the `description` method option). The SSOT
    /// for sdk-skill + help doc text; empty if the rpc declares none.
    pub description: String,
    /// Request fields, derived from the input message — the op's args. Lets
    /// sdk-skill + help generation read arg shapes from proto, not the registry.
    pub args: Vec<MethodArg>,
    /// Response fields, derived from the output message — the return schema.
    /// Empty for opaque returns (e.g. `JsonResponse`). Powers OpenAPI-level
    /// reference docs (the response *shape*, not just its type name).
    pub returns_fields: Vec<MethodArg>,
}

/// One request field, projected to the arg-kind vocabulary the generators use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodArg {
    pub name: String,
    /// `"string" | "number" | "bool" | "map" | "<base>[]"` (repeated).
    pub kind: String,
}

/// Strip a descriptor type ref (`.ship.ops.v1.ScopedMutation`) to its simple
/// name (`ScopedMutation`).
fn simple_type(t: &str) -> String {
    t.rsplit('.').next().unwrap_or(t).to_string()
}

/// Project a proto field's (type, label) to the generators' arg-kind vocab.
/// Field type numbers per `descriptor.proto`'s `FieldDescriptorProto.Type`;
/// label 3 = repeated.
fn field_kind(ty: Option<i32>, label: Option<i32>) -> String {
    let base = match ty {
        Some(8) => "bool",                       // BOOL
        Some(9) | Some(12) | Some(14) => "string", // STRING | BYTES | ENUM
        Some(10) | Some(11) => "map",            // GROUP | MESSAGE (nested object)
        _ => "number",                            // all numeric scalar types
    };
    if label == Some(3) && base != "map" {
        format!("{base}[]")
    } else {
        base.to_string()
    }
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
    // Message (simple name) → its fields, projected to arg kinds. Lets a method
    // resolve its request fields (= args) from the descriptor.
    let mut messages: std::collections::HashMap<String, Vec<MethodArg>> =
        std::collections::HashMap::new();
    for file in &fds.file {
        for msg in &file.message_type {
            let Some(mname) = msg.name.clone() else { continue };
            let args = msg
                .field
                .iter()
                .filter_map(|f| {
                    f.name
                        .clone()
                        .map(|name| MethodArg {
                            name,
                            kind: field_kind(f.r#type.map(|t| t as i32), f.label.map(|l| l as i32)),
                        })
                })
                .collect();
            messages.insert(mname, args);
        }
    }
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
                let input_type = simple_type(method.input_type.as_deref().unwrap_or_default());
                let output_type = simple_type(method.output_type.as_deref().unwrap_or_default());
                let description = opts.extension(&DESCRIPTION).unwrap_or_default();
                let args = messages.get(&input_type).cloned().unwrap_or_default();
                let returns_fields = messages.get(&output_type).cloned().unwrap_or_default();
                out.push(MethodPolicy {
                    package: package.clone(),
                    service: service_name.clone(),
                    method: name,
                    capability,
                    blast,
                    sensitive,
                    input_type,
                    output_type,
                    description,
                    args,
                    returns_fields,
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
    fn every_method_carries_a_capability() {
        // No rpc may ship without a declared capability — that would be an
        // ungated surface. (Reads + commands both carry one.)
        for p in read_policies() {
            assert!(!p.capability.is_empty(), "rpc `{}` has no capability option", p.method);
        }
    }
}
