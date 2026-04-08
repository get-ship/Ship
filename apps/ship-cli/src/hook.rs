use anyhow::{Result, anyhow};
use cli_framework::hooks::{
    handle_after_tool, handle_before_tool, handle_session_end, handle_session_start,
};
use std::io::Read;

fn actor_id() -> Result<String> {
    std::env::var("SHIP_ACTOR_ID")
        .map_err(|_| anyhow!("SHIP_ACTOR_ID env var is not set"))
}

fn workspace_id() -> Result<String> {
    std::env::var("SHIP_WORKSPACE_ID")
        .map_err(|_| anyhow!("SHIP_WORKSPACE_ID env var is not set"))
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn run_session_start() -> Result<()> {
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    let event = handle_session_start(&actor, &workspace)?;
    println!("{}", serde_json::to_string(&event)?);
    Ok(())
}

pub fn run_before_tool() -> Result<()> {
    let stdin = read_stdin()?;
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    if let Some(event) = handle_before_tool(&stdin, &actor, &workspace)? {
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}

pub fn run_after_tool() -> Result<()> {
    let stdin = read_stdin()?;
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    if let Some(event) = handle_after_tool(&stdin, &actor, &workspace)? {
        println!("{}", serde_json::to_string(&event)?);
    }
    Ok(())
}

pub fn run_session_end() -> Result<()> {
    let actor = actor_id()?;
    let workspace = workspace_id()?;
    let event = handle_session_end(&actor, &workspace)?;
    println!("{}", serde_json::to_string(&event)?);
    // Best-effort: broadcast session.ended via ship CLI (MCP bridge path).
    broadcast_session_ended(&actor, &workspace);
    // Fallback: if running inside a job (SHIP_MESH_ID set), signal completion
    // to the daemon so the pipeline can advance even if the agent didn't
    // explicitly call update_job.
    if let Ok(slug) = std::env::var("SHIP_MESH_ID") {
        signal_job_completed(&slug);
    }
    Ok(())
}

/// Signal job completion to the daemon via REST API.
/// Non-fatal — best-effort fallback for pipeline advancement.
fn signal_job_completed(slug: &str) {
    let home = dirs::home_dir().unwrap_or_default().join(".ship");
    let port = match std::fs::read_to_string(home.join("daemon.port")) {
        Ok(p) => p.trim().to_string(),
        Err(_) => return,
    };
    let token = match std::fs::read_to_string(home.join("daemon.token")) {
        Ok(t) => t.trim().to_string(),
        Err(_) => return,
    };
    let _ = std::process::Command::new("curl")
        .args([
            "-sf", "-X", "POST",
            &format!("http://127.0.0.1:{port}/api/runtime/jobs/end"),
            "-H", "Content-Type: application/json",
            "-H", &format!("Authorization: Bearer {token}"),
            "-d", &format!("{{\"slug\":\"{slug}\"}}"),
        ])
        .status();
}

/// Broadcast agent.session.ended via `ship mesh broadcast` CLI.
/// Non-fatal — if ship is not available or mesh is down this is a no-op.
fn broadcast_session_ended(actor_id: &str, workspace_id: &str) {
    let payload = serde_json::json!({
        "event_type": "agent.session.ended",
        "from_agent_id": actor_id,
        "payload": { "workspace_id": workspace_id }
    });
    let _ = std::process::Command::new("ship")
        .args(["mesh", "broadcast", &payload.to_string()])
        .status();
}
