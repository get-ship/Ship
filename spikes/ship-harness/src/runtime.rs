//! AgentRuntime — the agent as a CALLABLE PRIMITIVE.
//!
//! The Ship agent is the default runner for LLM pipeline steps, and the user
//! wants it orchestratable Claude-Workflow style. Both of those reduce to one
//! entrypoint: run an agent headless to completion and return a structured
//! result. `run_step` is that primitive.
//!
//! Two callers, one entrypoint:
//!   - a pipeline step (declarative .ship/pipelines) invokes `run_step` directly;
//!   - the Ship JS SDK — running in the SAME rquickjs engine that powers
//!     `run_script` — binds `agent()` to `run_step`, so a JS workflow can
//!     `await agent(spec, input)`, `parallel([...])`, `pipeline(...)` and fan out
//!     agents deterministically. One execution substrate, host-safe, no sidecar.
//!
//! `run_turn` (session.rs) is the INTERACTIVE/streamed variant of this same loop;
//! `run_step` is the headless/structured one.

use rig::agent::Agent;
use rig::completion::{CompletionModel, Prompt};
use serde_json::Value;

use crate::spec::{AgentSpec, OutputContract};

/// Structured result of a headless run — what a pipeline step or `agent()`
/// orchestration call consumes.
#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub text: String,
    /// Present when the spec's OutputContract is `Json` and parsing succeeded.
    pub json: Option<Value>,
}

/// Run an agent to completion, honoring the spec's turn budget and output
/// contract. No streaming, no UI — returns a value an orchestrator can compose.
///
/// This is the binding target for the JS SDK: `agent(spec, input) -> AgentOutput`
/// becomes a host function the rquickjs orchestration script awaits. Nesting
/// (an agent whose workflow calls `agent()` again) must cap depth and propagate
/// — never elevate — the parent's identity/ceiling, exactly like Claude
/// Workflow's one-level nesting + shared budget.
pub async fn run_step<M>(
    agent: &Agent<M>,
    spec: &AgentSpec,
    input: &str,
) -> anyhow::Result<AgentOutput>
where
    M: CompletionModel + 'static,
{
    let text = agent.prompt(input).max_turns(spec.max_turns).await?;
    let json = match &spec.output {
        OutputContract::Json { .. } => serde_json::from_str(&text).ok(),
        OutputContract::Text => None,
    };
    Ok(AgentOutput { text, json })
}
