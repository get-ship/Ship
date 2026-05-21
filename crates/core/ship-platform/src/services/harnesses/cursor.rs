//! Cursor CLI adapter — `cursor-agent` binary, `-p --output-format
//! stream-json` for headless runs with parseable events.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::harness_adapter::{HarnessAdapter, HarnessCapabilities, HarnessEvent};

pub struct CursorAdapter;

#[async_trait::async_trait]
impl HarnessAdapter for CursorAdapter {
    fn id(&self) -> &str { "cursor" }

    /// Cursor's CLI installs as `cursor-agent` (the `agent` alias is a
    /// symlink to the same binary). Verified against cursor-agent
    /// v2026.05.20. Configs key it as "cursor"; the executable is this.
    fn binary(&self) -> &str { "cursor-agent" }

    fn caps(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            headless: true,
            interactive: true,
            server_mode: false,
            json_stream: true,
            mcp: false,
            hooks: true,
            session_resume: false,
            persistent_session: true,
            push_mechanism: super::super::harness_adapter::PushMechanism::Stdin,
        }
    }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let mut args = Vec::new();
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "cursor-agent".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        // `--output-format stream-json` (only valid with `-p`) makes the run
        // emit JSONL events that parse_output maps to HarnessEvents; without
        // it the run prints plain text and analytics see nothing.
        let mut args = vec![
            "-p".to_string(),
            "--force".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
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
        LaunchCommand { bin: "cursor-agent".to_string(), args, env }
    }

    fn ready_pattern(&self) -> &str { "$ " }
    fn autostart_input(&self) -> &str { "go" }
    fn exit_command(&self) -> &str { "" }

    /// Map one `--output-format stream-json` line to a HarnessEvent.
    /// Cursor emits no session id and no token usage, so it produces
    /// Message / ToolUse / TurnComplete only (no SessionStarted or
    /// TurnUsage) — consistent with `emitsSessionStarted: false` in the
    /// harness-compat registry. Schema verified against cursor docs;
    /// field names: cursor.com/docs/cli/headless.
    fn parse_output(&self, line: &str) -> Option<HarnessEvent> {
        let v: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
        match v.get("type")?.as_str()? {
            // Assistant deltas carry message.content[].text; join the text
            // parts. Skip deltas with no text (tool-only turns, etc.).
            "assistant" => {
                let text: String = v
                    .get("message")?
                    .get("content")?
                    .as_array()?
                    .iter()
                    .filter_map(|part| part.get("text")?.as_str())
                    .collect();
                (!text.is_empty()).then_some(HarnessEvent::Message { text })
            }
            // Surface the *start* of a tool call. tool_call is keyed by the
            // variant (readToolCall, writeToolCall, …); strip the ToolCall
            // suffix for the name and preview path/command from args.
            // `completed` events would double-count, so drop them.
            "tool_call" => {
                if v.get("subtype")?.as_str()? != "started" {
                    return None;
                }
                let (key, body) = v.get("tool_call")?.as_object()?.iter().next()?;
                let tool = key.strip_suffix("ToolCall").unwrap_or(key).to_string();
                let input_preview = body
                    .get("args")
                    .and_then(|a| a.get("path").or_else(|| a.get("command")))
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(HarnessEvent::ToolUse { tool, input_preview })
            }
            // Terminal event ends the turn. Cursor reports duration but no
            // stop reason; flag is_error when the field is present.
            "result" => {
                let stop_reason = if v.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false) {
                    "error"
                } else {
                    "end_turn"
                };
                Some(HarnessEvent::TurnComplete { stop_reason: stop_reason.to_string() })
            }
            // system/init carries the model but no session id — cursor has no
            // resume and emits no SessionStarted.
            _ => None,
        }
    }
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_cursor.rs"]
mod tests;
