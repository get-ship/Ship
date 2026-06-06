//! TypeScript declaration (`.d.ts`) emission for the agent SDK.
//!
//! The agent's tsgo language server (LSP) needs a typed module that mirrors
//! the scoped JS SDK. The two MUST describe the same surface for a given
//! grant — drift between them lets an agent type-check calls that won't
//! resolve at runtime (or worse, makes ungranted ops look callable).
//!
//! The shared surface — `sdkgen::surface(grants)` — is the only place the
//! set of exported names is computed. `generate_dts` consumes it; the test
//! suite asserts the export sets agree. Adding a new service in
//! `sdkgen::ops_for` shows up in BOTH the JS and the `.d.ts` automatically.
//!
//! Typed args stay loose (`unknown`) for now: the spec calls out the surface
//! shape as the load-bearing part; the per-op argument schemas can land
//! alongside the proto descriptor set.

use crate::sdkgen::{surface, SurfaceItem};

/// Generate the scoped SDK `.d.ts` for a set of granted service domains.
/// Mirrors `generate_sdk` exactly: the exported names, and the set of ops
/// per service, are derived from the shared `surface(grants)`.
#[allow(dead_code)] // public API; consumed by the LSP host, not the spike binary
pub fn generate_dts(grants: &[String]) -> String {
    render_dts(&surface(grants), &format!("{grants:?}"))
}

/// Render a `.d.ts` from an explicit surface list. The renderer is pure over
/// `SurfaceItem`s — fed by the shared registry in production, fed by tests
/// with hand-built surfaces that exercise the typed-args path (which the
/// stub registry leaves as `None`).
pub(crate) fn render_dts(items: &[SurfaceItem], grant_label: &str) -> String {
    let mut out = String::new();
    out.push_str("// AUTO-GENERATED scoped SDK typings — grant = ");
    out.push_str(grant_label);
    out.push('\n');
    out.push_str("// The agent's COMPLETE op surface (typed). Soft layer; the gate enforces.\n");
    out.push_str("// Per-op args/returns: named types when the registry carries them,\n");
    out.push_str(
        "// `unknown` otherwise (the stub registry; the proto descriptor will populate).\n",
    );
    out.push_str("declare module \"@ship/agent-sdk\" {\n");

    if items.is_empty() {
        out.push_str("  // no grants — no exports\n");
    } else {
        for item in items {
            match item {
                SurfaceItem::Agent => {
                    out.push_str(
                        "  export function agent(spec: unknown, input: unknown): Promise<unknown>;\n",
                    );
                }
                SurfaceItem::Service(def) => {
                    out.push_str(&format!("  export const {}: {{\n", def.name));
                    for op in def.ops {
                        // Args: render the named shape when the registry carries
                        // one; fall back to `unknown` when it doesn't (the stub
                        // populates `None` until the proto descriptor lands).
                        let args_ty = op.args.unwrap_or("unknown");
                        let ret_ty = op.returns.unwrap_or("unknown");
                        out.push_str(&format!(
                            "    {}(args?: {}): Promise<{}>;\n",
                            op.name, args_ty, ret_ty
                        ));
                    }
                    out.push_str("  };\n");
                }
            }
        }
    }

    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdkgen::{generate_sdk, surface};

    fn g(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    // A granted service is declared in the .d.ts; an ungranted one is not.
    #[test]
    fn granted_service_present_ungranted_absent() {
        let dts = generate_dts(&g(&["collections"]));
        assert!(
            dts.contains("collections"),
            "granted service must be declared"
        );
        assert!(
            !dts.contains("export const mcp:"),
            "ungranted service must not be declared",
        );
    }

    // `agent` is declared ONLY with the runtime grant — spawning is a
    // capability, enforced at the type level too.
    #[test]
    fn agent_declared_only_with_runtime_grant() {
        assert!(!generate_dts(&g(&["collections"])).contains("function agent"));
        assert!(generate_dts(&g(&["runtime"])).contains("function agent"));
    }

    // No grant => no declarations. A comment marks the empty case so a
    // language server still has a (typed) module to resolve.
    #[test]
    fn empty_grant_declares_nothing() {
        let dts = generate_dts(&g(&[]));
        assert!(!dts.contains("export const"));
        assert!(!dts.contains("export function"));
    }

    // The set of exported names in the .d.ts must equal the set in the JS SDK
    // for the same grant. This is the anti-drift property — both generators
    // consume `surface(grants)`, so the equality holds by construction. If you
    // add a new grant mapping to `sdkgen::ops_for`, both pick it up.
    #[test]
    fn dts_and_js_export_names_agree() {
        let grants = g(&["collections", "mcp", "runtime", "not_a_service"]);
        let dts = generate_dts(&grants);
        let sdk = generate_sdk(&grants);

        // The canonical export names (from the shared registry).
        let expected: Vec<&'static str> =
            surface(&grants).iter().map(|i| i.export_name()).collect();

        // Each expected name appears in BOTH outputs (and only as the export
        // form, not as a substring of the grant-list header).
        for name in &expected {
            let dts_needle = if *name == "agent" {
                "function agent".to_string()
            } else {
                format!("export const {name}:")
            };
            let sdk_needle = if *name == "agent" {
                "export function agent".to_string()
            } else {
                format!("export const {name} =")
            };
            assert!(
                dts.contains(&dts_needle),
                ".d.ts must declare `{name}` (needle `{dts_needle}`)",
            );
            assert!(
                sdk.contains(&sdk_needle),
                "JS SDK must export `{name}` (needle `{sdk_needle}`)",
            );
        }
    }

    // Every granted service's ops are typed in the .d.ts.
    #[test]
    fn service_ops_are_typed() {
        let dts = generate_dts(&g(&["collections"]));
        for op in ["read", "get", "items", "put", "patch", "delete"] {
            assert!(
                dts.contains(&format!("{op}(args?: unknown): Promise<unknown>")),
                "collections.{op} must be typed in the .d.ts",
            );
        }
    }

    // When the registry carries per-op args/returns, the renderer must render
    // them as the named TS types (the path the proto descriptor wiring will
    // take). Exercised via `render_dts` directly so the test can construct a
    // surface with `Some(...)` — the stub `ops_for` only emits `None`.
    #[test]
    fn render_with_named_types_uses_them() {
        use crate::sdkgen::{OpDef, ServiceDef, SurfaceItem};
        let svc = ServiceDef {
            name: "collections",
            ops: &[
                OpDef {
                    name: "read",
                    args: Some("{ collection: string; }"),
                    returns: Some("CollectionDetail"),
                },
                OpDef {
                    name: "delete",
                    args: Some("{ id: string; }"),
                    returns: None,
                },
            ],
        };
        let items = [SurfaceItem::Service(svc)];
        let dts = render_dts(&items, "test");

        // Named args + return types render verbatim.
        assert!(
            dts.contains("read(args?: { collection: string; }): Promise<CollectionDetail>;"),
            "named args + return must be rendered verbatim; got:\n{dts}",
        );
        // `Some` for args, `None` for returns: only the return falls back.
        assert!(
            dts.contains("delete(args?: { id: string; }): Promise<unknown>;"),
            "delete must keep the named args shape but fall back on the missing return; got:\n{dts}",
        );
    }
}
