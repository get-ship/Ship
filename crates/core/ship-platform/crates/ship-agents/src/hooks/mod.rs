//! Hook ingress — CLI/HTTP layer that emits `hook.*` and `skill.*` events
//! for harness hook input. The workbench `handle_session_start` /
//! `handle_session_end` helpers (which emitted `session.started` /
//! `session.ended` workbench events) were retired with the workbench
//! family (job `workbench-teardown`). The `hook.session_started` /
//! `hook.session_ended` events emitted by the HTTP ingress in
//! `http_ingress` are part of the canonical `hook.*` family and stay.

use anyhow::{Result, anyhow};
use serde::Deserialize;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::types::{
    SkillCompleted, SkillFailed, SkillStarted, event_types,
};

mod soft_delete;
pub mod tool_gate;
pub mod http_ingress;

#[cfg(test)]
mod hooks_tests;
#[cfg(test)]
mod tests;

/// Extracts skill_id from a ship MCP tool name.
/// "mcp__ship__commit" -> Some("commit")
/// Non-ship tools -> None
pub fn extract_skill_id(tool_name: &str) -> Option<&str> {
    tool_name.strip_prefix("mcp__ship__")
}

// hook_event_name and mcp_context are deserialized for schema compatibility
// but dispatch is based on tool_name alone.
#[allow(dead_code)]
#[derive(Deserialize)]
struct ProviderHook {
    hook_event_name: String,
    tool_name: Option<String>,
    tool_input: Option<serde_json::Value>,
    tool_response: Option<serde_json::Value>,
    mcp_context: Option<serde_json::Value>,
}

/// Parses harness hook stdin JSON and emits skill.started.
/// Returns Ok(None) for non-ship tools (ignore silently).
/// Supports Claude PreToolUse and Cursor beforeMCPExecution schemas.
///
/// Also performs soft-delete: copies files to trash before destructive tool calls.
pub fn handle_before_tool(
    stdin_json: &str,
    actor_id: &str,
    // Used for soft-delete trash placement. The emitted skill.* event is
    // Runtime-owned, so the workspace is NOT stamped as the envelope scope.
    workspace_id: &str,
) -> Result<Option<EventEnvelope>> {
    if stdin_json.is_empty() {
        return Err(anyhow!("stdin is empty"));
    }
    let hook: ProviderHook = serde_json::from_str(stdin_json)
        .map_err(|e| anyhow!("invalid JSON in hook stdin: {e}"))?;

    let tool_name = hook.tool_name.as_deref().unwrap_or("");
    // Best-effort soft-delete for destructive tool calls.
    // Errors are logged but never block tool execution.
    soft_delete::maybe_trash_before_tool(
        tool_name,
        hook.tool_input.as_ref(),
        workspace_id,
    );

    let skill_id = match extract_skill_id(tool_name) {
        Some(id) => id.to_string(),
        None => return Ok(None),
    };
    let event = EventEnvelope::new(
        event_types::SKILL_STARTED,
        &skill_id,
        &SkillStarted { skill_id: skill_id.clone() },
    )?
    .with_actor_id(actor_id);
    Ok(Some(event))
}

/// Parses harness hook stdin JSON and emits skill.completed or skill.failed.
/// Returns Ok(None) for non-ship tools.
pub fn handle_after_tool(
    stdin_json: &str,
    actor_id: &str,
    // Runtime-owned event: workspace recorded by payload, not envelope scope.
    _workspace_id: &str,
) -> Result<Option<EventEnvelope>> {
    if stdin_json.is_empty() {
        return Err(anyhow!("stdin is empty"));
    }
    let hook: ProviderHook = serde_json::from_str(stdin_json)
        .map_err(|e| anyhow!("invalid JSON in hook stdin: {e}"))?;
    let skill_id = match extract_skill_id(hook.tool_name.as_deref().unwrap_or("")) {
        Some(id) => id.to_string(),
        None => return Ok(None),
    };
    let event = if let Some(error_msg) = hook.tool_response.as_ref().and_then(extract_error) {
        EventEnvelope::new(
            event_types::SKILL_FAILED,
            &skill_id,
            &SkillFailed { skill_id: skill_id.clone(), error: error_msg },
        )?
    } else {
        EventEnvelope::new(
            event_types::SKILL_COMPLETED,
            &skill_id,
            &SkillCompleted { skill_id: skill_id.clone(), duration_ms: None },
        )?
    };
    Ok(Some(event.with_actor_id(actor_id)))
}

fn extract_error(response: &serde_json::Value) -> Option<String> {
    let err = response.get("error")?;
    if err.is_null() {
        return None;
    }
    Some(err.as_str().unwrap_or("unknown error").to_string())
}
