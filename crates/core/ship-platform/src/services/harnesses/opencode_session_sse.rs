//! SSE frame parsing + `opencode serve` event mapping.
//!
//! Split from `opencode_session` so the session's core stays under the
//! 300-line file cap. Everything here is pure: no IO, no locks, no
//! state — given a byte buffer or a JSON frame, produce the next framing
//! decision or `SessionEvent`.

use serde_json::Value;

use crate::services::server_session::SessionEvent;

/// Locate the end of the next SSE frame (double-newline). Returns the
/// byte index **up to but not including** the separator so the caller
/// can drain the frame body, then drain the separator itself via
/// [`frame_separator_len`].
pub(super) fn find_frame_end(buffer: &str) -> Option<usize> {
    if let Some(idx) = buffer.find("\n\n") {
        return Some(idx);
    }
    buffer.find("\r\n\r\n")
}

/// Length of the SSE frame separator at the start of `buffer`, whichever
/// form the peer is using. Callers invoke this immediately after draining
/// a frame body so the separator is at the front.
pub(super) fn frame_separator_len(buffer: &str) -> usize {
    if buffer.starts_with("\r\n\r\n") {
        4
    } else if buffer.starts_with("\n\n") {
        2
    } else {
        0
    }
}

/// Parse a single SSE frame (text up to but not including the trailing
/// blank line) into a `(event_type, properties)` tuple. Returns `None` if
/// the frame has no `data:` line or the payload isn't JSON with a `type`.
pub(super) fn parse_sse_frame(frame: &str) -> Option<(String, Value)> {
    let mut data = String::new();
    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(rest.strip_prefix(' ').unwrap_or(rest));
        }
    }
    if data.is_empty() {
        return None;
    }
    let payload: Value = serde_json::from_str(&data).ok()?;
    let event_type = payload.get("type")?.as_str()?.to_string();
    let props = payload.get("properties").cloned().unwrap_or_default();
    Some((event_type, props))
}

/// Map an `opencode serve` SSE event `type` + `properties` payload to a
/// harness-agnostic `SessionEvent`. Unknown types are silently ignored
/// so new opencode event names don't break the loop.
pub(super) fn map_sse_event_to_session_event(
    event_type: &str,
    props: &Value,
) -> Option<SessionEvent> {
    match event_type {
        "assistant" => {
            let text = props
                .get("text")
                .or_else(|| props.get("delta"))
                .and_then(|v| v.as_str())?
                .to_string();
            Some(SessionEvent::AgentMessage { text })
        }
        "tool_call" => {
            let tool = props.get("tool").and_then(|v| v.as_str())?.to_string();
            let input_preview = props
                .get("input")
                .map(|v| v.to_string())
                .unwrap_or_default();
            Some(SessionEvent::ToolUse {
                tool,
                input_preview,
            })
        }
        "tool_result" => {
            let line = props
                .get("output")
                .or_else(|| props.get("text"))
                .and_then(|v| v.as_str())?
                .to_string();
            Some(SessionEvent::CommandOutput {
                stream: "stdout".to_string(),
                line,
            })
        }
        "session_completed" => Some(SessionEvent::TurnCompleted {
            stop_reason: "completed".to_string(),
        }),
        "session_error" => {
            let message = props
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("opencode session error")
                .to_string();
            Some(SessionEvent::Error { message })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_frame_extracts_type_and_properties() {
        let frame = "data: {\"type\":\"assistant\",\"properties\":{\"text\":\"hi\"}}";
        let (ty, props) = parse_sse_frame(frame).expect("parsed");
        assert_eq!(ty, "assistant");
        assert_eq!(props["text"], "hi");
    }

    #[test]
    fn unknown_event_type_is_ignored() {
        assert!(map_sse_event_to_session_event("new-event", &json!({})).is_none());
    }
}
