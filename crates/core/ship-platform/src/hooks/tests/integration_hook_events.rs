/// Integration tests — hook event handler pipeline.
///
/// Each test exercises the public handlers as a caller would:
/// build the JSON payload, call the handler, assert the returned EventEnvelope.
/// No subprocess is spawned; no event store is written — the handlers
/// return envelopes rather than appending to the kernel.
///
/// Unit tests in `hooks_tests.rs` already cover the basic happy-path per
/// handler. These integration tests add:
///   - Full context wiring (actor_id AND workspace_id on every handler)
///   - Session lifecycle sequence (start → end consistency)
///   - Tool lifecycle sequence (before → after skill.started → skill.completed)
///   - tool_name prefix stripping into entity_id
///   - Cursor camelCase schema recognised by `handle_before_tool`
///   - Error response inside `handle_after_tool` → skill.failed with message
use ship_substrate::events::types::event_types;

use crate::hooks::{handle_after_tool, handle_before_tool, handle_session_end, handle_session_start};

// ── Session lifecycle ─────────────────────────────────────────────────────────

#[test]
fn hooks_hook_events_session_start_actor_and_workspace_wired() {
    let event = handle_session_start("actor-a", "ws-a")
        .expect("handle_session_start must succeed");
    assert_eq!(event.event_type, event_types::SESSION_STARTED);
    assert_eq!(event.actor_id.as_deref(), Some("actor-a"),
        "actor_id must match the actor argument");
    assert_eq!(event.space_id.as_deref(), Some("ws-a"),
        "space_id must match the workspace_id argument");
}

#[test]
fn hooks_hook_events_session_end_actor_and_workspace_wired() {
    let event = handle_session_end("actor-b", "ws-b")
        .expect("handle_session_end must succeed");
    assert_eq!(event.event_type, event_types::SESSION_ENDED);
    assert_eq!(event.actor_id.as_deref(), Some("actor-b"));
    assert_eq!(event.space_id.as_deref(), Some("ws-b"));
}

#[test]
fn hooks_hook_events_session_lifecycle_event_types_differ() {
    let start = handle_session_start("actor-c", "ws-c").unwrap();
    let end   = handle_session_end("actor-c", "ws-c").unwrap();
    assert_ne!(
        start.event_type, end.event_type,
        "session.started and session.ended must be distinct event types"
    );
    assert_eq!(start.event_type, event_types::SESSION_STARTED);
    assert_eq!(end.event_type,   event_types::SESSION_ENDED);
}

#[test]
fn hooks_hook_events_session_start_workspace_survives_different_actors() {
    // Verify that the workspace_id arg is threaded correctly regardless of actor.
    let e1 = handle_session_start("actor-x", "shared-ws").unwrap();
    let e2 = handle_session_start("actor-y", "shared-ws").unwrap();
    assert_eq!(e1.space_id, e2.space_id,
        "both actors in the same workspace should produce the same space_id");
    assert_ne!(e1.actor_id, e2.actor_id,
        "different actors must produce different actor_ids");
}

// ── Tool lifecycle ────────────────────────────────────────────────────────────

#[test]
fn hooks_hook_events_tool_lifecycle_before_then_after_ship_tool() {
    let before_json = r#"{"tool_name":"mcp__ship__deploy","hook_event_name":"PreToolUse"}"#;
    let after_json  = r#"{"tool_name":"mcp__ship__deploy","hook_event_name":"PostToolUse"}"#;

    let before_env = handle_before_tool(before_json, "act", "ws")
        .expect("before_tool must succeed")
        .expect("ship tool must produce an event");
    let after_env = handle_after_tool(after_json, "act", "ws")
        .expect("after_tool must succeed")
        .expect("ship tool must produce an event");

    assert_eq!(before_env.event_type, event_types::SKILL_STARTED);
    assert_eq!(after_env.event_type,  event_types::SKILL_COMPLETED);
    // Both envelopes are for the same skill — entity_id should match.
    assert_eq!(before_env.entity_id, after_env.entity_id,
        "before and after envelopes must share the same entity_id (skill_id)");
}

#[test]
fn hooks_hook_events_tool_lifecycle_after_with_error_emits_skill_failed() {
    let after_json = r#"{
        "tool_name": "mcp__ship__build",
        "hook_event_name": "PostToolUse",
        "tool_response": {"error": "cargo build failed"}
    }"#;
    let env = handle_after_tool(after_json, "act", "ws")
        .unwrap()
        .expect("must produce an event");
    assert_eq!(env.event_type, event_types::SKILL_FAILED);
}

#[test]
fn hooks_hook_events_before_tool_strips_prefix_into_entity_id() {
    // entity_id on the envelope must be just the skill name, not the full mcp__ form.
    let json = r#"{"tool_name":"mcp__ship__commit","hook_event_name":"PreToolUse"}"#;
    let env = handle_before_tool(json, "act", "ws").unwrap().unwrap();
    assert_eq!(env.entity_id, "commit",
        "entity_id must be the stripped skill name, not the full mcp__ship__commit string");
}

#[test]
fn hooks_hook_events_before_tool_non_ship_returns_none() {
    let json = r#"{"tool_name":"Bash","tool_input":{"command":"echo hi"}}"#;
    let result = handle_before_tool(json, "act", "ws").unwrap();
    assert!(result.is_none(),
        "non-ship tools must return None from handle_before_tool");
}

#[test]
fn hooks_hook_events_after_tool_non_ship_returns_none() {
    let json = r#"{"tool_name":"Write","tool_input":{"file_path":"/tmp/x"}}"#;
    let result = handle_after_tool(json, "act", "ws").unwrap();
    assert!(result.is_none(),
        "non-ship tools must return None from handle_after_tool");
}

#[test]
fn hooks_hook_events_before_tool_actor_and_workspace_wired() {
    let json = r#"{"tool_name":"mcp__ship__sync","hook_event_name":"PreToolUse"}"#;
    let env = handle_before_tool(json, "my-actor", "my-ws").unwrap().unwrap();
    assert_eq!(env.actor_id.as_deref(), Some("my-actor"));
    assert_eq!(env.space_id.as_deref(), Some("my-ws"));
}

#[test]
fn hooks_hook_events_cursor_camelcase_tool_name_returns_err() {
    // ProviderHook requires "tool_name" (snake_case). Cursor's camelCase
    // "toolName" does not deserialize into it — the struct's required field is
    // missing and serde returns an Err. This is a known limitation: the cursor
    // schema is only fully handled by tool_gate::check (explicit .or_else
    // fallbacks), not by the ProviderHook envelope builder.
    let json = r#"{"toolName":"mcp__ship__build","hookEventName":"PreToolUse"}"#;
    let result = handle_before_tool(json, "a", "w");
    assert!(result.is_err(),
        "cursor camelCase toolName fails ProviderHook deserialization — must return Err");
}

#[test]
fn hooks_hook_events_before_tool_rejects_empty_stdin() {
    let err = handle_before_tool("", "a", "w");
    assert!(err.is_err(), "empty stdin must be rejected with Err");
}

#[test]
fn hooks_hook_events_after_tool_rejects_empty_stdin() {
    let err = handle_after_tool("", "a", "w");
    assert!(err.is_err(), "empty stdin must be rejected with Err");
}
