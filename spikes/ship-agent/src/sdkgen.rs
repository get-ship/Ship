//! Scoped SDK generation — `proto ∩ grant` -> the typed op surface the agent
//! codes against. Slice 2 emits JS (no transpile); the typed `.d.ts` for the
//! tsgo LSP is the same shape with signatures, generated alongside later.
//!
//! The generated module is the agent's COMPLETE callable op surface: it exposes
//! ONLY granted service domains. This is the SOFT layer — one-shot success +
//! token economy + LSP-as-tool-list. It is NOT the security boundary: the gate
//! in the gateway is. A script can still hand-call `op_ship_call` for an
//! ungranted op; the gate denies it there.
//!
//! `ops_for` is the SINGLE source of truth for which services exist, what their
//! exported name is, and which ops each one carries. Both `generate_sdk` (JS)
//! and `generate_dts` (TS declarations) consume `surface(grants)` — derived
//! from `ops_for` — so the two cannot drift: add a new service in one place
//! and both generators pick it up.
//!
//! `ops_for` stands in for the proto descriptor set (mid-refactor).

/// One op on a service. The `args` and `returns` fields are the per-op
/// TypeScript types the LSP will surface — rendered as the named type when
/// `Some`, and as `unknown` when `None`. The stub registry (this file)
/// populates both as `None`; the proto descriptor wiring will populate them
/// from the source of truth. Either way, the renderer is the same.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpDef {
    pub(crate) name: &'static str,
    /// TS shape of the args object, e.g. `"{ collection: string; }"`.
    /// `None` => renderer emits `args?: unknown`.
    pub(crate) args: Option<&'static str>,
    /// TS return type, e.g. `"CollectionDetail"`. `None` => renderer emits
    /// `Promise<unknown>`.
    pub(crate) returns: Option<&'static str>,
}

/// A recognized service domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ServiceDef {
    /// The exported identifier (`collections`, `mcp`, `runtime`).
    pub(crate) name: &'static str,
    /// The ops this service exposes, in declaration order.
    pub(crate) ops: &'static [OpDef],
}

/// The op registry. Look up the canonical def for a service name. `None` for
/// unknown grants — they contribute nothing to the surface. Stands in for the
/// proto descriptor set; the `args`/`returns` on each op are `None` until the
/// proto wiring lands.
pub(crate) fn ops_for(service: &str) -> Option<ServiceDef> {
    match service {
        "collections" => Some(ServiceDef {
            name: "collections",
            ops: &[
                OpDef {
                    name: "read",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "get",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "items",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "put",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "patch",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "delete",
                    args: None,
                    returns: None,
                },
            ],
        }),
        "mcp" => Some(ServiceDef {
            name: "mcp",
            ops: &[
                OpDef {
                    name: "list",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "get",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "create",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "put",
                    args: None,
                    returns: None,
                },
                OpDef {
                    name: "delete",
                    args: None,
                    returns: None,
                },
            ],
        }),
        "runtime" => Some(ServiceDef {
            name: "runtime",
            ops: &[OpDef {
                name: "dispatch",
                args: None,
                returns: None,
            }],
        }),
        "lsp" => Some(ServiceDef {
            name: "lsp",
            ops: &[
                OpDef { name: "diagnostics", args: None, returns: None },
                OpDef { name: "hover", args: None, returns: None },
            ],
        }),
        _ => None,
    }
}

/// One item on the agent's callable surface.
///
/// `Agent` is the orchestration entry point — only present with the `runtime`
/// grant (spawning is a capability, enforced at the type level).
///
/// `Service` is a granted service domain whose ops are exposed as methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfaceItem {
    Agent,
    Service(ServiceDef),
}

impl SurfaceItem {
    /// The top-level exported identifier for this item (`agent`, `collections`,
    /// `mcp`, …). Both the JS SDK and the `.d.ts` expose exactly this set for
    /// a given grant — derived from the same registry so the two cannot drift.
    #[allow(dead_code)] // used by dtsgen's anti-drift test under `cargo test`
    pub(crate) fn export_name(&self) -> &'static str {
        match self {
            SurfaceItem::Agent => "agent",
            SurfaceItem::Service(def) => def.name,
        }
    }
}

/// Canonical surface for a grant set. The SINGLE source of truth consumed by
/// both `generate_sdk` (JS) and `generate_dts` (TS declarations).
pub(crate) fn surface(grants: &[String]) -> Vec<SurfaceItem> {
    grants.iter().filter_map(|g| surface_for_one(g)).collect()
}

fn surface_for_one(service: &str) -> Option<SurfaceItem> {
    let def = ops_for(service)?;
    if def.name == "runtime" {
        // The runtime grant surfaces as the orchestration entry point, not
        // as a `runtime.dispatch` method — spawning is a capability, not a
        // service call. The service def still exists for the gate / proto.
        return Some(SurfaceItem::Agent);
    }
    Some(SurfaceItem::Service(def))
}

/// Generate the scoped SDK JS module for a set of granted service domains.
/// Exposes one namespace per granted service; `runtime` surfaces `agent()` (the
/// `runtime.dispatch` op) — so an agent without the `runtime` grant literally
/// cannot SEE `agent()`, keeping "spawning is a capability" true at the type level.
pub fn generate_sdk(grants: &[String]) -> String {
    let mut out = String::new();
    out.push_str("// AUTO-GENERATED scoped SDK — grant = ");
    out.push_str(&format!("{grants:?}\n"));
    out.push_str("// The agent's COMPLETE op surface. Soft layer; the gate enforces.\n");
    out.push_str("const __call = (op, args) => Deno.core.ops.op_ship_call(op, args ?? {});\n\n");

    for item in surface(grants) {
        match item {
            SurfaceItem::Agent => {
                out.push_str(
                    "export function agent(spec, input) { return __call(\"runtime.dispatch\", { spec, input }); }\n",
                );
            }
            SurfaceItem::Service(def) => {
                out.push_str(&format!("export const {} = {{\n", def.name));
                for op in def.ops {
                    out.push_str(&format!(
                        "  {}: (args) => __call(\"{}.{}\", args),\n",
                        op.name, def.name, op.name
                    ));
                }
                out.push_str("};\n");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    // The grant determines the surface: a granted service is reachable; an
    // ungranted one is absent. (Behavioral: presence/absence by grant.)
    #[test]
    fn granted_service_present_ungranted_absent() {
        let sdk = generate_sdk(&g(&["collections"]));
        assert!(
            sdk.contains("collections"),
            "granted service must be exposed"
        );
        assert!(
            !sdk.contains("export const mcp"),
            "ungranted service must not be exposed"
        );
    }

    // Orchestration (agent()) appears ONLY with the runtime grant — spawning is
    // a capability, enforced at the surface level too.
    #[test]
    fn agent_surfaces_only_with_runtime_grant() {
        assert!(!generate_sdk(&g(&["collections"])).contains("function agent"));
        assert!(generate_sdk(&g(&["runtime"])).contains("function agent"));
    }

    // No grant => no surface to call.
    #[test]
    fn empty_grant_exposes_nothing_callable() {
        let sdk = generate_sdk(&g(&[]));
        assert!(!sdk.contains("export const"));
        assert!(!sdk.contains("export function"));
    }

    // `surface()` is the single source of truth — the JS export set must equal
    // it. If you add a new service in `ops_for`, both generators pick it up.
    #[test]
    fn js_exports_match_surface() {
        let grants = g(&["collections", "mcp", "runtime"]);
        let sdk = generate_sdk(&grants);
        for item in surface(&grants) {
            let needle = match item {
                SurfaceItem::Agent => "export function agent".to_string(),
                SurfaceItem::Service(def) => format!("export const {}", def.name),
            };
            assert!(
                sdk.contains(&needle),
                "JS SDK must expose `{needle}` for the granted surface",
            );
        }
    }

    // Unknown grants are silently dropped — same as the JS behavior. The
    // grant list appears in the header comment, so assert against the
    // EXPORT form, not bare substrings.
    #[test]
    fn unknown_grant_does_not_appear() {
        let sdk = generate_sdk(&g(&["not_a_service"]));
        assert!(!sdk.contains("export const not_a_service"));
        assert!(!sdk.contains("export function not_a_service"));
        assert!(!sdk.contains("export const"));
        assert!(!sdk.contains("export function"));
    }
}
