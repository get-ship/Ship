use anyhow::{Result, anyhow};
use ship_platform::hooks::{
    handle_after_tool, handle_before_tool, handle_session_end, handle_session_start, tool_gate,
};
use std::io::Read;

fn agent_id() -> Option<String> {
    ship_substrate::env::read_raw(ship_substrate::env::Var::AgentId).ok().filter(|s| !s.is_empty())
}

fn workspace_id() -> Option<String> {
    ship_substrate::env::read_raw(ship_substrate::env::Var::WorkspaceId).ok().filter(|s| !s.is_empty())
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Detect the harness from environment variables.
fn detect_harness() -> &'static str {
    if std::env::var("CODEX_SESSION_ID").is_ok() {
        return "codex";
    }
    if std::env::var("CURSOR_SESSION_ID").is_ok()
        || std::env::var("CURSOR_HOOK_EVENT").is_ok()
    {
        return "cursor";
    }
    // Default: Claude (or unknown — same stdin format).
    "claude"
}

/// Normalize harness-specific stdin JSON into the daemon's ShipHookEvent.
fn normalize_stdin(stdin: &str, hook_type: &str) -> serde_json::Value {
    let parsed: serde_json::Value =
        serde_json::from_str(stdin).unwrap_or(serde_json::json!({}));

    let harness = detect_harness();

    // All harnesses currently use a similar JSON shape with tool_name / tool_input.
    // Normalize field names for cross-harness compatibility.
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

    let agent = ship_substrate::env::read_raw(ship_substrate::env::Var::AgentId).ok();
    let workspace = ship_substrate::env::read_raw(ship_substrate::env::Var::WorkspaceId).ok();
    let session = ship_substrate::env::read_raw(ship_substrate::env::Var::ActorId).ok();

    let mut event = serde_json::json!({
        "harness": harness,
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
    if let Some(v) = agent {
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
/// Hooks authenticate with the same daemon token every CLI call uses.
/// Discovery walks ProjectRoot::discover(cwd) → per-worktree daemon →
/// singleton fallback (devx-04).
///
/// For hooks, we don't auto-start — hooks must be fast and non-blocking.
/// Use `crate::daemon::ensure_daemon()` for commands that can afford the wait.
fn daemon_info() -> Option<(u16, String)> {
    let cwd = std::env::current_dir().ok()?;
    let handle = ship_substrate::home::discover_running_daemon(&cwd)?;
    Some((handle.port, handle.token))
}

/// POST a normalized hook event to the daemon. Returns the response body.
fn post_to_daemon(endpoint: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
    let (port, token) = daemon_info()
        .ok_or_else(|| anyhow!("daemon not running (no port/token file)"))?;
    let url = format!("http://127.0.0.1:{port}/hooks/{endpoint}");
    let client = crate::http::daemon_client();
    let resp = client
        .post(&url)
        .bearer_auth(&token)
        .json(body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .map_err(|e| anyhow!("daemon request failed: {e}"))?;
    crate::http::check_skew(resp.headers());
    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!("daemon returned {status}: {body}"));
    }
    serde_json::from_str(&body).map_err(|e| anyhow!("invalid daemon response: {e}"))
}

/// Try to POST to daemon; fall back to local event emission on failure.
///
/// Logs the daemon error to stderr so harness hook logs capture the reason.
/// Distinguishes between "daemon unreachable" (expected fallback) and
/// "daemon rejected request" (auth or other issue that needs attention).
fn try_daemon_or_local(
    endpoint: &str,
    body: &serde_json::Value,
    local_fallback: impl FnOnce() -> Result<()>,
) -> Result<()> {
    match post_to_daemon(endpoint, body) {
        Ok(resp) => {
            // Output the daemon response for harnesses that read stdout.
            println!("{}", serde_json::to_string(&resp)?);
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            // "connection refused" or missing port/token file → expected fallback.
            // Anything else (401, 403, 500) means the daemon is up but rejected us.
            let is_unreachable = msg.contains("connection refused")
                || msg.contains("no port/token file")
                || msg.contains("not running");
            if is_unreachable {
                eprintln!("ship hook: daemon unreachable ({msg}) — using local fallback");
            } else {
                eprintln!("ship hook: daemon rejected /{endpoint} ({msg}) — using local fallback");
            }
            local_fallback()
        }
    }
}

pub fn run_session_start() -> Result<()> {
    let body = normalize_stdin("", "session-start");
    try_daemon_or_local("session-start", &body, || {
        if let (Some(actor), Some(workspace)) = (agent_id(), workspace_id()) {
            let event = handle_session_start(&actor, &workspace)?;
            println!("{}", serde_json::to_string(&event)?);
        }
        Ok(())
    })
}

pub fn run_before_tool() -> Result<()> {
    let stdin = read_stdin()?;

    // Layer 2 — Ship-integrity gate (see SECURITY.md). Runs before the
    // daemon roundtrip; on deny, emit Claude Code's permission-decision
    // JSON and return without logging a (nonexistent) tool call.
    let ctx = tool_gate::Context {
        home_dir: home::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/")),
        daemon_port: std::env::current_dir()
            .ok()
            .and_then(|c| ship_substrate::home::discover_running_daemon(&c))
            .map(|h| h.port),
    };
    if let tool_gate::Decision::Deny(reason) = tool_gate::check(&stdin, &ctx) {
        // Best-effort log the denial to the daemon so analytics +
        // dashboards can surface blocked tool attempts. Failures here
        // must not affect the deny semantics — print the deny shape
        // either way so the harness still blocks.
        let mut body = normalize_stdin(&stdin, "permission-denied");
        body["reason"] = serde_json::Value::String(reason.clone());
        body["denied_by"] = serde_json::Value::String("ship-gate".to_string());
        if let Err(e) = post_to_daemon("permission-denied", &body) {
            eprintln!("ship hook: permission-denied log failed ({e})");
        }
        println!(
            "{}",
            serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            })
        );
        return Ok(());
    }

    let body = normalize_stdin(&stdin, "before-tool");
    try_daemon_or_local("before-tool", &body, || {
        if let (Some(actor), Some(workspace)) = (agent_id(), workspace_id())
            && let Some(event) = handle_before_tool(&stdin, &actor, &workspace)? {
                println!("{}", serde_json::to_string(&event)?);
            }
        Ok(())
    })
}

pub fn run_after_tool() -> Result<()> {
    let stdin = read_stdin()?;
    let body = normalize_stdin(&stdin, "after-tool");
    try_daemon_or_local("after-tool", &body, || {
        if let (Some(actor), Some(workspace)) = (agent_id(), workspace_id())
            && let Some(event) = handle_after_tool(&stdin, &actor, &workspace)? {
                println!("{}", serde_json::to_string(&event)?);
            }
        Ok(())
    })
}

pub fn run_session_end() -> Result<()> {
    let body = normalize_stdin("", "session-end");
    try_daemon_or_local("session-end", &body, || {
        if let (Some(actor), Some(workspace)) = (agent_id(), workspace_id()) {
            let event = handle_session_end(&actor, &workspace)?;
            println!("{}", serde_json::to_string(&event)?);
            broadcast_session_ended(&actor, &workspace);
        }
        if let Ok(slug) = ship_substrate::env::read_raw(ship_substrate::env::Var::ActorId) {
            signal_job_completed(&slug);
        }
        Ok(())
    })
}

/// Signal job completion to the daemon via `PipelinesService.EndTurn` (Connect).
/// Non-fatal — best-effort fallback for pipeline advancement.
fn signal_job_completed(slug: &str) {
    use connectrpc::client::{ClientConfig, HttpClient};
    use ship_ops_rpc::{EndTurnReq, PipelinesServiceClient};
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(_) => return,
    };
    let handle = match ship_substrate::home::discover_running_daemon(&cwd) {
        Some(h) => h,
        None => return,
    };
    let uri: http::Uri = match format!("http://127.0.0.1:{}", handle.port).parse() {
        Ok(u) => u,
        Err(_) => return,
    };
    let config = ClientConfig::new(uri)
        .default_header("authorization", format!("Bearer {}", handle.token));
    let client = PipelinesServiceClient::new(HttpClient::plaintext(), config);
    if let Ok(rt) = tokio::runtime::Runtime::new() {
        let _ = rt.block_on(
            client.end_turn(EndTurnReq { slug: slug.to_string(), ..Default::default() }),
        );
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_stdin_carries_env_identity() {
        // SAFETY: single-threaded test, no other threads read env here.
        unsafe {
            ship_substrate::env::set(ship_substrate::env::Var::AgentId, "agent.rust-lane");
            ship_substrate::env::set(ship_substrate::env::Var::ActorId, "job-abc");
            ship_substrate::env::set(ship_substrate::env::Var::WorkspaceId, "ws-1");
        }
        let stdin = r#"{"tool_name":"Bash","tool_input":{"command":"ls"}}"#;
        let event = normalize_stdin(stdin, "before-tool");
        assert_eq!(event["tool_name"], "Bash");
        assert_eq!(event["actor_id"], "agent.rust-lane");
        assert_eq!(event["session_id"], "job-abc");
        assert_eq!(event["workspace"], "ws-1");
        assert_eq!(event["hook_type"], "before-tool");
        unsafe {
            ship_substrate::env::remove(ship_substrate::env::Var::AgentId);
            ship_substrate::env::remove(ship_substrate::env::Var::ActorId);
            ship_substrate::env::remove(ship_substrate::env::Var::WorkspaceId);
        }
    }

    #[test]
    fn normalize_stdin_accepts_harness_variants() {
        // cursor-style camelCase keys
        let stdin = r#"{"toolName":"Edit","toolInput":{"path":"f"}}"#;
        let event = normalize_stdin(stdin, "before-tool");
        assert_eq!(event["tool_name"], "Edit");
        assert_eq!(event["tool_input"]["path"], "f");
    }
}
