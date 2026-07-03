//! Claude Code CLI adapter.

use std::collections::HashMap;


use harness_adapter::dispatch_ports::{LaunchCommand, LaunchProfile};
use harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, ResumeError,
};

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
            // Claude channel state — CURRENTLY DEGRADED. See
            // .artifacts/agents-arc/discussions/claude-channel-dialog-blocker.md
            //
            // Short version: `server:` channels (used for Ship's push-to-
            // agent Channel mechanism) require
            // --dangerously-load-development-channels, which triggers an
            // interactive confirmation dialog that blocks the TUI in
            // dispatched-keepalive mode (no user to press Enter).
            //
            // Today we pass --channels server:ship alone: claude survives,
            // channel is declared, but server: entries are inactive
            // (claude logs "server: entries need
            // --dangerously-load-development-channels"). Push via Channel
            // mechanism HTTP-202s and emits studio.chat_turn, but
            // delivery to claude's MCP subscriber doesn't actually
            // happen.
            //
            // Proper fix: PTY-side dialog auto-dismissal in the
            // supervisor when keepalive=true. Substantial work; tracked.
            "--channels".to_string(),
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

#[async_trait::async_trait]
impl HarnessAdapter for ClaudeAdapter {
    fn id(&self) -> &str { "claude" }

    /// Claude Code's CLI is invoked as `claude`; configs also name it
    /// "claude-code". Both resolve to this adapter.
    fn matches(&self, id: &str) -> bool { id == "claude" || id == "claude-code" }

    fn caps(&self) -> HarnessCapabilities {
        HarnessCapabilities {
            headless: true,
            interactive: true,
            server_mode: false,
            json_stream: true,
            mcp: true,
            hooks: true,
            // Claude supports `--resume <uuid>` end-to-end. The original
            // adapter audit pinned this `false`; the cross-provider-resume
            // job corrects it now that build_resume_args is wired.
            session_resume: true,
            persistent_session: true,
            push_mechanism: harness_adapter::PushMechanism::Channel,
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
        // `base_args` already maps `profile.resume_session` to `--resume <id>`
        // + `--fork-session`. Reuse that path so resume launches share the
        // same flag shape as the rest of dispatch (--mcp-config, --name,
        // --model, etc.).
        let mut p = profile.clone();
        p.resume_session = Some(session_id.to_string());
        Ok(self.build_headless(slug, &p))
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

    fn parse_output(&self, line: &str) -> Option<HarnessEvent> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let ty = v.get("type")?.as_str()?;
        match ty {
            // Session bootstrap: capture session id for next --resume.
            "system" => {
                let subtype = v.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
                if subtype == "init" {
                    if let Some(sid) = v.get("session_id").and_then(|s| s.as_str()) {
                        return Some(HarnessEvent::SessionStarted {
                            session_id: sid.to_string(),
                        });
                    }
                }
                Some(HarnessEvent::Custom {
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
                            return Some(HarnessEvent::Message {
                                text: text.to_string(),
                            });
                        }
                    } else if bt == "tool_use" {
                        if let Some(name) = block.get("name").and_then(|n| n.as_str()) {
                            return Some(HarnessEvent::ToolUse {
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

            // Turn completion + metadata + token usage.
            //
            // In `-p` stream-json mode the result event carries aggregate
            // usage for the whole run plus claude's own cost calc
            // (total_cost_usd) — we capture tokens and let cost stay
            // a derived view (subscription users shouldn't see dollar
            // amounts for sessions that have no marginal cost).
            "result" => {
                let usage = v.get("usage");
                let input_tokens = usage
                    .and_then(|u| u.get("input_tokens"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0);
                let output_tokens = usage
                    .and_then(|u| u.get("output_tokens"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0);
                let cache_creation_tokens = usage
                    .and_then(|u| u.get("cache_creation_input_tokens"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0);
                let cache_read_tokens = usage
                    .and_then(|u| u.get("cache_read_input_tokens"))
                    .and_then(|n| n.as_u64())
                    .unwrap_or(0);
                let stop = v
                    .get("stop_reason")
                    .and_then(|s| s.as_str())
                    .unwrap_or("end_turn")
                    .to_string();
                // Only surface as TurnUsage when at least one token count
                // is non-zero; otherwise fall back to TurnComplete
                // (claude has emitted bare `result` events without
                // usage in some shapes — staying robust).
                if input_tokens + output_tokens + cache_creation_tokens + cache_read_tokens > 0 {
                    // claude reports total_cost_usd in result events
                    // (API-key sessions report a real number; subscription
                    // sessions may report 0 or the API-equivalent value).
                    // We capture verbatim — never recompute.
                    let harness_reported_cost_usd = v
                        .get("total_cost_usd")
                        .and_then(|c| c.as_f64());
                    Some(HarnessEvent::TurnUsage {
                        model: None,
                        input_tokens,
                        output_tokens,
                        reasoning_tokens: 0,
                        cache_creation_tokens,
                        cache_read_tokens,
                        stop_reason: Some(stop),
                        harness_reported_cost_usd,
                    })
                } else {
                    Some(HarnessEvent::TurnComplete { stop_reason: stop })
                }
            }

            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "tests_claude.rs"]
mod tests;
