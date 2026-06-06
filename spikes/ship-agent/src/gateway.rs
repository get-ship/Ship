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
}

/// The single door. Async because real dispatch is async (and ops fan out on
/// Tokio under `parallel`).
#[async_trait]
pub trait Gateway: Send + Sync {
    async fn call(&self, op: &str, args: Value, auth: &AuthContext) -> Result<Value, GatewayError>;
}

/// Stub gateway: mirrors the proto op registry just enough to demonstrate the
/// gate. In production (op, service, blast) come from the proto descriptor set;
/// here a tiny table stands in. It runs the REAL gate decision, then echoes.
pub struct InProcessGateway;

impl InProcessGateway {
    /// Intrinsic blast for an op — in production read from the proto method
    /// option, never from the caller.
    fn blast_of(op: &str) -> Option<Blast> {
        Some(match op {
            "collections.read" | "collections.get" | "collections.items" => Blast::C0Safe,
            "collections.put" | "collections.patch" | "collections.delete" => Blast::C1Space,
            "mcp.create" | "mcp.put" | "mcp.delete" => Blast::C2Elevated,
            "runtime.dispatch" => Blast::C2Elevated,
            "account.rotate_key" => Blast::C3Critical,
            _ => return None,
        })
    }
}

#[async_trait]
impl Gateway for InProcessGateway {
    async fn call(&self, op: &str, args: Value, auth: &AuthContext) -> Result<Value, GatewayError> {
        let blast = Self::blast_of(op).ok_or_else(|| GatewayError::UnknownOp(op.to_string()))?;
        let service = gate::service_of(op);

        // THE GATE — the only enforcement point.
        match gate::decide(blast, service, auth) {
            Decision::Deny(reason) => return Err(GatewayError::Denied(reason)),
            Decision::Allow => {}
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
            .call("collections.read", serde_json::json!({}), &agent(&[]))
            .await;
        assert!(r.is_ok());
    }

    // A C2 op in an ungranted service is denied AT THE GATEWAY (the one door).
    #[tokio::test]
    async fn ungranted_elevated_op_is_denied() {
        let r = InProcessGateway
            .call("mcp.create", serde_json::json!({}), &agent(&["collections"]))
            .await;
        assert!(matches!(r, Err(GatewayError::Denied(_))));
    }

    // The same op is allowed once its service is granted.
    #[tokio::test]
    async fn granting_the_service_allows_the_op() {
        let r = InProcessGateway
            .call("mcp.create", serde_json::json!({}), &agent(&["mcp"]))
            .await;
        assert!(r.is_ok());
    }

    // An unknown op is rejected, never silently accepted.
    #[tokio::test]
    async fn unknown_op_is_rejected() {
        let r = InProcessGateway
            .call("nope.nope", serde_json::json!({}), &agent(&[]))
            .await;
        assert!(matches!(r, Err(GatewayError::UnknownOp(_))));
    }
}
