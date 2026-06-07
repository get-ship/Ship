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
//! The descriptor set in `crate::registry` is the SINGLE source of truth for
//! which services exist, what their exported name is, and which methods each
//! one carries — and for each method's blast tier (the gate consults the same
//! set). Both `generate_sdk` (JS) and `generate_dts` (TS declarations) consume
//! `surface(grants)` — derived from `registry::service` — so the two cannot
//! drift: add a new service in `registry` and both generators pick it up.

use crate::registry::{self, Service};

/// One item on the agent's callable surface.
///
/// `Agent` is the orchestration entry point — only present with the `runtime`
/// grant (spawning is a capability, enforced at the type level).
///
/// `Service` is a granted service domain whose methods are exposed as methods
/// of a const namespace.
#[derive(Debug, Clone, Copy)]
pub(crate) enum SurfaceItem<'a> {
    Agent,
    Service(&'a Service),
}

impl<'a> SurfaceItem<'a> {
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
pub(crate) fn surface(grants: &[String]) -> Vec<SurfaceItem<'_>> {
    grants.iter().filter_map(|g| surface_for_one(g)).collect()
}

fn surface_for_one(service: &str) -> Option<SurfaceItem<'_>> {
    let def = registry::service(service)?;
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
                for method in def.methods {
                    out.push_str(&format!(
                        "  {}: (args) => __call(\"{}.{}\", args),\n",
                        method.name, def.name, method.name
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
    // it. If you add a new service to `registry::DESCRIPTOR_SET`, both
    // generators pick it up.
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
