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

/// Blast-radius class — the risk gradient the gate enforces against an agent's
/// ceiling, carried as a proto method option. The canonical home is here (next
/// to the FDS reader that produces it); shipd's gate re-exports it.
///
/// Four tiers, ordered C0 < C1 < C2 < C3 (derive `Ord`): **C0** read (anyone),
/// **C1** workspace-write (any agent), **C2** elevated (granted agents —
/// dispatch/settings/authoring), **C3** critical (operator/2FA only —
/// account/permissions/credentials). `from_descriptor`/unknown fails closed to
/// `C3Critical` — the most restrictive tier — so a misdeclared op is operator-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub enum BlastRadius {
    #[serde(rename = "C0_SAFE")]
    C0Read,
    #[serde(rename = "C1_WORKSPACE")]
    C1Workspace,
    #[serde(rename = "C2_ELEVATED")]
    C2Elevated,
    #[serde(rename = "C3_CRITICAL")]
    C3Critical,
}

impl BlastRadius {
    pub fn from_descriptor(s: &str) -> Self {
        match s {
            "C0_SAFE" | "C0_READ" => Self::C0Read,
            "C1_WORKSPACE" => Self::C1Workspace,
            "C2_ELEVATED" => Self::C2Elevated,
            "C3_CRITICAL" => Self::C3Critical,
            _ => Self::C3Critical,
        }
    }

    pub fn as_descriptor(self) -> &'static str {
        match self {
            Self::C0Read => "C0_SAFE",
            Self::C1Workspace => "C1_WORKSPACE",
            Self::C2Elevated => "C2_ELEVATED",
            Self::C3Critical => "C3_CRITICAL",
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

/// snake_case a PascalCase proto identifier: `WriteFrontmatter` → `write_frontmatter`.
pub fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.char_indices() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// The dotted op name for a proto (service, method) — the SINGLE derivation used
/// by `ship ops call`, gen-skills, and gen-fixtures: the service carries the
/// domain (`RulesService` → `rules`), the method the rest (`List` → `list`) ⇒
/// `rules.list`. The service boundary removes pascal()'s `.`-vs-`_` ambiguity.
pub fn op_name(service: &str, method: &str) -> String {
    let svc = snake(service);
    let domain = svc.strip_suffix("_service").unwrap_or(&svc);
    format!("{domain}.{}", snake(method))
}

/// The op-domain an op name belongs to — its first dotted segment
/// (`git.create` → `git`). The domain is the unit of **capability**: one per
/// per-domain proto service, 1:1 with an sdk-skill (`ship-<domain>`).
pub fn domain_of(op_name: &str) -> &str {
    op_name.split('.').next().unwrap_or(op_name)
}

/// The generated umbrella + streaming services are not capability domains:
/// `ReadOps`/`CommandOps` are the retiring bulk-port scaffold (every op is also
/// on its per-domain service); `StreamService` is server-streaming.
pub fn is_non_domain_service(service: &str) -> bool {
    matches!(service, "ReadOpsService" | "CommandOpsService" | "StreamService")
}

/// Operational domains (#95) are not part of the agent capability set: kernel/
/// actor control (`actor.*`) and raw mesh coordination (`mesh.*`) are
/// operator-surface ABI, not capabilities a user grants an agent.
pub fn is_operational(op_name: &str) -> bool {
    matches!(domain_of(op_name), "actor" | "mesh")
}

/// A grantable Ship capability: one op domain (e.g. `git`), 1:1 with its
/// sdk-skill (`ship-git`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// Domain id, e.g. `git` — what a user grants and what `requires:` names.
    pub id: String,
    /// The sdk-skill that teaches this capability's ops, e.g. `ship-git`.
    pub sdk_skill: String,
}

/// The agent-facing capability registry: every grantable domain, **derived from
/// the proto descriptor** — the single source the sdk-skills are generated from,
/// so the capability list, the sdk-skill set, and the gate's grant axis cannot
/// drift. Excludes the umbrella scaffold and operational domains. Sorted/deduped;
/// 1:1 with the generated sdk-skills. Capability is orthogonal to blast-radius
/// (the internal C0/C1/C2 risk calc).
pub fn capabilities() -> Vec<Capability> {
    let mut domains: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for p in read_policies() {
        if is_non_domain_service(&p.service) {
            continue;
        }
        let name = op_name(&p.service, &p.method);
        if is_operational(&name) {
            continue;
        }
        domains.insert(domain_of(&name).to_string());
    }
    domains
        .into_iter()
        .map(|id| Capability { sdk_skill: format!("ship-{id}"), id })
        .collect()
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

/// Map the proto `BlastRadius` enum number to the gate's tier. Unspecified (0)
/// or unknown → `C3Critical` (**fail closed** to the most restrictive tier: a
/// misdeclared op is operator-only). Reconciles the proto name (`C0_SAFE`) with
/// the gate's `C0Read`.
fn blast_from_proto(n: Option<i32>) -> BlastRadius {
    match n {
        Some(1) => BlastRadius::C0Read,      // BLAST_RADIUS_C0_SAFE
        Some(2) => BlastRadius::C1Workspace, // BLAST_RADIUS_C1_WORKSPACE
        Some(3) => BlastRadius::C2Elevated,  // BLAST_RADIUS_C2_ELEVATED
        Some(4) => BlastRadius::C3Critical,  // BLAST_RADIUS_C3_CRITICAL
        _ => BlastRadius::C3Critical,        // UNSPECIFIED, unknown — fail closed
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

#[cfg(test)]
mod capability_tests {
    use super::*;

    #[test]
    fn capabilities_are_per_domain_and_map_1to1_to_sdk_skills() {
        let caps = capabilities();
        assert!(!caps.is_empty(), "capability registry must derive domains from proto");
        // Known domains present; sdk-skill id is exactly ship-<id>.
        let git = caps.iter().find(|c| c.id == "git").expect("git capability");
        assert_eq!(git.sdk_skill, "ship-git");
        assert!(caps.iter().any(|c| c.id == "pipelines"));
        // Operational domains (#95) and the umbrella scaffold are NOT capabilities.
        assert!(!caps.iter().any(|c| c.id == "actor"));
        assert!(!caps.iter().any(|c| c.id == "mesh"));
        // No duplicates.
        let mut ids: Vec<&str> = caps.iter().map(|c| c.id.as_str()).collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "capability ids are unique");
    }
}
