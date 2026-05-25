//! OpenCode CLI adapter — `opencode` binary with `run` subcommand.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, ResumeError,
};

pub struct OpenCodeAdapter;

#[async_trait::async_trait]
impl HarnessAdapter for OpenCodeAdapter {
    fn id(&self) -> &str { "opencode" }

    /// OpenCode runs a server (`opencode serve`) we hold as a persistent
    /// session. Spawn it; on failure return None so routing falls back to the
    /// one-shot path. (Caps currently gate this off until the session client
    /// is rewritten for the new prompt API — see `caps`.)
    async fn spawn_server_session(
        &self,
    ) -> Option<std::sync::Arc<dyn ship_substrate::services::server_session::ServerSession>> {
        match super::opencode_session::OpenCodeSession::spawn().await {
            Ok(session) => Some(std::sync::Arc::new(session)),
            Err(err) => {
                tracing::warn!(%err, "opencode: server-session spawn failed; falling back to one-shot");
                None
            }
        }
    }

    fn caps(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            headless: true,
            interactive: true,
            // Opencode's `/session/:id/message` API changed shape in April
            // 2026 — body now expects `{parts:[{type,text}]}` instead of
            // `{role,content}`, and the response is synchronous (full
            // assistant message inline) rather than returning a messageID
            // that drives SSE subscription. OpenCodeSession still targets
            // the old shape; re-enable server_mode after the session code
            // is updated to use `/session/:id/prompt_async` + SSE, or to
            // parse the sync response into SessionEvents directly.
            server_mode: false,
            json_stream: true,
            mcp: true,
            hooks: false,
            session_resume: true,
            persistent_session: true,
            push_mechanism: super::super::harness_adapter::PushMechanism::Stdin,
        }
    }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        // Interactive TUI; for keepalive dispatch we still want
        // auto-approve so tool calls don't block.
        let mut args = vec!["--dangerously-skip-permissions".to_string()];
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "opencode".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        // `opencode run` with --format json gives a structured event
        // stream (step_start, text, tool, etc. — all carrying sessionID),
        // which parse_output below consumes for session-id capture +
        // tool-use observability synthesis. --dangerously-skip-permissions
        // auto-approves to match claude --dangerously-skip-permissions /
        // codex --full-auto / gemini --yolo.
        let mut args = vec![
            "run".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--dangerously-skip-permissions".to_string(),
        ];
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let prompt = profile
            .autostart_prompt
            .clone()
            .unwrap_or_else(|| "Read .artifacts/job-spec.md and execute it.".to_string());
        args.push(prompt);

        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());

        LaunchCommand { bin: "opencode".to_string(), args, env }
    }

    fn ready_pattern(&self) -> &str { "$ " }
    fn autostart_input(&self) -> &str { "go" }
    fn exit_command(&self) -> &str { "" }

    fn build_resume_args(
        &self,
        slug: &str,
        session_id: &str,
        profile: &LaunchProfile,
    ) -> Result<LaunchCommand, ResumeError> {
        if session_id.is_empty() {
            return Err(ResumeError::BadSession {
                harness: self.id().to_string(),
                reason: "empty session id".to_string(),
            });
        }
        // `opencode run --session <SESSION_ID> [prompt]` targets the
        // existing REST session UUID. Same shape as a fresh `opencode run`
        // otherwise so prompt + model overrides flow through unchanged.
        let mut args = vec![
            "run".to_string(),
            "--session".to_string(),
            session_id.to_string(),
        ];
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let prompt = profile
            .autostart_prompt
            .clone()
            .unwrap_or_else(|| "Continue.".to_string());
        args.push(prompt);

        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        Ok(LaunchCommand { bin: "opencode".to_string(), args, env })
    }

    /// Opencode `--format json` emits one event per line. Every event
    /// carries `sessionID`. Discriminator is the outer `type`:
    ///   step_start   — turn boundary; we use the first one to emit
    ///                  SessionStarted.
    ///   step_finish  — turn complete; reason field gives "tool-calls"
    ///                  or "stop".
    ///   text         — assistant text chunk; `part.text` is the body.
    ///   tool_use     — tool invocation. `part.tool` is the name;
    ///                  `part.state.status` is "completed" | "error"
    ///                  (opencode reports start+end in one event,
    ///                  unlike claude's separate before/after hooks).
    ///
    /// Once the first sessionID is observed we emit SessionStarted so
    /// the supervisor's tokio reader writes session.json (drives
    /// subsequent `--continue`/`--session` resumes via build_resume_args).
    fn parse_output(&self, line: &str) -> Option<HarnessEvent> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let ty = v.get("type")?.as_str()?;
        let part = v.get("part");
        let session_id = v.get("sessionID").and_then(|s| s.as_str());

        // First well-formed line carries sessionID — surface as
        // SessionStarted so the supervisor persists session.json.
        // Subsequent lines re-emit ToolUse / Message variants per type.
        if let Some(sid) = session_id
            && ty == "step_start"
        {
            return Some(HarnessEvent::SessionStarted {
                session_id: sid.to_string(),
            });
        }

        match ty {
            "step_finish" => {
                // step_finish carries token usage for the turn.
                // Shape: part.tokens = { total, input, output, reasoning,
                //   cache: { write, read } }. reason field gives stop_reason.
                let tokens = part.and_then(|p| p.get("tokens"))?;
                let input_tokens = tokens.get("input").and_then(|n| n.as_u64()).unwrap_or(0);
                let output_tokens = tokens.get("output").and_then(|n| n.as_u64()).unwrap_or(0);
                let reasoning_tokens =
                    tokens.get("reasoning").and_then(|n| n.as_u64()).unwrap_or(0);
                let cache = tokens.get("cache");
                let cache_creation_tokens = cache
                    .and_then(|c| c.get("write"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0);
                let cache_read_tokens = cache
                    .and_then(|c| c.get("read"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0);
                let stop_reason = part
                    .and_then(|p| p.get("reason"))
                    .and_then(|r| r.as_str())
                    .map(String::from);
                // opencode emits `cost` (number) on step_finish when the
                // provider returns cost info. We capture it as-is.
                let harness_reported_cost_usd = part
                    .and_then(|p| p.get("cost"))
                    .and_then(|c| c.as_f64());
                Some(HarnessEvent::TurnUsage {
                    model: None,
                    input_tokens,
                    output_tokens,
                    reasoning_tokens,
                    cache_creation_tokens,
                    cache_read_tokens,
                    stop_reason,
                    harness_reported_cost_usd,
                })
            }
            "tool_use" => {
                let tool = part
                    .and_then(|p| p.get("tool"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let input_preview = part
                    .and_then(|p| p.get("state"))
                    .and_then(|s| s.get("input"))
                    .map(|i| {
                        let s = i.to_string();
                        if s.len() > 400 { s.chars().take(400).collect() } else { s }
                    })
                    .unwrap_or_default();
                Some(HarnessEvent::ToolUse { tool, input_preview })
            }
            "text" => {
                let text = part
                    .and_then(|p| p.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    None
                } else {
                    Some(HarnessEvent::Message { text })
                }
            }
            _ => None,
        }
    }
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_opencode.rs"]
mod tests;
