//! AgentEvent stream — user-facing event vocabulary, sinks, and translation.
//!
//! Why: an interactive run (shell, UI, CLI) must be able to observe every
//! meaningful step without coupling to rig internals. This module owns the
//! vocabulary (`AgentEvent`), the sink abstraction, and a pure translation
//! function that maps local `LoopItem` values to events — testable without a
//! live model.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// Event vocabulary
// ---------------------------------------------------------------------------

/// Every observable step an agent turn can produce, ACP-shaped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    /// Lifecycle status message (start, step boundaries, …).
    Status { message: String },
    /// A streaming text chunk from the assistant.
    AssistantDelta { text: String },
    /// The model issued a tool call.
    ToolCall { name: String, args: Value },
    /// A tool returned a result.
    ToolResult { name: String, result: String },
    /// The turn completed successfully.
    Completed { turns: u32 },
    /// An unrecoverable error ended the run.
    Error { message: String },
    /// The loop is awaiting a human decision for an `Ask`-stance effect.
    /// `request_id` lets an interactive approver correlate the decision back.
    ApprovalRequested { op: String, args: Value, request_id: String },
    /// The run was cooperatively cancelled. Surfaced as data, not an error.
    Cancelled { reason: String },
}

/// The decision a human (or scripted approver) returns for an `Ask`-stance
/// effect. `Approve` lets the effect proceed through the one door; `Deny`
/// surfaces as a normal denial result — never an exception.
#[derive(Debug, Clone, PartialEq)]
pub enum UserInput {
    Approve { request_id: String },
    Deny { request_id: String, reason: String },
}

// ---------------------------------------------------------------------------
// Sink abstraction + impls
// ---------------------------------------------------------------------------

/// Receives events emitted during an agent run.
pub trait EventSink: Send + Sync {
    fn emit(&self, ev: &AgentEvent);
}

/// Collects every emitted event in order — used by tests and inspection.
#[derive(Default)]
pub struct CollectingSink {
    events: Mutex<Vec<AgentEvent>>,
}

impl CollectingSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of all events received so far.
    pub fn collected(&self) -> Vec<AgentEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl EventSink for CollectingSink {
    fn emit(&self, ev: &AgentEvent) {
        self.events.lock().unwrap().push(ev.clone());
    }
}

/// Prints each event to stdout — stand-in for a SUI terminal view.
pub struct TerminalSink;

impl EventSink for TerminalSink {
    fn emit(&self, ev: &AgentEvent) {
        match ev {
            AgentEvent::Status { message } => println!("[status] {message}"),
            AgentEvent::AssistantDelta { text } => print!("{text}"),
            AgentEvent::ToolCall { name, args } => println!("[tool→] {name}({args})"),
            AgentEvent::ToolResult { name, result } => println!("[←tool] {name}: {result}"),
            AgentEvent::Completed { turns } => println!("[done] completed in {turns} turn(s)"),
            AgentEvent::Error { message } => println!("[error] {message}"),
            AgentEvent::ApprovalRequested { op, request_id, .. } => {
                println!("[ask] {op} (id={request_id})")
            }
            AgentEvent::Cancelled { reason } => println!("[cancelled] {reason}"),
        }

    }
}

// ---------------------------------------------------------------------------
// EventLog — durable, append-only, replayable emit (S15)
// ---------------------------------------------------------------------------

/// A single recorded event with its monotonic sequence number.
#[derive(Debug, Clone, PartialEq)]
pub struct LoggedEvent {
    pub seq: u64,
    pub event: AgentEvent,
}

/// Durable, append-only event sink. Stamps each event with a monotonic
/// sequence number — deterministic, NO wall-clock, so order is the only thing
/// that matters — and stores it. `replay()` returns the ordered record.
///
/// This is the event-sourcing seam: in the daemon it becomes a persistent log
/// (`ship_subscriber`), here it is in-memory behind the SAME `EventSink` trait,
/// so every existing emit path feeds it unchanged. S14's design awaits child
/// runs over exactly this log; S15 gives it a concrete, replayable shape.
#[derive(Default)]
pub struct EventLog {
    seq: AtomicU64,
    entries: Mutex<Vec<LoggedEvent>>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// The ordered event record so far. Replaying the same run yields the same
    /// sequence — the foundation for recovery/reconciliation.
    pub fn replay(&self) -> Vec<LoggedEvent> {
        self.entries.lock().unwrap().clone()
    }
}

impl EventSink for EventLog {
    fn emit(&self, ev: &AgentEvent) {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        self.entries.lock().unwrap().push(LoggedEvent { seq, event: ev.clone() });
    }
}

// ---------------------------------------------------------------------------
// LoopItem — local mirror of the loop's observable steps
// ---------------------------------------------------------------------------

/// A single observable step produced by the agent loop.
/// This mirrors the loop's internal steps without depending on rig types,
/// keeping `translate` purely testable.
#[derive(Debug, Clone)]
pub enum LoopItem {
    /// A streaming text chunk from the assistant.
    TextDelta(String),
    /// The model called a tool.
    ToolCall { name: String, args: Value },
    /// A tool returned a result.
    ToolResult { name: String, result: String },
    /// The turn finished; `turns` counts completed model turns.
    Final { turns: u32 },
    /// An internal bookkeeping step the caller doesn't surface to users.
    Internal,
    /// The loop is awaiting a human decision for an `Ask`-stance effect.
    ApprovalRequested { op: String, args: Value, request_id: String },
    /// The run was cooperatively cancelled.
    Cancelled { reason: String },
}

// ---------------------------------------------------------------------------
// Translation
// ---------------------------------------------------------------------------

/// Maps a `LoopItem` to an `AgentEvent`, or `None` for items with no
/// user-facing representation (e.g. `Internal`).
pub fn translate(item: &LoopItem) -> Option<AgentEvent> {
    match item {
        LoopItem::TextDelta(text) => Some(AgentEvent::AssistantDelta { text: text.clone() }),
        LoopItem::ToolCall { name, args } => Some(AgentEvent::ToolCall {
            name: name.clone(),
            args: args.clone(),
        }),
        LoopItem::ToolResult { name, result } => Some(AgentEvent::ToolResult {
            name: name.clone(),
            result: result.clone(),
        }),
        LoopItem::Final { turns } => Some(AgentEvent::Completed { turns: *turns }),
        LoopItem::Internal => None,
        LoopItem::ApprovalRequested { op, args, request_id } => Some(
            AgentEvent::ApprovalRequested {
                op: op.clone(),
                args: args.clone(),
                request_id: request_id.clone(),
            },
        ),
        LoopItem::Cancelled { reason } => Some(AgentEvent::Cancelled { reason: reason.clone() }),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // CollectingSink records events in emission order.
    #[test]
    fn collecting_sink_records_in_order() {
        let sink = CollectingSink::new();
        let events = [
            AgentEvent::Status { message: "starting".into() },
            AgentEvent::AssistantDelta { text: "hello".into() },
            AgentEvent::Completed { turns: 1 },
        ];
        for ev in &events {
            sink.emit(ev);
        }
        let collected = sink.collected();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], AgentEvent::Status { message: "starting".into() });
        assert_eq!(collected[1], AgentEvent::AssistantDelta { text: "hello".into() });
        assert_eq!(collected[2], AgentEvent::Completed { turns: 1 });
    }

    // translate: TextDelta -> AssistantDelta
    #[test]
    fn translate_text_delta() {
        let item = LoopItem::TextDelta("hi".into());
        let ev = translate(&item).expect("should produce event");
        assert!(matches!(ev, AgentEvent::AssistantDelta { text } if text == "hi"));
    }

    // translate: ToolCall -> AgentEvent::ToolCall
    #[test]
    fn translate_tool_call() {
        let item = LoopItem::ToolCall {
            name: "ship_op".into(),
            args: json!({ "op": "collections.read" }),
        };
        let ev = translate(&item).expect("should produce event");
        assert!(matches!(ev, AgentEvent::ToolCall { name, .. } if name == "ship_op"));
    }

    // translate: ToolResult -> AgentEvent::ToolResult
    #[test]
    fn translate_tool_result() {
        let item = LoopItem::ToolResult {
            name: "ship_op".into(),
            result: "ok".into(),
        };
        let ev = translate(&item).expect("should produce event");
        assert!(matches!(ev, AgentEvent::ToolResult { name, result } if name == "ship_op" && result == "ok"));
    }

    // translate: Final -> Completed
    #[test]
    fn translate_final() {
        let item = LoopItem::Final { turns: 3 };
        let ev = translate(&item).expect("should produce event");
        assert!(matches!(ev, AgentEvent::Completed { turns: 3 }));
    }

    // translate: Internal -> None
    #[test]
    fn translate_internal_is_none() {
        let item = LoopItem::Internal;
        assert!(translate(&item).is_none());
    }

    // AgentEvent round-trips through serde
    #[test]
    fn serde_roundtrip_status() {
        let ev = AgentEvent::Status { message: "ready".into() };
        let json = serde_json::to_string(&ev).unwrap();
        let back: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_roundtrip_tool_call() {
        let ev = AgentEvent::ToolCall { name: "foo".into(), args: json!({ "x": 1 }) };
        let json = serde_json::to_string(&ev).unwrap();
        let back: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_roundtrip_completed() {
        let ev = AgentEvent::Completed { turns: 5 };
        let json = serde_json::to_string(&ev).unwrap();
        let back: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_roundtrip_error() {
        let ev = AgentEvent::Error { message: "boom".into() };
        let json = serde_json::to_string(&ev).unwrap();
        let back: AgentEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    // Internally-tagged serde: "type" field present in output
    #[test]
    fn serde_internally_tagged() {
        let ev = AgentEvent::AssistantDelta { text: "chunk".into() };
        let v: serde_json::Value = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "assistant_delta");
        assert_eq!(v["text"], "chunk");
    }

    // Sink is usable through a trait object (object-safe)
    #[test]
    fn sink_as_trait_object() {
        let sink: Box<dyn EventSink> = Box::new(CollectingSink::new());
        sink.emit(&AgentEvent::Error { message: "test".into() });
        // no panic = success
    }

    // ApprovalRequested serde round-trip — request_id must survive.
    #[test]
    fn serde_roundtrip_approval_requested() {
        let ev = AgentEvent::ApprovalRequested {
            op: "mcp.create".into(),
            args: json!({ "name": "demo" }),
            request_id: "req-123".into(),
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "approval_requested");
        assert_eq!(v["request_id"], "req-123");
        let back: AgentEvent = serde_json::from_value(v).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_roundtrip_cancelled() {
        let ev = AgentEvent::Cancelled { reason: "user-requested".into() };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "cancelled");
        let back: AgentEvent = serde_json::from_value(v).unwrap();
        assert_eq!(ev, back);
    }

    // translate: ApprovalRequested
    #[test]
    fn translate_approval_requested() {
        let item = LoopItem::ApprovalRequested {
            op: "mcp.create".into(),
            args: json!({}),
            request_id: "r1".into(),
        };
        let ev = translate(&item).expect("should produce event");
        assert!(matches!(ev, AgentEvent::ApprovalRequested { ref request_id, .. } if request_id == "r1"));
    }

    // translate: Cancelled
    #[test]
    fn translate_cancelled() {
        let item = LoopItem::Cancelled { reason: "stop".into() };
        let ev = translate(&item).expect("should produce event");
        assert!(matches!(ev, AgentEvent::Cancelled { reason } if reason == "stop"));
    }

    // UserInput semantics: Approve and Deny both have a request_id.
    #[test]
    fn user_input_variants_carry_request_id() {
        let a = UserInput::Approve { request_id: "r".into() };
        let d = UserInput::Deny { request_id: "r".into(), reason: "no".into() };
        match a {
            UserInput::Approve { request_id } => assert_eq!(request_id, "r"),
            _ => panic!("expected Approve"),
        }
        match d {
            UserInput::Deny { request_id, reason } => {
                assert_eq!(request_id, "r");
                assert_eq!(reason, "no");
            }
            _ => panic!("expected Deny"),
        }
    }

    // EventLog (S15): replays in emission order with strictly increasing seq.
    #[test]
    fn event_log_replays_in_order_with_increasing_seq() {
        let log = EventLog::new();
        log.emit(&AgentEvent::Status { message: "a".into() });
        log.emit(&AgentEvent::Status { message: "b".into() });
        let replayed = log.replay();
        assert_eq!(replayed.len(), 2);
        assert!(replayed[0].seq < replayed[1].seq, "seq must strictly increase");
        assert_eq!(replayed[0].event, AgentEvent::Status { message: "a".into() });
        assert_eq!(replayed[1].event, AgentEvent::Status { message: "b".into() });
    }

    // EventLog is an EventSink — usable through the seam, object-safe, so every
    // existing emit path can feed the durable log unchanged.
    #[test]
    fn event_log_is_an_event_sink() {
        let log: Box<dyn EventSink> = Box::new(EventLog::new());
        log.emit(&AgentEvent::Completed { turns: 1 });
        // no panic = the durable log slots in behind the trait
    }
}
