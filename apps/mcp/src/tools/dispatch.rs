//! Dispatch tool helpers — delegate agent lifecycle operations to the daemon.
//!
//! All dispatch operations go through the daemon's REST API, which owns the
//! orchestrator, process registry, and executor. This eliminates the duplicate
//! `DispatchService` that previously lived in runtime.

use crate::requests::{DispatchAgentRequest, SteerAgentRequest, StopAgentRequest};

/// Create a job via the daemon — the orchestrator handles worktree creation,
/// agent compilation, and process spawning.
pub async fn dispatch_agent(req: DispatchAgentRequest) -> String {
    let provider = req.provider.as_deref().unwrap_or("claude");
    let branch = format!("job/{}", req.slug);

    let body = serde_json::json!({
        "slug": req.slug,
        "agent": req.agent_profile,
        "branch": branch,
        "spec_path": ".artifacts/job-spec.md",
        "provider": provider,
        "spec_content": req.job_spec,
    });

    match crate::network_client::create_job_via_daemon(body).await {
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
