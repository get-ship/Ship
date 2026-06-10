//! The ONE door — `command::call`.
//!
//! Every effect an agent causes enters here, carrying the agent's AuthContext.
//! This is the only place the gate runs. The rquickjs/deno_core host function is
//! a WIRE to this door, not a second gate and not a backdoor: it forwards the
//! call; the gateway decides. In-process vs over-socket is just transport — the
//! invariant is "enter at the gateway with real auth, let the gate decide."
//!
//! `InProcessGateway` is the seam: it carries no policy beyond looking up the
//! op's intrinsic (service, blast) and running the gate. The real impl wraps
//! shipd's `services::command::call`; this stub stands in while that surface is
//! mid-refactor, and the agent runtime is written against the seam, unchanged.
//!
//! One small branch consults the MCP registry for `mcp.call` (S-mcp v1, design
//! §4): the per-call blast lives on the registry, not the static table, because
//! the registry is dynamic. The branch is the ONLY dynamic-blast path; every
//! other op still resolves through `registry::blast_of`. The gate decision
//! itself is unchanged — `gate::decide` is never touched.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::auth::AuthContext;
use crate::gate::{self, Blast, Decision};
use crate::mcp::McpRegistry;
use crate::registry;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("DENIED: {0}")]
    Denied(String),
    #[error("unknown op `{0}`")]
    UnknownOp(String),
    #[error("dispatch failed: {0}")]
    Dispatch(String),
    /// The op is `Ask`-stance: the gate would allow it, but a human must
    /// approve this particular call. The tool layer routes this through
    /// the `Approver` handshake. `request_id` correlates the eventual
    /// `UserInput` decision back to this op.
    #[error("ASK: {op} (id={request_id})")]
    Ask { op: String, args: Value, request_id: String },
}

/// How a tool is invoking the gateway. First-time callers go through the
/// ask-stance check; an approved re-dispatch (post-`Approver::request`)
/// sets `Approved` so the gate's stance check is skipped — the approval
/// has already been granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayCallKind {
    /// First-time call: ask-stance ops return `Err(Ask)`.
    First,
    /// Re-dispatch after approval was granted: skip the ask-stance check
    /// (the human already said yes); the gate still runs.
    Approved,
}

/// The single door. Async because real dispatch is async (and ops fan out on
/// Tokio under `parallel`).
#[async_trait]
pub trait Gateway: Send + Sync {
    async fn call(
        &self,
        op: &str,
        args: Value,
        auth: &AuthContext,
        kind: GatewayCallKind,
    ) -> Result<Value, GatewayError>;
}

/// Stub gateway: the op, service, and blast all come from the in-process
/// `registry` module — the SAME descriptor set the SDK + `.d.ts` surface use
/// (S8). In production the descriptor set is loaded from shipd's
/// `FileDescriptorSet`; the seam is the same lookup helpers. The gateway runs
/// the REAL gate decision, then echoes the result.
///
/// For `mcp.call` (S-mcp v1), the per-call blast comes from the optional
/// `McpRegistry` seam — a `None` registry means MCP isn't wired on this
/// gateway, and `mcp.call` is unknown. The service-grant gate and the
/// per-server allow-list (the second deny-by-default axis) both run here;
/// the gate decision itself is unchanged.
pub struct InProcessGateway {
    pub mcp_registry: Option<Arc<dyn McpRegistry>>,
}

impl Default for InProcessGateway {
    fn default() -> Self {
        Self { mcp_registry: None }
    }
}

impl InProcessGateway {
    /// Construct a gateway with an MCP registry wired. The registry is
    /// the dynamic source for `mcp.call`'s blast; without it, the op is
    /// unknown at this gateway.
    pub fn with_mcp(registry: Arc<dyn McpRegistry>) -> Self {
        Self { mcp_registry: Some(registry) }
    }
}

impl InProcessGateway {
    /// Intrinsic human-in-the-loop stance for an op (S11). Ops in this set are
    /// allowed by the gate (when the capability is granted) but the
    /// `Ask` layer must also approve them. This is policy of the *call*,
    /// not of the agent — it never elevates the agent or clears C3.
    fn is_ask_stance(op: &str) -> bool {
        matches!(op, "mcp.create" | "mcp.delete" | "account.rotate_key")
    }

    /// Allocate a stable-ish request id for a pending `Ask`. Per-process
    /// counter; unique within a single run. Production would key this
    /// on the daemon's id minting — this is the seam, not the policy.
    fn next_request_id(op: &str) -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        format!("ask:{op}#{n}")
    }
}

#[async_trait]
impl Gateway for InProcessGateway {
    async fn call(
        &self,
        op: &str,
        args: Value,
        auth: &AuthContext,
        kind: GatewayCallKind,
    ) -> Result<Value, GatewayError> {
        // MCP special case (S-mcp): per-call blast is dynamic and lives on the
        // registry. server/tool extraction, blast lookup, gate decision,
        // per-server allow check, and dispatch all live in `call_mcp`. No other
        // op takes this branch; the static `registry::blast_of` path below is
        // unchanged. mcp.call is C2 (not ask-stance), so it short-circuits ahead
        // of the Ask check — its second deny-by-default axis is the allow-list.
        if op == "mcp.call" {
            return self.call_mcp(&args, auth).await;
        }

        // Intrinsic blast — read from the descriptor set (S8), never claimed by
        // spec or request. `None` => unknown op (not in the descriptor set).
        let blast = registry::blast_of(op).ok_or_else(|| GatewayError::UnknownOp(op.to_string()))?;
        let service = gate::service_of(op);

        // THE GATE — the only enforcement point.
        match gate::decide(blast, service, auth) {
            Decision::Deny(reason) => return Err(GatewayError::Denied(reason)),
            Decision::Allow => {}
        }

        // Ask stance: gate would allow, but a human must approve this call.
        // Skip on a post-approval re-dispatch: the human already said yes.
        // The op still passes through the gate; the Ask layer does not
        // bypass it.
        if kind == GatewayCallKind::First && Self::is_ask_stance(op) {
            return Err(GatewayError::Ask {
                op: op.to_string(),
                args: args.clone(),
                request_id: Self::next_request_id(op),
            });
        }

        // Stub dispatch: a real gateway routes to the domain service. We echo so
        // the round-trip + gate decision are observable.
        Ok(json!({
            "op": op,
            "service": service,
            "blast": format!("{blast:?}"),
            "caller": auth.agent_id,
            "args": args,
            "ok": true,
        }))
    }
}

impl InProcessGateway {
    /// Dispatch `mcp.call`. The blast tier is intrinsic to the op
    /// (C2 by design — v1 freezes it; the registry is consulted for
    /// defense-in-depth so a future C3-via-MCP cannot sneak through).
    /// The service-grant gate runs first; the per-server allow-list
    /// second; the client's `call_tool` third. Denials are data, never
    /// exceptions.
    async fn call_mcp(
        &self,
        args: &Value,
        auth: &AuthContext,
    ) -> Result<Value, GatewayError> {
        let registry = self
            .mcp_registry
            .as_ref()
            .ok_or_else(|| GatewayError::UnknownOp("mcp.call".into()))?;

        // The args shape is fixed: { server, tool, args }. A missing or
        // non-string `server`/`tool` is a client error, not a denial —
        // the gate never sees it.
        let server = args.get("server").and_then(Value::as_str).ok_or_else(|| {
            GatewayError::Dispatch("mcp.call: `server` (string) is required".into())
        })?;
        let tool = args.get("tool").and_then(Value::as_str).ok_or_else(|| {
            GatewayError::Dispatch("mcp.call: `tool` (string) is required".into())
        })?;
        let call_args = args
            .get("args")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));

        // Per-call blast, sourced from the registry. The blast is
        // intrinsic to the op AND clamped to <=C2 (the only tier the
        // gate ever grants to a non-operator caller). v1 is C2 for
        // every known (server, tool); v2 reads MCP `annotations` to
        // pick a lower tier.
        let blast = registry
            .blast_of(server, tool)
            .ok_or_else(|| GatewayError::UnknownOp(format!("mcp.call({server}.{tool})")))?;
        if blast == Blast::C3Critical {
            return Err(GatewayError::Denied(
                "MCP tool can never clear C3 (no self-elevation)".into(),
            ));
        }

        // THE GATE — service-grant axis. C2 needs `mcp` in grants; the
        // `gate::decide` call here is the only enforcement on that
        // axis. Identical to every other C2 op.
        match gate::decide(blast, "mcp", auth) {
            Decision::Deny(reason) => return Err(GatewayError::Denied(reason)),
            Decision::Allow => {}
        }

        // Per-server / per-tool allow-list — the SECOND deny-by-default
        // axis. A configured-but-not-allowed server yields a denial
        // whose reason names the server (per design §5). The gate
        // itself does not move.
        if !registry.is_allowed(server, tool) {
            return Err(GatewayError::Denied(format!(
                "MCP server `{server}` is not on this agent's mcp.allow list"
            )));
        }

        // Dispatch: hand the call to the configured client's
        // `call_tool`. A real client (v2) frames JSON-RPC; the v1 fake
        // returns a canned text result. The spine verifier accepts the
        // canned shape (text content, isError=false) and refuses
        // anything else.
        let client = registry
            .client_of(server)
            .ok_or_else(|| GatewayError::UnknownOp(format!("mcp.call({server}.{tool})")))?;
        let result = client.call_tool(tool, call_args.clone()).await.map_err(|e| {
            GatewayError::Dispatch(format!("mcp.call({server}.{tool}): {e}"))
        })?;

        Ok(json!({
            "op": "mcp.call",
            "service": "mcp",
            "blast": format!("{blast:?}"),
            "caller": auth.agent_id,
            "server": server,
            "tool": tool,
            "args": call_args,
            "result": result,
            "ok": true,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthContext;
    use crate::mcp::{InMemoryMcpRegistry, McpServerSpec, McpToolDef};
    use serde_json::json;

    fn agent(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    fn fs_spec() -> McpServerSpec {
        McpServerSpec {
            id: "fs".into(),
            transport: crate::mcp::McpTransport::Stdio {
                cmd: "echo".into(),
                args: vec![],
                env: vec![],
            },
            tools: vec![McpToolDef {
                server: "fs".into(),
                name: "read".into(),
                title: None,
                description: String::new(),
                blast: Blast::C2Elevated,
            }],
        }
    }

    // A baseline op succeeds for any agent.
    #[tokio::test]
    async fn baseline_op_succeeds() {
        let r = InProcessGateway::default()
            .call("collections.read", serde_json::json!({}), &agent(&[]), GatewayCallKind::First)
            .await;
        assert!(r.is_ok());
    }

    // A C2 op in an ungranted service is denied AT THE GATEWAY (the one door).
    #[tokio::test]
    async fn ungranted_elevated_op_is_denied() {
        let gw = InProcessGateway::with_mcp(Arc::new(InMemoryMcpRegistry::new(
            vec![fs_spec()],
            vec!["fs".into()],
            std::collections::HashMap::new(),
        )));
        let r = gw
            .call(
                "mcp.call",
                json!({"server": "fs", "tool": "read", "args": {}}),
                &agent(&["collections"]),
                GatewayCallKind::First,
            )
            .await;
        assert!(matches!(r, Err(GatewayError::Denied(_))));
    }

    // The same op, with the service granted, becomes `Ask`-stance: the gate
    // allows the capability, but a human must approve this call. The op
    // still passes through the gate; the ask is layered on top, not a
    // bypass. A `mcp.create` is in the ask-stance set.
    #[tokio::test]
    async fn granting_the_service_makes_it_ask_stance() {
        // mcp.create is a C2 config op AND ask-stance: granted service ⇒ the
        // gate allows, but the Ask layer requires approval for this call.
        let r = InProcessGateway::default()
            .call("mcp.create", serde_json::json!({}), &agent(&["mcp"]), GatewayCallKind::First)
            .await;
        assert!(matches!(r, Err(GatewayError::Ask { ref op, .. }) if op == "mcp.create"));
    }

    // An approved re-dispatch SKIPS the ask-stance check: the gate still
    // runs (and would deny an ungranted op), but the stance layer is
    // already-resolved. This proves the Ask path is layered on top of
    // the gate, not a bypass.
    #[tokio::test]
    async fn approved_redispatch_skips_ask_stance() {
        let r = InProcessGateway::default()
            .call("mcp.create", serde_json::json!({}), &agent(&["mcp"]), GatewayCallKind::Approved)
            .await;
        assert!(r.is_ok(), "approved re-dispatch must succeed for a granted op");
    }

    // mcp.call (tool invocation, NOT ask-stance) with the service granted and
    // the server allow-listed succeeds — its second axis is the allow-list.
    #[tokio::test]
    async fn granting_the_service_allows_the_op() {
        let gw = InProcessGateway::with_mcp(Arc::new(InMemoryMcpRegistry::new(
            vec![fs_spec()],
            vec!["fs".into()],
            std::collections::HashMap::new(),
        )));
        let r = gw
            .call(
                "mcp.call",
                json!({"server": "fs", "tool": "read", "args": {"path": "/x"}}),
                &agent(&["mcp"]),
                GatewayCallKind::First,
            )
            .await;
        assert!(r.is_ok(), "expected ok; got: {r:?}");
    }

    // An unknown op is rejected, never silently accepted.
    #[tokio::test]
    async fn unknown_op_is_rejected() {
        let r = InProcessGateway::default()
            .call("nope.nope", serde_json::json!({}), &agent(&[]), GatewayCallKind::First)
            .await;
        assert!(matches!(r, Err(GatewayError::UnknownOp(_))));
    }

    // `mcp.call` with no registry wired is unknown at this gateway —
    // the seam is opt-in.
    #[tokio::test]
    async fn mcp_call_without_registry_is_unknown() {
        let r = InProcessGateway::default()
            .call(
                "mcp.call",
                json!({"server": "fs", "tool": "read"}),
                &agent(&["mcp"]),
                GatewayCallKind::First,
            )
            .await;
        assert!(matches!(r, Err(GatewayError::UnknownOp(_))));
    }

    // DESIGN §5 — agent WITH `mcp` but no matching `mcp.allow` entry is
    // denied at the registry layer; the denial reason names the server.
    #[tokio::test]
    async fn granted_mcp_but_unallowed_server_is_denied_with_server_named() {
        // Server is configured but the agent's allow-list is empty ⇒
        // the registry's `is_allowed` returns false. The gateway
        // surfaces the second deny-by-default as a normal denial, with
        // the server id in the reason.
        let gw = InProcessGateway::with_mcp(Arc::new(InMemoryMcpRegistry::new(
            vec![fs_spec()],
            vec![], // no servers allowed for this agent
            std::collections::HashMap::new(),
        )));
        let r = gw
            .call(
                "mcp.call",
                json!({"server": "fs", "tool": "read"}),
                &agent(&["mcp"]),
                GatewayCallKind::First,
            )
            .await;
        match r {
            Err(GatewayError::Denied(reason)) => {
                assert!(
                    reason.contains("fs"),
                    "denial reason must name the server; got: {reason}",
                );
            }
            other => panic!("expected denied, got: {other:?}"),
        }
    }

    // DESIGN §5 — agent WITHOUT `mcp` grant is denied at the gate for
    // `mcp.call`. The denial reason names the missing service grant,
    // not the server (the service-grant gate runs FIRST).
    #[tokio::test]
    async fn without_mcp_grant_call_is_denied_at_the_gate() {
        let gw = InProcessGateway::with_mcp(Arc::new(InMemoryMcpRegistry::new(
            vec![fs_spec()],
            vec!["fs".into()],
            std::collections::HashMap::new(),
        )));
        let r = gw
            .call(
                "mcp.call",
                json!({"server": "fs", "tool": "read"}),
                &agent(&["collections"]),
                GatewayCallKind::First,
            )
            .await;
        match r {
            Err(GatewayError::Denied(reason)) => {
                assert!(
                    reason.contains("mcp"),
                    "service-grant denial must name the missing service; got: {reason}",
                );
            }
            other => panic!("expected denied, got: {other:?}"),
        }
    }

    // The allowed path: a server-named call returns a `result` whose
    // shape the spine verifier accepts (text content, isError=false).
    #[tokio::test]
    async fn allowed_call_returns_a_verifier_acceptable_result() {
        let gw = InProcessGateway::with_mcp(Arc::new(InMemoryMcpRegistry::new(
            vec![fs_spec()],
            vec!["fs".into()],
            std::collections::HashMap::new(),
        )));
        let r = gw
            .call(
                "mcp.call",
                json!({"server": "fs", "tool": "read", "args": {"path": "/x"}}),
                &agent(&["mcp"]),
                GatewayCallKind::First,
            )
            .await
            .expect("granted + allowed call succeeds");
        // Result-shape check (mirrors the spine verifier posture).
        let result = r.get("result").expect("ok payload carries a `result`");
        assert_eq!(result.get("is_error").and_then(Value::as_bool), Some(false));
        let content = result.get("content").and_then(Value::as_array).expect("content array");
        assert!(!content.is_empty(), "at least one content item");
        let first = &content[0];
        assert_eq!(first.get("type").and_then(Value::as_str), Some("text"));
        assert!(
            first.get("text").and_then(Value::as_str).is_some(),
            "text item carries a non-empty `text` field",
        );
    }
}
