use anyhow::{Result, anyhow};
use cli_framework::hooks::{
    handle_after_tool, handle_before_tool, handle_session_end, handle_session_start,
};
use std::io::Read;

fn actor_id() -> Option<String> {
    std::env::var("SHIP_ACTOR_ID").ok().filter(|s| !s.is_empty())
}

fn workspace_id() -> Option<String> {
    std::env::var("SHIP_WORKSPACE_ID").ok().filter(|s| !s.is_empty())
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Detect the provider from environment variables.
fn detect_provider() -> &'static str {
    if std::env::var("CODEX_SESSION_ID").is_ok() {
        return "codex";
    }
    if std::env::var("GEMINI_SESSION_ID").is_ok()
        || std::env::var("GEMINI_CLI").is_ok()
    {
        return "gemini";
    }
    if std::env::var("CURSOR_SESSION_ID").is_ok()
        || std::env::var("CURSOR_HOOK_EVENT").is_ok()
    {
        return "cursor";
    }
    // Default: Claude (or unknown — same stdin format).
    "claude"
}

/// Normalize provider-specific stdin JSON into the daemon's ShipHookEvent.
fn normalize_stdin(stdin: &str, hook_type: &str) -> serde_json::Value {
    let parsed: serde_json::Value =
        serde_json::from_str(stdin).unwrap_or(serde_json::json!({}));

    let provider = detect_provider();

    // All providers currently use a similar JSON shape with tool_name / tool_input.
    // Normalize field names for cross-provider compatibility.
    let tool_name = parsed
        .get("tool_name")
        .or_else(|| parsed.get("toolName"))
        .or_else(|| parsed.get("name"))
        .cloned();
    let tool_input = parsed
        .get("tool_input")
        .or_else(|| parsed.get("toolInput"))
        .or_else(|| parsed.get("input"))
        .cloned();
    let tool_response = parsed
        .get("tool_response")
        .or_else(|| parsed.get("toolResponse"))
        .or_else(|| parsed.get("output"))
        .cloned();

    let actor = std::env::var("SHIP_ACTOR_ID").ok();
    let workspace = std::env::var("SHIP_WORKSPACE_ID").ok();
    let session = std::env::var("SHIP_SESSION_ID").ok();

    let mut event = serde_json::json!({
        "provider": provider,
        "hook_type": hook_type,
    });
    if let Some(v) = tool_name {
        event["tool_name"] = v;
    }
    if let Some(v) = tool_input {
        event["tool_input"] = v;
    }
    if let Some(v) = tool_response {
        event["tool_response"] = v;
    }
    if let Some(v) = actor {
        event["actor_id"] = serde_json::json!(v);
    }
    if let Some(v) = workspace {
        event["workspace"] = serde_json::json!(v);
    }
    if let Some(v) = session {
        event["session_id"] = serde_json::json!(v);
    }
    event
}

/// Read daemon connection info. Returns `(port, token)` if available.
///
/// For hooks, we don't auto-start — hooks must be fast and non-blocking.
/// Use `crate::daemon::ensure_daemon()` for commands that can afford the wait.
fn daemon_info() -> Option<(u16, String)> {
    let home = dirs::home_dir()?.join(".ship");
    let port_str = std::fs::read_to_string(home.join("daemon.port")).ok()?;
    let port = port_str.trim().parse::<u16>().ok()?;
    let token = std::fs::read_to_string(home.join("daemon.token")).ok()?;
    Some((port, token.trim().to_string()))
}

/// POST a normalized hook event to the daemon. Returns the response body.
fn post_to_daemon(endpoint: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
    let (port, token) = daemon_info()
        .ok_or_else(|| anyhow!("daemon not running (no port/token file)"))?;
    let url = format!("http://127.0.0.1:{port}/api/hooks/{endpoint}");
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .bearer_auth(&token)
        .json(body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow!("daemon request failed: {e}"))?;
    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("daemon returned {status}: {body}"));
    }
    serde_json::from_str(&body).map_err(|e| anyhow!("invalid daemon response: {e}"))
}

/// Try to POST to daemon; fall back to local event emission on failure.
fn try_daemon_or_local(
    endpoint: &str,
    body: &serde_json::Value,
    local_fallback: impl FnOnce() -> Result<()>,
) -> Result<()> {
    match post_to_daemon(endpoint, body) {
        Ok(resp) => {
            // Output the daemon response for providers that read stdout.
            println!("{}", serde_json::to_string(&resp)?);
            Ok(())
        }
        Err(_) => {
            // Daemon unavailable — fall back to local event emission.
            local_fallback()
        }
    }
}

pub fn run_session_start() -> Result<()> {
    let body = normalize_stdin("", "session-start");
    try_daemon_or_local("session-start", &body, || {
        if let (Some(actor), Some(workspace)) = (actor_id(), workspace_id()) {
            let event = handle_session_start(&actor, &workspace)?;
            println!("{}", serde_json::to_string(&event)?);
        }
        Ok(())
    })
}

pub fn run_before_tool() -> Result<()> {
    let stdin = read_stdin()?;
    let body = normalize_stdin(&stdin, "before-tool");
    try_daemon_or_local("before-tool", &body, || {
        if let (Some(actor), Some(workspace)) = (actor_id(), workspace_id()) {
            if let Some(event) = handle_before_tool(&stdin, &actor, &workspace)? {
                println!("{}", serde_json::to_string(&event)?);
            }
        }
        Ok(())
    })
}

pub fn run_after_tool() -> Result<()> {
    let stdin = read_stdin()?;
    let body = normalize_stdin(&stdin, "after-tool");
    try_daemon_or_local("after-tool", &body, || {
        if let (Some(actor), Some(workspace)) = (actor_id(), workspace_id()) {
            if let Some(event) = handle_after_tool(&stdin, &actor, &workspace)? {
                println!("{}", serde_json::to_string(&event)?);
            }
        }
        Ok(())
    })
}

pub fn run_session_end() -> Result<()> {
    let body = normalize_stdin("", "session-end");
    try_daemon_or_local("session-end", &body, || {
        if let (Some(actor), Some(workspace)) = (actor_id(), workspace_id()) {
            let event = handle_session_end(&actor, &workspace)?;
            println!("{}", serde_json::to_string(&event)?);
            broadcast_session_ended(&actor, &workspace);
        }
        if let Ok(slug) = std::env::var("SHIP_MESH_ID") {
            signal_job_completed(&slug);
        }
        Ok(())
    })
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
    let client = reqwest::blocking::Client::new();
    let _ = client
        .post(format!("http://127.0.0.1:{port}/api/runtime/jobs/end"))
        .bearer_auth(&token)
        .json(&serde_json::json!({"slug": slug}))
        .timeout(std::time::Duration::from_secs(5))
        .send();
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
