//! The op registry — a single, descriptor-driven source of truth.
//!
//! S8 collapses the two hand-written match arms (`sdkgen::ops_for` and
//! `InProcessGateway::blast_of`) into one descriptor set. Both consumers —
//! `sdkgen` / `dtsgen` for the surface, `gateway` for the blast — read from
//! `services()` (the descriptor set) and the lookup helpers below. Adding a
//! new op in one place is now mechanically impossible: there is no other
//! place to forget it.
//!
//! The shape is a contract-faithful fake of a proto `FileDescriptorSet`:
//! services carry methods; each method carries a `blast` option (intrinsic to
//! the op, never claimed by spec or request). The real proto descriptor
//! wiring to shipd is a later, refactor-gated slice — this module is the seam
//! it will replace.

use crate::gate::Blast;

/// A single method on a service domain. The `blast` field is the method's
/// proto option (intrinsic to the op, mirrors `policy.proto`); the `args` /
/// `returns` are the TS shapes the LSP will surface.
#[derive(Debug, Clone, Copy)]
pub struct Method {
    pub name: &'static str,
    /// TS shape of the args object, e.g. `"{ collection: string; }"`.
    /// `None` => renderer emits `args?: unknown`.
    pub args: Option<&'static str>,
    /// TS return type, e.g. `"CollectionDetail"`. `None` => renderer emits
    /// `Promise<unknown>`.
    pub returns: Option<&'static str>,
    /// The op's intrinsic blast radius. Mirrors a proto method option;
    /// intrinsic to the op, never claimed by spec or request.
    pub blast: Blast,
}

/// A recognized service domain. The descriptor for one `FileDescriptorProto`
/// service: a name and an ordered list of methods.
#[derive(Debug, Clone, Copy)]
pub struct Service {
    /// The exported identifier (`collections`, `mcp`, `runtime`, `lsp`).
    pub name: &'static str,
    pub methods: &'static [Method],
}

// -----------------------------------------------------------------------------
// The descriptor set.
//
// Stands in for a real `FileDescriptorSet` shipped from shipd. The blast
// values on each method mirror `proto/ship/ops/v1/policy.proto`. When the
// real descriptor wiring lands, this `static` is the only thing that
// disappears — every consumer reads through the lookup helpers and is
// unaffected.
// -----------------------------------------------------------------------------

const COLLECTIONS: Service = Service {
    name: "collections",
    methods: &[
        Method { name: "read", args: None, returns: None, blast: Blast::C0Safe },
        Method { name: "get", args: None, returns: None, blast: Blast::C0Safe },
        Method { name: "items", args: None, returns: None, blast: Blast::C0Safe },
        Method { name: "put", args: None, returns: None, blast: Blast::C1Space },
        Method { name: "patch", args: None, returns: None, blast: Blast::C1Space },
        Method { name: "delete", args: None, returns: None, blast: Blast::C1Space },
    ],
};

const MCP: Service = Service {
    name: "mcp",
    methods: &[
        // MCP server config-management ops (C2, ask-stance). Registering or
        // removing a third-party MCP server is sensitive, so the gateway's
        // S11 Ask layer (`is_ask_stance`) requires human approval for these.
        //
        // NOTE: `mcp.call`-as-ship-op (the dynamic-blast tool-invocation path)
        // was deliberately CUT from Floki — it was a parallel per-call gate path
        // for a foreign protocol (the antipattern). Floki reaches foreign tools
        // through the harness/driver layer, never by re-exposing MCP as a ship
        // op. Only the config ops remain, and only as parked config CRUD.
        Method { name: "create", args: None, returns: None, blast: Blast::C2Elevated },
        Method { name: "delete", args: None, returns: None, blast: Blast::C2Elevated },
    ],
};

const RUNTIME: Service = Service {
    name: "runtime",
    methods: &[
        Method { name: "dispatch", args: None, returns: None, blast: Blast::C2Elevated },
        Method { name: "parallel", args: None, returns: None, blast: Blast::C2Elevated },
    ],
};

const LSP: Service = Service {
    name: "lsp",
    methods: &[
        // Read-only analysis — baseline.
        Method { name: "diagnostics", args: None, returns: None, blast: Blast::C0Safe },
        Method { name: "hover", args: None, returns: None, blast: Blast::C0Safe },
    ],
};

const ACCOUNT: Service = Service {
    name: "account",
    methods: &[
        Method { name: "rotate_key", args: None, returns: None, blast: Blast::C3Critical },
    ],
};

/// The full descriptor set — the SINGLE source of truth for which services
/// exist, what their methods are, and what blast each method carries. The
/// `account` service is intentionally NOT exposed in the agent SDK surface
/// (it is operator-only, never granted to an agent) — it is in the
/// descriptor set so the gateway rejects its ops as `C3Critical` at the
/// gate. The descriptor set is a superset of the surface.
const DESCRIPTOR_SET: &[Service] = &[COLLECTIONS, MCP, RUNTIME, LSP, ACCOUNT];

/// The full descriptor set, in declaration order. Drives `surface(grants)`.
pub fn services() -> &'static [Service] {
    DESCRIPTOR_SET
}

/// Look up a service domain by name (`"collections"`, `"mcp"`, …). `None`
/// for unknown services — they contribute nothing to the surface.
pub fn service(name: &str) -> Option<&'static Service> {
    DESCRIPTOR_SET.iter().find(|s| s.name == name)
}

/// Look up a method on a service by its short name (`"read"`, `"get"`, …).
/// `None` if the service or method is unknown.
pub fn method(service_name: &str, op_name: &str) -> Option<&'static Method> {
    service(service_name)?.methods.iter().find(|m| m.name == op_name)
}

/// Intrinsic blast for an op by its fully-qualified name
/// (`"collections.read"`, `"account.rotate_key"`). `None` for unknown ops.
/// In production this is read from the proto method option; here it is
/// derived from the descriptor set.
pub fn blast_of(op: &str) -> Option<Blast> {
    let (svc, method_name) = op.split_once('.')?;
    method(svc, method_name).map(|m| m.blast)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The descriptor set is the SINGLE source of truth. Every op that the
    // gateway can dispatch (and the surface can surface) is in it; nothing
    // else is. The blast tier on the wire is the tier declared here.
    #[test]
    fn descriptor_set_carries_every_known_op_with_a_blast() {
        // Each service declares at least one method.
        for s in services() {
            assert!(!s.methods.is_empty(), "service `{}` has no methods", s.name);
        }
        // The known op set is exactly the descriptor set.
        let known: Vec<String> = services()
            .iter()
            .flat_map(|s| s.methods.iter().map(move |m| format!("{}.{}", s.name, m.name)))
            .collect();
        for op in &known {
            assert!(
                blast_of(op).is_some(),
                "descriptor set declares `{op}` but blast_of returns None",
            );
        }
    }

    // Surface and gate agree: an op present in the descriptor set surfaces
    // AND has a blast. An op NOT in the descriptor set has neither.
    #[test]
    fn surface_presence_and_blast_agree_on_descriptor_membership() {
        // Pick a representative cross-section of the descriptor set.
        let known_ops = [
            "collections.read",   // C0
            "collections.put",    // C1
            "mcp.create",         // C2 (config op, ask-stance)
            "runtime.dispatch",   // C2
            "runtime.parallel",   // C2
            "account.rotate_key", // C3
        ];
        for op in known_ops {
            let (svc, m) = op.split_once('.').unwrap();
            assert!(method(svc, m).is_some(), "known op `{op}` must resolve");
            assert!(blast_of(op).is_some(), "known op `{op}` must have a blast");
        }

        // An op NOT in the descriptor set: surface lookup is `None` AND
        // blast lookup is `None`. (Surface != SDK; the SDK is built from
        // `surface(grants)`; both lookups consult the descriptor set.)
        let unknown = "nope.nope";
        let (svc, m) = unknown.split_once('.').unwrap();
        assert!(method(svc, m).is_none());
        assert!(blast_of(unknown).is_none());
    }

    // Blast tiers in the descriptor set match the canonical policy: reads
    // are baseline (C0), writes are space (C1), domain-elevation is C2,
    // operator-only is C3. Asserted as a property, not a literal number
    // for a specific op — the property is what the gate enforces.
    #[test]
    fn blast_tiers_match_canonical_policy() {
        // C0 reads
        assert_eq!(blast_of("collections.read"), Some(Blast::C0Safe));
        assert_eq!(blast_of("collections.items"), Some(Blast::C0Safe));
        assert_eq!(blast_of("lsp.diagnostics"), Some(Blast::C0Safe));
        // C1 writes
        assert_eq!(blast_of("collections.put"), Some(Blast::C1Space));
        assert_eq!(blast_of("collections.delete"), Some(Blast::C1Space));
        // C2 elevated
        assert_eq!(blast_of("mcp.create"), Some(Blast::C2Elevated));
        assert_eq!(blast_of("runtime.dispatch"), Some(Blast::C2Elevated));
        assert_eq!(blast_of("runtime.parallel"), Some(Blast::C2Elevated));
        // C3 operator-only
        assert_eq!(blast_of("account.rotate_key"), Some(Blast::C3Critical));
    }
}
