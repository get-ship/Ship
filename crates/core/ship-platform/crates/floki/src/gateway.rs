//! The ONE door — `command::call`.
//!
//! Every effect an agent causes enters here, carrying the agent's AuthContext.
//! This is the only place the gate runs. The deno_core host op is a WIRE to this
//! door (via [`crate::dispatch::GatewayDispatch`]), not a second gate and not a
//! backdoor: it forwards the call; the gateway decides. In-process vs
//! over-socket is just transport — the invariant is "enter at the gateway with
//! real auth, let the gate decide."
//!
//! `InProcessGateway` is the in-crate test/dev seam. The real impl is
//! `ShipdGateway` (apps/shipd/src/floki_gateway.rs), which wraps shipd's ops
//! router reached in-process (RPC-as-IPC); the agent runtime is written against
//! the Gateway trait and runs unchanged over either. Every op resolves its
//! intrinsic blast through `registry::blast_of`.

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::auth::AuthContext;
use crate::gate::{self, Decision};
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
    Ask {
        op: String,
        args: Value,
        request_id: String,
    },
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
#[derive(Default)]
pub struct InProcessGateway;

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
        // Intrinsic blast — read from the descriptor set (S8), never claimed by
        // spec or request. `None` => unknown op (not in the descriptor set).
        let blast =
            registry::blast_of(op).ok_or_else(|| GatewayError::UnknownOp(op.to_string()))?;
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
            .call(
                "collections.read",
                serde_json::json!({}),
                &agent(&[]),
                GatewayCallKind::First,
            )
            .await;
        assert!(r.is_ok());
    }

    // A C2 op in an ungranted service is denied AT THE GATEWAY (the one door).
    #[tokio::test]
    async fn ungranted_elevated_op_is_denied() {
        let r = InProcessGateway
            .call(
                "runtime.dispatch",
                serde_json::json!({}),
                &agent(&["collections"]),
                GatewayCallKind::First,
            )
            .await;
        assert!(matches!(r, Err(GatewayError::Denied(_))));
    }

    // A granted C2 op that is ask-stance returns `Ask`: the gate allows the
    // capability, but a human must approve this call. The op still passes
    // through the gate; the ask is layered on top, not a bypass.
    #[tokio::test]
    async fn granting_the_service_makes_it_ask_stance() {
        let r = InProcessGateway
            .call(
                "mcp.create",
                serde_json::json!({}),
                &agent(&["mcp"]),
                GatewayCallKind::First,
            )
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
            .call(
                "mcp.create",
                serde_json::json!({}),
                &agent(&["mcp"]),
                GatewayCallKind::Approved,
            )
            .await;
        assert!(r.is_ok(), "approved re-dispatch must succeed for a granted op");
    }

    // An ask-stance op WITHOUT its service grant is denied at the gate — the
    // service-grant check runs before the ask-stance check.
    #[tokio::test]
    async fn ask_stance_without_grant_is_denied_at_the_gate() {
        let r = InProcessGateway
            .call(
                "mcp.create",
                serde_json::json!({}),
                &agent(&["collections"]),
                GatewayCallKind::First,
            )
            .await;
        match r {
            Err(GatewayError::Denied(reason)) => {
                assert!(reason.contains("mcp"), "denial names the missing service: {reason}");
            }
            other => panic!("expected a gate denial, got: {other:?}"),
        }
    }

    // An unknown op is rejected, never silently accepted.
    #[tokio::test]
    async fn unknown_op_is_rejected() {
        let r = InProcessGateway
            .call(
                "nope.nope",
                serde_json::json!({}),
                &agent(&[]),
                GatewayCallKind::First,
            )
            .await;
        assert!(matches!(r, Err(GatewayError::UnknownOp(_))));
    }

    // C3 is operator-only: an agent can never clear it, even with the service
    // granted (account is not even agent-grantable, but the gate is the backstop).
    #[tokio::test]
    async fn c3_is_denied_for_any_agent() {
        let r = InProcessGateway
            .call(
                "account.rotate_key",
                serde_json::json!({}),
                &agent(&["account"]),
                GatewayCallKind::First,
            )
            .await;
        assert!(matches!(r, Err(GatewayError::Denied(_))));
    }
}
