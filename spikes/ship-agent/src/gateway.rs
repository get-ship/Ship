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

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::auth::AuthContext;
use crate::gate::{self, Blast, Decision};

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

pub struct InProcessGateway;

impl InProcessGateway {
    /// Intrinsic blast for an op — in production read from the proto method
    /// option, never from the caller.
    fn blast_of(op: &str) -> Option<Blast> {
        Some(match op {
            "collections.read" | "collections.get" | "collections.items" => Blast::C0Safe,
            "lsp.diagnostics" | "lsp.hover" => Blast::C0Safe, // read-only analysis — baseline
            "collections.put" | "collections.patch" | "collections.delete" => Blast::C1Space,
            "mcp.create" | "mcp.put" | "mcp.delete" => Blast::C2Elevated,
            "runtime.dispatch" => Blast::C2Elevated,
            "account.rotate_key" => Blast::C3Critical,
            _ => return None,
        })
    }

    /// Intrinsic human-in-the-loop stance for an op. Ops in this set are
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
        let blast = Self::blast_of(op).ok_or_else(|| GatewayError::UnknownOp(op.to_string()))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthContext;

    fn agent(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    // A baseline op succeeds for any agent.
    #[tokio::test]
    async fn baseline_op_succeeds() {
        let r = InProcessGateway
            .call("collections.read", serde_json::json!({}), &agent(&[]), GatewayCallKind::First)
            .await;
        assert!(r.is_ok());
    }

    // A C2 op in an ungranted service is denied AT THE GATEWAY (the one door).
    #[tokio::test]
    async fn ungranted_elevated_op_is_denied() {
        let r = InProcessGateway
            .call("mcp.create", serde_json::json!({}), &agent(&["collections"]), GatewayCallKind::First)
            .await;
        assert!(matches!(r, Err(GatewayError::Denied(_))));
    }

    // The same op, with the service granted, becomes `Ask`-stance: the gate
    // allows the capability, but a human must approve this call. The op
    // still passes through the gate; the ask is layered on top, not a
    // bypass. A `mcp.create` is in the ask-stance set.
    #[tokio::test]
    async fn granting_the_service_makes_it_ask_stance() {
        let r = InProcessGateway
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
        let r = InProcessGateway
            .call("mcp.create", serde_json::json!({}), &agent(&["mcp"]), GatewayCallKind::Approved)
            .await;
        assert!(r.is_ok(), "approved re-dispatch must succeed for a granted op");
    }

    // An unknown op is rejected, never silently accepted.
    #[tokio::test]
    async fn unknown_op_is_rejected() {
        let r = InProcessGateway
            .call("nope.nope", serde_json::json!({}), &agent(&[]), GatewayCallKind::First)
            .await;
        assert!(matches!(r, Err(GatewayError::UnknownOp(_))));
    }
}
