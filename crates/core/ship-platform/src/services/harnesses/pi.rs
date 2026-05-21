//! Pi CLI adapter — `pi` binary, `--mode json` for headless runs with a
//! structured JSON event stream (pi.dev/docs/latest/json).
//!
//! Pi also exposes an RPC mode (stdin/stdout JSONL) suited to a persistent
//! server session; that's the natural prototype for the harness-conformance
//! protocol and is a follow-up. This adapter wires the headless json path.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::harness_adapter::{HarnessAdapter, HarnessCapabilities, HarnessEvent, ResumeError};

/// Shared headless arg prefix: non-interactive (`--print`) + JSON event
/// stream (`--mode json`). Verified against pi v0.75.4.
fn headless_base(profile: &LaunchProfile) -> Vec<String> {
    let mut args = vec![
        "--print".to_string(),
        "--mode".to_string(),
        "json".to_string(),
    ];
    // Pi's --model accepts "provider/id" patterns; provider defaults to
    // google otherwise. Model selection still resolves a provider API key
    // from the environment.
    if let Some(model) = &profile.model {
        args.push("--model".to_string());
        args.push(model.clone());
    }
    args
}

pub struct PiAdapter;

#[async_trait::async_trait]
impl HarnessAdapter for PiAdapter {
    fn id(&self) -> &str { "pi" }

    fn caps(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            headless: true,
            interactive: true,
            // RPC/server mode exists but isn't wired here yet (follow-up).
            server_mode: false,
            json_stream: true,
            // MCP + hooks support unconfirmed against the live CLI; declare
            // conservatively until verified.
            mcp: false,
            hooks: false,
            // Pi persists sessions and resumes them by id (`--session <id>`,
            // also -c/-r). Verified in `pi --help` v0.75.4.
            session_resume: true,
            persistent_session: true,
            push_mechanism: super::super::harness_adapter::PushMechanism::Stdin,
        }
    }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        // Interactive TUI — no --print. Model still selectable via --model.
        let mut args = Vec::new();
        if let Some(model) = &profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "pi".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        // `--print` exits after the prompt (non-interactive); `--mode json`
        // emits the event stream parse_output consumes. The prompt is the
        // positional arg. Both flags required — `--mode json` alone leaves
        // the TUI running.
        let mut args = headless_base(profile);
        args.push(
            profile
                .autostart_prompt
                .clone()
                .unwrap_or_else(|| "Read .artifacts/job-spec.md and execute it.".to_string()),
        );
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "pi".to_string(), args, env }
    }

    fn build_resume_args(
        &self,
        slug: &str,
        session_id: &str,
        profile: &LaunchProfile,
    ) -> Result<LaunchCommand, ResumeError> {
        if session_id.trim().is_empty() {
            return Err(ResumeError::BadSession {
                harness: self.id().to_string(),
                reason: "empty session id".to_string(),
            });
        }
        // `--session <id>` resumes a saved session by (partial) UUID; pair
        // with the headless flags and feed the next turn as the prompt.
        let mut args = headless_base(profile);
        args.push("--session".to_string());
        args.push(session_id.to_string());
        args.push(
            profile
                .autostart_prompt
                .clone()
                .unwrap_or_else(|| "Continue.".to_string()),
        );
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        Ok(LaunchCommand { bin: "pi".to_string(), args, env })
    }

    fn ready_pattern(&self) -> &str { "› " }
    fn autostart_input(&self) -> &str { "go" }
    fn exit_command(&self) -> &str { "" }

    /// Map one `--mode json` event line to a HarnessEvent. Pi reports no
    /// token usage, so there's no TurnUsage; it does emit a session id, so
    /// SessionStarted is surfaced. Schema: pi.dev/docs/latest/json.
    fn parse_output(&self, line: &str) -> Option<HarnessEvent> {
        let v: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
        match v.get("type")?.as_str()? {
            // The header line carries the session id.
            "session" => Some(HarnessEvent::SessionStarted {
                session_id: v.get("id")?.as_str()?.to_string(),
            }),
            // message_end carries the completed assistant message; join its
            // text parts. message_start/_update (deltas) are skipped so we
            // emit one Message per assistant message, not per token.
            "message_end" => {
                let text: String = v
                    .get("message")?
                    .get("content")?
                    .as_array()?
                    .iter()
                    .filter_map(|part| part.get("text")?.as_str())
                    .collect();
                (!text.is_empty()).then_some(HarnessEvent::Message { text })
            }
            // Surface the start of each tool call; update/end would
            // double-count. args carry path/command for the preview.
            "tool_execution_start" => {
                let tool = v.get("toolName")?.as_str()?.to_string();
                let input_preview = v
                    .get("args")
                    .and_then(|a| {
                        a.get("path")
                            .or_else(|| a.get("command"))
                            .or_else(|| a.get("file_path"))
                    })
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(HarnessEvent::ToolUse { tool, input_preview })
            }
            // A turn finished. Pi reports no stop reason here.
            "turn_end" => Some(HarnessEvent::TurnComplete {
                stop_reason: "end_turn".to_string(),
            }),
            _ => None,
        }
    }

    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_pi.rs"]
mod tests;
