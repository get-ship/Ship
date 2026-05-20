//! Dispatch tool helpers — delegate agent lifecycle operations to the daemon.
//!
//! All dispatch operations go through the daemon's REST API, which owns the
//! orchestrator, process registry, and executor. This eliminates the duplicate
//! `DispatchService` that previously lived in runtime.

use crate::requests::{DispatchAgentRequest, SteerAgentRequest, StopAgentRequest};

/// Dispatch a single agent session via the daemon — the orchestrator
/// handles worktree creation, agent compilation, and process spawning.
///
/// Uses the agent-session primitive (`/api/v1/agent_sessions/dispatch`,
/// emits `agent.session.started`). A programmatic MCP dispatch is a
/// background batch: no UI tab, PTY invocation (subscription-billed),
/// one-shot (no keepalive). Operators wanting an interactive or
/// keepalive session use the `ship use` CLI surface.
pub async fn dispatch_agent(req: DispatchAgentRequest) -> String {
    let harness = req.harness.as_deref().unwrap_or("claude");
    let branch = format!("job/{}", req.slug);

    let body = serde_json::json!({
        "slug": req.slug,
        "agent": req.agent_profile,
        "branch": branch,
        "harness": harness,
        "spec": req.run_spec,
        "open_tab": false,
        "invocation": "pty",
        "keepalive": false,
    });

    match crate::network_client::dispatch_agent_session_via_daemon(body).await {
        Ok(resp) => resp,
        Err(e) => format!("Error: {e}"),
    }
}

/// List all running agents via the daemon.
pub async fn list_agents() -> String {
    match crate::network_client::list_agents_via_daemon().await {
        Ok(resp) => resp,
        Err(e) => format!("Error: {e}"),
    }
}

/// Stop a running agent by slug via the daemon.
pub async fn stop_agent(req: StopAgentRequest) -> String {
    match crate::network_client::stop_agent_via_daemon(&req.agent_id).await {
        Ok(resp) => resp,
        Err(e) => format!("Error: {e}"),
    }
}

/// Inject a message into a running agent via the daemon.
pub async fn steer_agent(req: SteerAgentRequest) -> String {
    match crate::network_client::steer_agent_via_daemon(&req.agent_id, &req.message).await {
        Ok(resp) => resp,
        Err(e) => format!("Error: {e}"),
    }
}
