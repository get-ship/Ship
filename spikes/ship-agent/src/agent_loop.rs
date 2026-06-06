//! S1.6 — the deliberately-boring loop.
//!
//! rig drives the model turn (commodity; we do not reinvent it). Our only job is
//! to make the model's tool calls land on the ONE door, gated and verified. The
//! differentiator is tool quality, not loop cleverness — so this file is small.
//!
//! `ShipOpTool` is the formal-tool path: the model emits a `ship_op` tool call,
//! it routes through `gateway.call` (where the gate runs) wrapped by the spine
//! (nothing accepted blindly). A gate denial comes back as a tool RESULT the
//! model can read and adapt to — not an exception. Real failures error.
//!
//! Code-mode (`run_script` as a loop tool) is deferred: deno_core's JsRuntime is
//! `!Send`, which collides with rig's `Send` tool-future bound, so it needs a
//! dedicated current-thread executor + a Send result channel. That's S3.

use std::sync::Arc;

use rig::agent::AgentBuilder;
use rig::completion::{CompletionModel, Prompt, ToolDefinition};
use rig::tool::Tool;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::AuthContext;
use crate::gateway::{Gateway, GatewayError};
use crate::spec::AgentSpec;
use crate::spine::SpineError;

/// Structured result of a headless run — what a pipeline step or an orchestrator
/// consumes.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub text: String,
    pub json: Option<Value>,
}

/// The formal op tool: one tool, every granted op. Routes through the one door.
pub struct ShipOpTool {
    pub gateway: Arc<dyn Gateway>,
    pub auth: AuthContext,
}

#[derive(Deserialize)]
pub struct OpArgs {
    op: String,
    #[serde(default)]
    args: Value,
}

impl Tool for ShipOpTool {
    const NAME: &'static str = "ship_op";
    type Error = SpineError;
    type Args = OpArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Call a gated Ship op (e.g. collections.read). The daemon gate decides; \
                          a denial returns as { denied } in the result — adjust, don't pretend."
                .into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "op":   { "type": "string", "description": "dotted op name, e.g. collections.read" },
                    "args": { "type": "object" }
                },
                "required": ["op"]
            }),
        }
    }

    async fn call(&self, a: OpArgs) -> Result<String, SpineError> {
        match self.gateway.call(&a.op, a.args, &self.auth).await {
            // VERIFY: nothing accepted blindly — the result must confirm itself.
            Ok(v) => {
                if v.get("ok").and_then(Value::as_bool) == Some(true) {
                    Ok(v.to_string())
                } else {
                    Err(SpineError::Unverified(format!(
                        "op `{}` did not confirm success",
                        a.op
                    )))
                }
            }
            // A gate denial is a RESULT the model reads, not an exception.
            Err(GatewayError::Denied(msg)) => Ok(json!({ "denied": msg }).to_string()),
            // A real failure (unknown op, dispatch error) surfaces as an error.
            Err(e) => Err(SpineError::Execute(e.to_string())),
        }
    }
}

/// Run an agent to completion: rig drives the turn loop; our `ShipOpTool` makes
/// every effect pass the gate + spine. Headless; returns a structured result.
pub async fn run_step<M>(
    model: M,
    spec: &AgentSpec,
    gateway: Arc<dyn Gateway>,
    auth: AuthContext,
    input: &str,
) -> anyhow::Result<AgentOutput>
where
    M: CompletionModel + 'static,
{
    let preamble = if spec.disposition.rules.is_empty() {
        "You are a Ship agent. Use the ship_op tool to call gated ops. \
         If an op returns { denied }, adjust — do not pretend it succeeded."
    } else {
        &spec.disposition.rules
    };

    let agent = AgentBuilder::new(model)
        .preamble(preamble)
        .tool(ShipOpTool { gateway, auth })
        .build();

    let text = agent
        .prompt(input)
        .max_turns(spec.disposition.max_turns.unwrap_or(10))
        .await?;

    Ok(AgentOutput { text, json: None })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::InProcessGateway;
    use async_trait::async_trait;
    use rig::test_utils::{MockCompletionModel, MockTurn};
    use std::sync::Mutex;

    fn auth(grants: &[&str]) -> AuthContext {
        AuthContext::in_process("t", grants.iter().map(|s| s.to_string()).collect())
    }

    // ── tool-level behavior (deterministic, no model) ──────────────────────
    #[tokio::test]
    async fn tool_returns_result_for_granted_op() {
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway), auth: auth(&["collections"]) };
        let out = tool
            .call(OpArgs { op: "collections.read".into(), args: json!({}) })
            .await
            .expect("granted op should succeed");
        assert!(!out.contains("denied"), "granted op must not be denied");
    }

    #[tokio::test]
    async fn tool_surfaces_denial_as_result_not_error() {
        let tool = ShipOpTool { gateway: Arc::new(InProcessGateway), auth: auth(&["collections"]) };
        // mcp is ungranted; the gate denies. The tool returns it as DATA.
        let out = tool
            .call(OpArgs { op: "mcp.create".into(), args: json!({}) })
            .await
            .expect("denial is a result, not a tool error");
        assert!(out.contains("denied"), "ungranted op must come back denied");
    }

    // ── loop behavior: a model tool call routes through the gateway ─────────
    /// Records the ops the loop actually dispatched, to prove the model's tool
    /// call reached the one door.
    struct RecordingGateway {
        inner: InProcessGateway,
        seen: Arc<Mutex<Vec<String>>>,
    }
    #[async_trait]
    impl Gateway for RecordingGateway {
        async fn call(&self, op: &str, args: Value, a: &AuthContext) -> Result<Value, GatewayError> {
            self.seen.lock().unwrap().push(op.to_string());
            self.inner.call(op, args, a).await
        }
    }

    #[tokio::test]
    async fn model_tool_call_routes_through_the_gate() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let gw = Arc::new(RecordingGateway { inner: InProcessGateway, seen: seen.clone() });

        // Scripted model: call ship_op once, then answer.
        let model = MockCompletionModel::new([
            MockTurn::tool_call("1", "ship_op", json!({ "op": "collections.read", "args": {} })),
            MockTurn::text("done"),
        ]);
        let spec = AgentSpec::demo("t", "mock", &["collections"]);

        let out = run_step(model, &spec, gw, auth(&["collections"]), "read the leads")
            .await
            .expect("loop should complete");

        // Behavioral: the loop finished AND the op the model requested actually
        // crossed the gateway — the model→tool→one-door path is wired.
        assert!(out.text.contains("done"));
        assert!(
            seen.lock().unwrap().iter().any(|op| op == "collections.read"),
            "the model's tool call must route through the gateway"
        );
    }
}
