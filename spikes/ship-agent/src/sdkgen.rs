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
//! The op registry here stands in for the proto descriptor set (mid-refactor).

/// Ops exposed by a service domain. Stub for the proto op surface.
fn ops_for(service: &str) -> &'static [&'static str] {
    match service {
        "collections" => &["read", "get", "items", "put", "patch", "delete"],
        "mcp" => &["list", "get", "create", "put", "delete"],
        "runtime" => &["dispatch"],
        _ => &[],
    }
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

    for service in grants {
        let ops = ops_for(service);
        if ops.is_empty() {
            continue;
        }
        if service == "runtime" {
            // Orchestration surfaces as agent(); only present with the grant.
            out.push_str(
                "export function agent(spec, input) { return __call(\"runtime.dispatch\", { spec, input }); }\n",
            );
            continue;
        }
        out.push_str(&format!("export const {service} = {{\n"));
        for op in ops {
            out.push_str(&format!(
                "  {op}: (args) => __call(\"{service}.{op}\", args),\n"
            ));
        }
        out.push_str("};\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(s: &[&str]) -> Vec<String> { s.iter().map(|x| x.to_string()).collect() }

    // The grant determines the surface: a granted service is reachable; an
    // ungranted one is absent. (Behavioral: presence/absence by grant.)
    #[test]
    fn granted_service_present_ungranted_absent() {
        let sdk = generate_sdk(&g(&["collections"]));
        assert!(sdk.contains("collections"), "granted service must be exposed");
        assert!(!sdk.contains("export const mcp"), "ungranted service must not be exposed");
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
}
