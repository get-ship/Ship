//! `GatewayDispatch` — adapt Floki's rich op door to the L0 [`Dispatch`] seam.
//!
//! ship-js and durable-execution both speak ONE contract: [`Dispatch::call`]
//! `(op, args) -> Result<Value, DispatchError>`, carrying no auth, no gate, no
//! transport (see `ship_js::dispatch`). Floki's own door, [`Gateway`], is richer
//! — it takes the caller's [`AuthContext`] and a [`GatewayCallKind`], and it
//! distinguishes `Denied` / `Ask` / `UnknownOp` / `Dispatch` outcomes so the
//! interactive tool layer can run the approval handshake.
//!
//! This adapter is the bridge: it captures `(gateway, auth)` so the credential
//! is held *inside* the impl and never crosses the seam, then flattens the rich
//! `GatewayError` to the seam's two-way classification:
//!
//! - a **denial** is terminal *data*, not a retryable failure — a JS script
//!   reads it back (ship-js maps a host `Err` to `{ error }`) and adapts; the
//!   durable engine never re-attempts it.
//! - an **unknown op** / **bad args** is terminal (re-running won't help).
//! - a **dispatch blip** is transient (the engine may retry).
//! - an **`Ask`** can't be resolved inside a non-interactive script step, so it
//!   surfaces as a terminal denial here; the interactive `ship_op` tool path
//!   (which talks to [`Gateway`] directly, not through this adapter) is where the
//!   human approval handshake actually runs.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use ship_js::{Dispatch, DispatchError};

use crate::auth::AuthContext;
use crate::gateway::{Gateway, GatewayCallKind, GatewayError};

/// A [`Dispatch`] over a [`Gateway`] + a captured [`AuthContext`]. This is the
/// "host-dispatch impl" the L0 engines are written against: ship-js hands it to
/// each code-mode run; durable-execution wraps it to journal each op.
pub struct GatewayDispatch {
    pub gateway: Arc<dyn Gateway>,
    pub auth: AuthContext,
}

impl GatewayDispatch {
    pub fn new(gateway: Arc<dyn Gateway>, auth: AuthContext) -> Self {
        Self { gateway, auth }
    }
}

#[async_trait]
impl Dispatch for GatewayDispatch {
    async fn call(&self, op: &str, args: Value) -> Result<Value, DispatchError> {
        // First-kind call: the gate runs. `Approved` re-dispatch only happens on
        // the interactive `ship_op` path, which uses the Gateway directly.
        match self
            .gateway
            .call(op, args, &self.auth, GatewayCallKind::First)
            .await
        {
            Ok(v) => Ok(v),
            // A denial is data the program reads, never retried (the contract's
            // `Terminal`). ship-js flattens this to `{ error }` for the isolate.
            Err(GatewayError::Denied(msg)) => Err(DispatchError::Terminal(format!("DENIED: {msg}"))),
            Err(GatewayError::Ask { op, request_id, .. }) => Err(DispatchError::Terminal(format!(
                "DENIED: op `{op}` requires human approval (id={request_id}); not available in a script step"
            ))),
            Err(GatewayError::UnknownOp(op)) => {
                Err(DispatchError::Terminal(format!("unknown op `{op}`")))
            }
            // A dispatch blip might clear on retry — the engine's job, not ours.
            Err(GatewayError::Dispatch(msg)) => Err(DispatchError::Transient(msg)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::InProcessGateway;
    use serde_json::json;

    fn auth(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    fn dispatch(grants: &[&str]) -> GatewayDispatch {
        GatewayDispatch::new(Arc::new(InProcessGateway::default()), auth(grants))
    }

    // A granted op returns the gateway's Ok value, unchanged, through the seam.
    #[tokio::test]
    async fn granted_op_passes_through_as_ok() {
        let d = dispatch(&["collections"]);
        let v = d
            .call("collections.read", json!({ "collection": "leads" }))
            .await
            .expect("granted baseline op should succeed through the seam");
        assert_eq!(v.get("ok").and_then(Value::as_bool), Some(true));
        assert_eq!(v.get("op").and_then(Value::as_str), Some("collections.read"));
    }

    // A gate denial becomes a TERMINAL DispatchError (never retried), carrying
    // the denial reason — ship-js will surface this to the isolate as `{ error }`.
    #[tokio::test]
    async fn gate_denial_is_terminal_not_transient() {
        let d = dispatch(&["collections"]); // no `runtime` grant
        let err = d
            .call("runtime.dispatch", json!({ "agent": "child" }))
            .await
            .expect_err("ungranted C2 op must be denied at the gate");
        match err {
            DispatchError::Terminal(msg) => {
                assert!(msg.to_lowercase().contains("denied"), "reason carried: {msg}");
            }
            DispatchError::Transient(msg) => panic!("a denial must be terminal, not transient: {msg}"),
        }
    }

    // An unknown op is terminal — re-running can't make it known.
    #[tokio::test]
    async fn unknown_op_is_terminal() {
        let d = dispatch(&[]);
        let err = d.call("nope.nope", json!({})).await.expect_err("unknown op");
        assert!(matches!(err, DispatchError::Terminal(_)));
    }

    // The captured credential is what the gate sees: the SAME op is denied for an
    // ungranted credential and allowed for a granted one — proving auth lives
    // inside the impl, not on the seam.
    #[tokio::test]
    async fn captured_credential_governs_the_decision() {
        // mcp.create is C2 + ask-stance; without the grant it's a flat gate
        // denial, with the grant it becomes an Ask — both terminal here, but the
        // reasons differ, proving the captured auth is consulted.
        let denied = dispatch(&["collections"])
            .call("mcp.create", json!({}))
            .await
            .expect_err("no mcp grant → gate denial");
        let asked = dispatch(&["mcp"])
            .call("mcp.create", json!({}))
            .await
            .expect_err("mcp grant → ask-stance, terminal in a script step");
        let (DispatchError::Terminal(d) | DispatchError::Transient(d)) = denied;
        let (DispatchError::Terminal(a) | DispatchError::Transient(a)) = asked;
        assert!(d.contains("mcp"), "ungranted reason names the missing service: {d}");
        assert!(a.to_lowercase().contains("approval"), "granted reason is the ask: {a}");
    }
}
