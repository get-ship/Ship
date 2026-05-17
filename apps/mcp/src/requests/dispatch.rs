use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct DispatchAgentRequest {
    /// Agent profile name (e.g. "rust-runtime").
    pub agent_profile: String,
    /// Short slug for the job (e.g. "my-feature"). Used to name the branch and worktree.
    pub slug: String,
    /// Contents of the job spec — written to `.artifacts/job-spec.md`.
    pub job_spec: String,
    /// Harness to use: "claude" (default) or "codex".
    pub harness: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct StopAgentRequest {
    /// Agent ID returned from dispatch_agent.
    pub agent_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SteerAgentRequest {
    /// Agent ID returned from dispatch_agent.
    pub agent_id: String,
    /// Message to inject into the running agent.
    pub message: String,
}
