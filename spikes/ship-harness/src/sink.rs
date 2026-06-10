//! TerminalSink — the spike's stand-in for a Ship SUI view. Renders the
//! `AgentEvent` stream to the terminal. In the product this exact event stream
//! drives a reactive view subscribed to the session's events.

use std::io::Write;

use async_trait::async_trait;

use crate::session::{AgentEvent, EventSink};

pub struct TerminalSink;

#[async_trait]
impl EventSink for TerminalSink {
    async fn emit(&self, ev: AgentEvent) {
        match ev {
            AgentEvent::Status { state } => {
                eprint!("\x1b[2m[{state}]\x1b[0m ");
                let _ = std::io::stderr().flush();
            }
            AgentEvent::AssistantDelta { text } => {
                print!("{text}");
                let _ = std::io::stdout().flush();
            }
            AgentEvent::Reasoning { text } => {
                eprint!("\x1b[2;3m{text}\x1b[0m");
                let _ = std::io::stderr().flush();
            }
            AgentEvent::ToolCall { name, args, .. } => {
                let a = serde_json::to_string(&args).unwrap_or_default();
                let a: String = a.chars().take(80).collect();
                println!("\n\x1b[36m  ⚙ {name}({a})\x1b[0m");
            }
            AgentEvent::ToolResult { ok, summary, .. } => {
                let mark = if ok { "\x1b[32m✓\x1b[0m" } else { "\x1b[31m✗\x1b[0m" };
                println!("  {mark} {summary}");
            }
            AgentEvent::ApprovalRequested { what, tier, .. } => {
                println!("\n\x1b[33m  ⚠ approval needed [{tier}]: {what}\x1b[0m");
            }
            AgentEvent::Completed {
                usage_in,
                usage_out,
            } => {
                eprintln!("\n\x1b[2m[done · in {usage_in} / out {usage_out} tok]\x1b[0m");
            }
            AgentEvent::Error { message } => {
                eprintln!("\n\x1b[31m[error] {message}\x1b[0m");
            }
        }
    }
}
