//! OpenCode CLI adapter — `opencode` binary with `run` subcommand.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::harness_adapter::{HarnessAdapter, HarnessCapabilities, HarnessEvent};

pub struct OpenCodeAdapter;

impl HarnessAdapter for OpenCodeAdapter {
    fn id(&self) -> &str { "opencode" }

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
        let mut args = Vec::new();
        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }
        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
        LaunchCommand { bin: "opencode".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        // `opencode run` takes the message as positional args; no dedicated
        // bypass flag exists in the current CLI (unlike Claude's `--yolo` /
        // Codex's `--full-auto`). Server-mode via `opencode serve` is the
        // primary automation path — CLI is a fallback when the session
        // can't bind.
        let mut args = vec!["run".to_string()];
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

    fn parse_output(&self, _line: &str) -> Option<HarnessEvent> { None }
    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_opencode.rs"]
mod tests;
