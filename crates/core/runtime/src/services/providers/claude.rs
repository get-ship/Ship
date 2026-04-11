//! Claude Code CLI adapter.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::provider_adapter::{ProviderAdapter, ProviderEvent};

pub struct ClaudeAdapter;

impl ClaudeAdapter {
    fn base_args(slug: &str, profile: &LaunchProfile) -> (Vec<String>, HashMap<String, String>) {
        let mut args = vec![
            "--dangerously-skip-permissions".to_string(),
            "--dangerously-load-development-channels".to_string(),
            "server:ship".to_string(),
        ];

        if let Some(ref name) = profile.session_name {
            args.push("--name".to_string());
            args.push(name.clone());
        } else {
            args.push("--name".to_string());
            args.push(format!("job-{slug}"));
        }

        if let Some(ref session_id) = profile.resume_session {
            args.push("--resume".to_string());
            args.push(session_id.clone());
        }

        if let Some(ref model) = profile.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        if let Some(ref effort) = profile.effort {
            args.push("--effort".to_string());
            args.push(effort.clone());
        }

        if let Some(budget) = profile.max_budget_usd {
            args.push("--max-budget-usd".to_string());
            args.push(format!("{budget}"));
        }

        if let Some(ref mode) = profile.permission_mode {
            args.push("--permission-mode".to_string());
            args.push(mode.clone());
        }

        if !profile.persist_session {
            args.push("--no-session-persistence".to_string());
        }

        let mut env = HashMap::new();
        env.insert("SHIP_MESH_ID".to_string(), slug.to_string());

        (args, env)
    }
}

impl ProviderAdapter for ClaudeAdapter {
    fn id(&self) -> &str { "claude" }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let (args, env) = Self::base_args(slug, profile);
        LaunchCommand { bin: "claude".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let (mut args, env) = Self::base_args(slug, profile);
        args.push("--output-format".to_string());
        args.push("stream-json".to_string());
        args.push("--print".to_string());
        args.push("-p".to_string());
        args.push("go".to_string());
        LaunchCommand { bin: "claude".to_string(), args, env }
    }

    fn ready_pattern(&self) -> &str { "❯" }
    fn autostart_input(&self) -> &str {
        "Read .artifacts/job-spec.md and execute it autonomously."
    }
    fn exit_command(&self) -> &str { "/exit" }

    fn parse_output(&self, line: &str) -> Option<ProviderEvent> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        if v.get("type")?.as_str()? == "system" {
            return Some(ProviderEvent::Custom {
                kind: v.get("subtype")?.as_str()?.to_string(),
                data: v,
            });
        }
        if v.get("type")?.as_str()? != "stream_event" { return None; }
        let event = v.get("event")?;
        match event.get("type")?.as_str()? {
            "message_start" => {
                let session_id = v.get("session_id")?.as_str()?.to_string();
                Some(ProviderEvent::SessionStarted { session_id })
            }
            "content_block_start" => {
                let block = event.get("content_block")?;
                if block.get("type")?.as_str()? == "tool_use" {
                    Some(ProviderEvent::ToolUse {
                        tool: block.get("name")?.as_str()?.to_string(),
                        input_preview: String::new(),
                    })
                } else { None }
            }
            "content_block_delta" => {
                let delta = event.get("delta")?;
                if delta.get("type")?.as_str()? == "text_delta" {
                    Some(ProviderEvent::Message { text: delta.get("text")?.as_str()?.to_string() })
                } else { None }
            }
            "message_delta" => {
                if let Some(u) = event.get("usage") {
                    let output_tokens = u.get("output_tokens")?.as_u64()?;
                    return Some(ProviderEvent::Usage { input_tokens: 0, output_tokens });
                }
                None
            }
            _ => None,
        }
    }

    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_claude.rs"]
mod tests;
