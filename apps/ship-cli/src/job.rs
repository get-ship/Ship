//! `ship step-done` — the agent's pipeline-completion signal, ported off the
//! retired MCP `step_done` tool onto `PipelinesService.StepDone` (Connect).
//!
//! The job slug is implicit: it comes from `SHIP_ACTOR_ID`, injected at
//! dispatch. Agents call `ship step-done --summary "..."` and the daemon
//! resolves the active step from its projection and emits
//! `step.done_declared` through the kernel — the same rail every other
//! writer uses.

use anyhow::{Context, Result, anyhow};
use connectrpc::client::{ClientConfig, HttpClient};
use ship_ops_rpc::{PipelinesServiceClient, StepDoneRequest};
use ship_substrate::env::{Var, read_raw};

pub fn step_done(summary: String) -> Result<()> {
    let slug = match read_raw(Var::ActorId) {
        Ok(id) if !id.is_empty() => id,
        _ => {
            return Err(anyhow!(
                "SHIP_ACTOR_ID not set — `ship step-done` can only be called from a dispatched job"
            ));
        }
    };

    let (port, token) = crate::daemon::ensure_daemon()?;
    let uri: http::Uri =
        format!("http://127.0.0.1:{port}").parse().context("building daemon URI")?;
    let config =
        ClientConfig::new(uri).default_header("authorization", format!("Bearer {token}"));
    let client = PipelinesServiceClient::new(HttpClient::plaintext(), config);
    let rt = tokio::runtime::Runtime::new().context("starting tokio runtime")?;

    rt.block_on(client.step_done(StepDoneRequest {
        slug: slug.clone(),
        summary,
        ..Default::default()
    }))
    .map_err(|e| anyhow!("step-done failed: {e}"))?;

    println!("Step marked done for job '{slug}'");
    Ok(())
}
