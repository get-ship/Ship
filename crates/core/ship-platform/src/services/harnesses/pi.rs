//! Pi CLI adapter — `pi` binary, `--mode json` for headless runs with a
//! structured JSON event stream (pi.dev/docs/latest/json).
//!
//! Pi also exposes an RPC mode (stdin/stdout JSONL) suited to a persistent
//! server session; that's the natural prototype for the harness-conformance
//! protocol and is a follow-up. This adapter wires the headless json path.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::harness_adapter::{HarnessAdapter, HarnessCapabilities, HarnessEvent};

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
            // Pi has sessions, but the resume CLI flag is unverified; keep
            // resume off so dispatch 400s rather than silently fresh-launch.
            session_resume: false,
            persistent_session: true,
            push_mechanism: super::super::harness_adapter::PushMechanism::Stdin,
        }
    }

    fn build_interactive(&self, slug: &str, _profile: &LaunchProfile) -> LaunchCommand {
        // Pi selects its model via config + provider API key env (e.g.
        // ANTHROPIC_API_KEY), not a CLI flag we can rely on yet.
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "pi".to_string(), args: Vec::new(), env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        // `--mode json` runs non-interactively and emits the JSON event
        // stream parse_output consumes; the prompt is the positional arg.
        let prompt = profile
            .autostart_prompt
            .clone()
            .unwrap_or_else(|| "Read .artifacts/job-spec.md and execute it.".to_string());
        let args = vec!["--mode".to_string(), "json".to_string(), prompt];
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "pi".to_string(), args, env }
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
