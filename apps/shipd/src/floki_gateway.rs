//! ShipdGateway — the L2 adapter that wires floki's `Gateway` trait to the real
//! shipd ops router in-process (`tower::ServiceExt::oneshot`). The floki crate
//! (L1) gains no shipd dependency; this file (L2) is the sole seam.
//!
//! # Design (GATEWAY-BIND plan)
//!
//! **Step 1 — op-name resolution.**
//! A `LazyLock<HashMap<String, String>>` maps every dotted op-name (e.g.
//! `"mcp.create"`) to its Connect wire-path (e.g.
//! `"ship.ops.v1.McpService/Create"`), built once from the embedded
//! `FileDescriptorSet` via `ship_ops_rpc::policy::read_policies()`. An op not
//! present in the map returns `GatewayError::UnknownOp`.
//!
//! **Step 2 — ask-stance pre-check.**
//! For ops in the ask-stance set (`mcp.create`, `mcp.delete`,
//! `account.rotate_key`) on a `First` call, the real shipd gate runs
//! _before_ dispatching to the handler. A hard gate denial
//! (`Decision::Deny`) returns `GatewayError::Denied`; gate clearance returns
//! `GatewayError::Ask` without ever touching the handler. `Approved` calls
//! skip this check (the human already said yes) and dispatch directly.
//!
//! **Step 3 — in-process dispatch.**
//! Build an `axum::http::Request` (POST `/{wire_path}`, JSON body, shipd
//! `AuthContext` extension) and call `tower::ServiceExt::oneshot`. A 403
//! response body maps to `Err(GatewayError::Denied)`; 200 to
//! `Ok(value_with_ok_true)`; anything else to `Err(GatewayError::Dispatch)`.
//!
//! **Spine-verify compatibility.**
//! `ShipOpTool` checks `v.get("ok") == Some(true)` before accepting a result.
//! Real proto handlers return typed shapes without this field. This gateway
//! synthesises `ok: true` into every 200 response body before returning `Ok`.

use std::collections::HashMap;
use std::sync::LazyLock;

use async_trait::async_trait;
use serde_json::{Value, json};
use ship_substrate::events::CallerKind;
use tower::ServiceExt as _;

use crate::auth::{AuthContext as ShipdAuth, gate};
use floki::{
    auth::AuthContext as FlokiAuth,
    gateway::{Gateway, GatewayCallKind, GatewayError},
};

// ── Op-name → wire-path map ───────────────────────────────────────────────────

/// `"mcp.create"` → `"ship.ops.v1.McpService/Create"`.
///
/// Built once from the embedded `FileDescriptorSet`. The umbrella scaffold
/// services (`ReadOpsService` / `CommandOpsService` / `StreamService`) are
/// filtered out — the per-domain service is the SSOT for each op's policy,
/// matching the filter in `ship_ops_rpc::policy::BY_OP_NAME`.
static OP_TO_WIRE: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    use ship_ops_rpc::policy::{is_non_domain_service, op_name, read_policies};
    read_policies()
        .into_iter()
        .filter(|p| !is_non_domain_service(&p.service))
        .map(|p| (op_name(&p.service, &p.method), p.wire_path()))
        .collect()
});

// ── Ask-stance set (mirrors `InProcessGateway::is_ask_stance`) ────────────────

/// Ops that require a human approval even when the grant is present (S11).
/// Matches `floki::gateway::InProcessGateway::is_ask_stance` exactly so the
/// two impls never diverge on policy.
fn is_ask_stance(op: &str) -> bool {
    matches!(op, "mcp.create" | "mcp.delete" | "account.rotate_key")
}

/// Stable-ish request-id for a pending `Ask`. Per-process counter; mirrors
/// the `InProcessGateway` seam.
fn next_request_id(op: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("ask:{op}#{n}")
}

// ── Auth conversion ───────────────────────────────────────────────────────────

/// Convert a floki `AuthContext` to the shipd `AuthContext` that the gate and
/// connectrpc request-extensions expect.
///
/// The gate reads only `grants` + `operator_cred` for C2/C3 enforcement, so
/// those are faithfully threaded. `agent_id` is populated for observability.
/// All other fields are `None`/default — space/workspace/session identity is
/// carried separately via `ToolCtx`, not through the gateway hop.
fn floki_auth_to_shipd(fa: &FlokiAuth) -> ShipdAuth {
    ShipdAuth {
        caller_kind: CallerKind::Mcp,
        agent_id: Some(fa.agent_id.clone()),
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![],
        grants: fa.grants.clone(),
        operator_cred: fa.operator_cred,
    }
}

// ── ShipdGateway ──────────────────────────────────────────────────────────────

/// The real gateway: routes ops through the live shipd ops router via
/// `tower::ServiceExt::oneshot`. No TCP, no token — the `AuthContext` is
/// injected as a request extension, bypassing `auth_middleware` (which
/// skips token validation when the context is already present).
///
/// Construct once at composition time from `ops_rpc::router(state)`, then
/// clone the `Arc<ShipdGateway>` into every floki session closure. The inner
/// `axum::Router` is `Clone + Send + Sync`.
pub struct ShipdGateway {
    router: axum::Router,
}

impl ShipdGateway {
    /// Wrap the router produced by `crate::ops_rpc::router(state)`. Cloning an
    /// `axum::Router` is cheap (Arc internals).
    pub fn new(router: axum::Router) -> Self {
        Self { router }
    }
}

#[async_trait]
impl Gateway for ShipdGateway {
    async fn call(
        &self,
        op: &str,
        args: Value,
        auth: &FlokiAuth,
        kind: GatewayCallKind,
    ) -> Result<Value, GatewayError> {
        // ── Step 1: op-name resolution ────────────────────────────────────────
        // Ops not in the proto descriptor (e.g. runtime.dispatch — no proto RPC
        // yet) are rejected here, never dispatched.
        let wire_path = OP_TO_WIRE
            .get(op)
            .ok_or_else(|| GatewayError::UnknownOp(op.to_string()))?
            .clone();

        // ── Step 2: ask-stance pre-check (First calls only) ───────────────────
        // For ask-stance ops on a first call we run the real gate ourselves so
        // we can return the correct variant:
        //   - gate denies  → GatewayError::Denied  (missing grant / C3)
        //   - gate allows  → GatewayError::Ask      (human approval needed)
        // On an Approved re-dispatch the human already said yes; skip directly
        // to the handler.
        if kind == GatewayCallKind::First && is_ask_stance(op) {
            let shipd_auth = floki_auth_to_shipd(auth);
            let caller = gate::caller_from_auth(&shipd_auth);
            let blast = ship_ops_rpc::policy::policy_for(op)
                .map(|p| p.blast)
                .unwrap_or(ship_ops_rpc::BlastRadius::C3Critical);
            let service = gate::service_of(op);
            return match gate::decide(blast, service, &caller) {
                gate::Decision::Deny(reason) => Err(GatewayError::Denied(reason.to_string())),
                gate::Decision::Allow => Err(GatewayError::Ask {
                    op: op.to_string(),
                    args,
                    request_id: next_request_id(op),
                }),
            };
        }

        // ── Step 3: in-process dispatch ───────────────────────────────────────
        let shipd_auth = floki_auth_to_shipd(auth);
        let body_bytes = serde_json::to_vec(&args)
            .map_err(|e| GatewayError::Dispatch(format!("serialize args: {e}")))?;

        // Build the Connect-JSON unary request. Injecting `AuthContext` as a
        // request extension bypasses `auth_middleware`'s token check (it checks
        // `req.extensions().get::<AuthContext>().is_some()` and fast-paths when
        // already present). The `GateFn` closure inside every wire handler reads
        // `AuthContext` from `RequestContext.extensions`, which connectrpc moves
        // verbatim from the request extensions.
        let req = axum::http::Request::builder()
            .method("POST")
            .uri(format!("/{wire_path}"))
            .header("content-type", "application/json")
            .extension(shipd_auth)
            .body(axum::body::Body::from(body_bytes))
            .map_err(|e| GatewayError::Dispatch(format!("build request: {e}")))?;

        // `Router::Error = Infallible` — oneshot never returns Err.
        let resp = self
            .router
            .clone()
            .oneshot(req)
            .await
            .expect("axum Router service is infallible");

        let status = resp.status().as_u16();
        let bytes = axum::body::to_bytes(resp.into_body(), 4 * 1024 * 1024)
            .await
            .map_err(|e| GatewayError::Dispatch(format!("collect body: {e}")))?;

        // ── Response mapping ──────────────────────────────────────────────────
        if status == 403 {
            // Gate denial from the wire handler's GateFn. Parse the Connect
            // error envelope for the human-readable message.
            let msg = serde_json::from_slice::<Value>(&bytes)
                .ok()
                .and_then(|v| v.get("message").and_then(Value::as_str).map(str::to_string))
                .unwrap_or_else(|| "permission denied".to_string());
            return Err(GatewayError::Denied(msg));
        }

        if status != 200 {
            let msg = String::from_utf8_lossy(&bytes).to_string();
            return Err(GatewayError::Dispatch(format!("http {status}: {msg}")));
        }

        // Synthesise `ok: true` so `ShipOpTool`'s spine-verify check passes.
        // The stub gateway echoed this field; real proto handlers return typed
        // shapes (e.g. `ScopedMutation`, `McpIndexResponse`) without it. We
        // merge it here rather than relaxing the spine check.
        let body: Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            json!({ "raw": String::from_utf8_lossy(&bytes).as_ref() })
        });

        let result = match body {
            Value::Object(mut map) => {
                map.insert("ok".into(), Value::Bool(true));
                Value::Object(map)
            }
            other => json!({ "ok": true, "result": other }),
        };

        Ok(result)
    }
}

// ── Smoke tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use floki::{
        auth::AuthContext as FlokiAuth,
        gateway::{GatewayCallKind, GatewayError},
    };
    use serde_json::Value;

    fn agent(grants: &[&str]) -> FlokiAuth {
        FlokiAuth::in_process("test-agent", grants.iter().map(|s| s.to_string()).collect())
    }

    /// Wiring compiles + ask-stance op with grant → `Ask` (not `Denied`).
    /// This is the anchor test: gate clears (grant present, C2), but the
    /// call is ask-stance so the human must approve before dispatch.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn ask_stance_granted_returns_ask() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw = ShipdGateway::new(crate::ops_rpc::router(harness.state.clone()));

        let r = gw
            .call(
                "mcp.create",
                serde_json::json!({"id":"probe","serverJson":"{}","scope":"project"}),
                &agent(&["mcp"]),
                GatewayCallKind::First,
            )
            .await;

        assert!(
            matches!(r, Err(GatewayError::Ask { ref op, .. }) if op == "mcp.create"),
            "granted C2 ask-stance op must return Ask, got: {r:?}",
        );
    }

    /// Ask-stance op WITHOUT the grant → `Denied` (gate runs before Ask check).
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn ask_stance_ungranted_returns_denied() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw = ShipdGateway::new(crate::ops_rpc::router(harness.state.clone()));

        let r = gw
            .call(
                "mcp.create",
                serde_json::json!({"id":"probe","serverJson":"{}","scope":"project"}),
                &agent(&[]),           // no mcp grant
                GatewayCallKind::First,
            )
            .await;

        assert!(
            matches!(r, Err(GatewayError::Denied(_))),
            "ungranted C2 must be Denied at the gate, got: {r:?}",
        );
    }

    /// C0 read dispatches through the router and returns `ok: true` in the body.
    /// Uses `mcp.list` — confirmed C0 in the proto, no gate call in the handler.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn c0_op_dispatches_ok_with_ok_true() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw = ShipdGateway::new(crate::ops_rpc::router(harness.state.clone()));

        let r = gw
            .call(
                "mcp.list",
                serde_json::json!({}),
                &agent(&[]),           // C0 — no grant required
                GatewayCallKind::First,
            )
            .await;

        assert!(
            matches!(r, Ok(ref v) if v.get("ok") == Some(&Value::Bool(true))),
            "C0 dispatch must succeed with ok:true, got: {r:?}",
        );
    }

    /// An op absent from the proto descriptor returns `UnknownOp` — never
    /// silently dispatched.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unknown_op_returns_unknown_op() {
        let harness = crate::test_harness::TestDaemon::new().await;
        let gw = ShipdGateway::new(crate::ops_rpc::router(harness.state.clone()));

        let r = gw
            .call("nope.nope", serde_json::json!({}), &agent(&[]), GatewayCallKind::First)
            .await;

        assert!(matches!(r, Err(GatewayError::UnknownOp(_))));
    }
}
