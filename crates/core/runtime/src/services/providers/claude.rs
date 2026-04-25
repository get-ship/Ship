//! Claude Code CLI adapter.

use std::collections::HashMap;
use std::path::Path;

use crate::services::dispatch_ports::{LaunchCommand, LaunchProfile};
use crate::services::provider_adapter::{ProviderAdapter, ProviderCapabilities, ProviderEvent};

pub struct ClaudeAdapter;

impl ClaudeAdapter {
    fn base_args(slug: &str, profile: &LaunchProfile) -> (Vec<String>, HashMap<String, String>) {
        // Register the Ship MCP server under a private name (`ship-bridge`)
        // via inline JSON so it cannot be silently disabled by Claude's
        // per-project trust gate on the cwd `.mcp.json`. The trust gate
        // pins the `ship` name to "disabled" the first time a user
        // dismisses Claude's project-trust prompt; using a Ship-managed
        // name avoids that user-state entirely. Tool names become
        // `mcp__ship-bridge__<name>`; the agent's tool resolver matches
        // by suffix so prompts can keep using `chat_reply` etc.
        let inline_mcp = r#"{"mcpServers":{"ship-bridge":{"command":"ship","args":["mcp","serve"]}}}"#;
        let mut args = vec![
            "--dangerously-skip-permissions".to_string(),
            "--dangerously-load-development-channels".to_string(),
            "server:ship".to_string(),
            "--mcp-config".to_string(),
            inline_mcp.to_string(),
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
            // `--fork-session` re-initialises MCP server connections for
            // the new turn while still carrying the conversation context
            // forward. Without it, resumed turns inherit the original
            // session's MCP set (which was empty in the cwd-trust-disabled
            // first turn) and our `--mcp-config` is silently ignored.
            // The session id rotates per turn — captured by the adapter's
            // SessionStarted parse and stored in meta for the next call.
            args.push("--fork-session".to_string());
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

        if let Some(ref sys) = profile.append_system_prompt {
            args.push("--append-system-prompt".to_string());
            args.push(sys.clone());
        }

        let mut env = HashMap::new();
        env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());

        (args, env)
    }
}

impl ProviderAdapter for ClaudeAdapter {
    fn id(&self) -> &str { "claude" }

    fn caps(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            headless: true,
            interactive: true,
            server_mode: false,
            json_stream: true,
            mcp: true,
            hooks: true,
            session_resume: false,
            channels: true,
        }
    }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let (args, env) = Self::base_args(slug, profile);
        LaunchCommand { bin: "claude".to_string(), args, env }
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        let (mut args, env) = Self::base_args(slug, profile);
        let prompt = profile.autostart_prompt.clone().unwrap_or_else(|| "go".to_string());
        args.extend([
            "--verbose".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--print".to_string(),
            "-p".to_string(),
            prompt,
        ]);
        LaunchCommand { bin: "claude".to_string(), args, env }
    }

    // `bypass permissions on` appears under the real input prompt and NOT in
    // any of claude's startup menus (channels confirmation, model picker,
    // etc.). Using `❯` alone false-matched the menu selector and fired
    // autostart before the menu was dismissed. Lost a real e2e run to this.
    fn ready_pattern(&self) -> &str { "bypass permissions on" }
    fn autostart_input(&self) -> &str {
        "Read .artifacts/job-spec.md and execute it autonomously."
    }
    fn exit_command(&self) -> &str { "/exit" }

    fn parse_output(&self, line: &str) -> Option<ProviderEvent> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let ty = v.get("type")?.as_str()?;
        match ty {
            // Session bootstrap: capture session id for next --resume.
            "system" => {
                let subtype = v.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
                if subtype == "init" {
                    if let Some(sid) = v.get("session_id").and_then(|s| s.as_str()) {
                        return Some(ProviderEvent::SessionStarted {
                            session_id: sid.to_string(),
                        });
                    }
                }
                Some(ProviderEvent::Custom {
                    kind: subtype.to_string(),
                    data: v,
                })
            }

            // Whole assistant turn — current stream-json shape. Claude
            // emits this with the full content array. Flatten the first
            // content block (the common one-text case) into a Message;
            // tool_use blocks emit ToolUse.
            "assistant" => {
                let content = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())?;
                for block in content {
                    let bt = block.get("type")?.as_str()?;
                    if bt == "text" {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            return Some(ProviderEvent::Message {
                                text: text.to_string(),
                            });
                        }
                    } else if bt == "tool_use" {
                        if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                            return Some(ProviderEvent::ToolUse {
                                tool: name.to_string(),
                                input_preview: block
                                    .get("input")
                                    .map(|i| i.to_string())
                                    .unwrap_or_default(),
                            });
                        }
                    }
                }
                None
            }

            // Turn completion + metadata.
            "result" => {
                let stop = v
                    .get("stop_reason")
                    .and_then(|s| s.as_str())
                    .unwrap_or("end_turn")
                    .to_string();
                Some(ProviderEvent::TurnComplete { stop_reason: stop })
            }

            // Legacy stream_event format — kept as a fallback.
            "stream_event" => {
                let event = v.get("event")?;
                match event.get("type")?.as_str()? {
                    "content_block_delta" => {
                        let delta = event.get("delta")?;
                        if delta.get("type")?.as_str()? == "text_delta" {
                            Some(ProviderEvent::Message {
                                text: delta.get("text")?.as_str()?.to_string(),
                            })
                        } else {
                            None
                        }
                    }
                    "message_delta" => {
                        let u = event.get("usage")?;
                        let output_tokens = u.get("output_tokens")?.as_u64()?;
                        Some(ProviderEvent::Usage {
                            input_tokens: 0,
                            output_tokens,
                        })
                    }
                    _ => None,
                }
            }

            _ => None,
        }
    }

    fn extract_session_meta(&self, _work_dir: &Path) -> Option<serde_json::Value> { None }
}

#[cfg(test)]
#[path = "tests_claude.rs"]
mod tests;
