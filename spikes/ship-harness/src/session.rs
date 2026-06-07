//! The user-facing channel — a typed `AgentEvent` stream.
//!
//! This is how the user interfaces with the agent. The agent emits events; a
//! sink renders them. In the product the sink is the Ship event bus feeding a
//! SUI view (reactive); in the spike it's a terminal renderer. User input
//! (prompt / steer / approve / cancel) flows back via dispatch in prod, stdin
//! here.
//!
//! `AgentEvent` is intentionally ACP-shaped (content deltas, tool-call updates,
//! approval requests) so a thin ACP adapter is cheap later for editor interop —
//! but it rides Ship's own event bus, NOT ACP-over-stdio.

use async_trait::async_trait;
use futures::StreamExt;
use serde::Serialize;
use serde_json::Value;

use rig::agent::{Agent, MultiTurnStreamItem};
use rig::completion::CompletionModel;
use rig::message::Message;
use rig::streaming::{StreamedAssistantContent, StreamedUserContent, StreamingChat};

/// What the UI receives. Tagged for clean JSON over the bus.
#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentEvent {
    /// Coarse lifecycle: "thinking" | "running" | "idle".
    Status { state: &'static str },
    /// A streamed assistant text delta (token-ish).
    AssistantDelta { text: String },
    /// Streamed chain-of-thought, when the provider exposes it.
    Reasoning { text: String },
    /// The model asked to run a tool.
    ToolCall { id: String, name: String, args: Value },
    /// A tool finished (post-spine: this is the accepted/verified result).
    ToolResult { id: String, ok: bool, summary: String },
    /// The permit gate is escalating to the human. The UI prompts; the answer
    /// returns via `UserInput::Approve`. Wiring the mid-stream pause is the next
    /// increment — the event + seam are defined here so the UI contract is final.
    #[allow(dead_code)]
    ApprovalRequested { id: String, what: String, tier: String },
    /// Turn finished. Usage drives metering for the future hosted tier.
    Completed { usage_in: u64, usage_out: u64 },
    /// Something failed; surfaced, not swallowed.
    Error { message: String },
}

/// Where events go. SUI view (prod) / terminal (spike) implement this.
#[async_trait]
pub trait EventSink: Send + Sync {
    async fn emit(&self, ev: AgentEvent);
}

/// User → agent control. Arrives via dispatch/steer ops in prod, stdin here.
#[allow(dead_code)]
pub enum UserInput {
    Prompt(String),
    Steer(String),
    Approve { id: String, allow: bool },
    Cancel,
}

/// Run one streamed turn, translating rig's stream into `AgentEvent`s. History
/// is threaded so the next turn continues the conversation.
pub async fn run_turn<M>(
    agent: &Agent<M>,
    prompt: &str,
    history: &mut Vec<Message>,
    sink: &dyn EventSink,
) where
    M: CompletionModel + 'static,
{
    sink.emit(AgentEvent::Status { state: "thinking" }).await;
    let mut stream = agent.stream_chat(prompt, &*history).await;

    while let Some(item) = stream.next().await {
        match item {
            Ok(MultiTurnStreamItem::StreamAssistantItem(content)) => match content {
                StreamedAssistantContent::Text(t) => {
                    sink.emit(AgentEvent::AssistantDelta { text: t.text }).await;
                }
                StreamedAssistantContent::ToolCall { tool_call, .. } => {
                    sink.emit(AgentEvent::ToolCall {
                        id: tool_call.id,
                        name: tool_call.function.name,
                        args: tool_call.function.arguments,
                    })
                    .await;
                }
                StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                    sink.emit(AgentEvent::Reasoning { text: reasoning }).await;
                }
                _ => {}
            },
            Ok(MultiTurnStreamItem::StreamUserItem(StreamedUserContent::ToolResult {
                tool_result,
                ..
            })) => {
                let summary = serde_json::to_string(&tool_result.content)
                    .unwrap_or_default()
                    .chars()
                    .take(160)
                    .collect();
                sink.emit(AgentEvent::ToolResult {
                    id: tool_result.id,
                    ok: true,
                    summary,
                })
                .await;
            }
            Ok(MultiTurnStreamItem::FinalResponse(fin)) => {
                if let Some(h) = fin.history() {
                    *history = h.to_vec();
                }
                let u = fin.usage();
                sink.emit(AgentEvent::Completed {
                    usage_in: u.input_tokens,
                    usage_out: u.output_tokens,
                })
                .await;
            }
            Ok(_) => {}
            Err(e) => {
                sink.emit(AgentEvent::Error {
                    message: e.to_string(),
                })
                .await;
            }
        }
    }
    sink.emit(AgentEvent::Status { state: "idle" }).await;
}
