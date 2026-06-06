//! Tests for CursorAdapter — the Cursor CLI harness adapter.

use ship_substrate::services::dispatch_ports::LaunchProfile;
use ship_runner::harness_adapter::{HarnessAdapter, HarnessEvent};
use super::CursorAdapter;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── Launch command tests ────────────────────────────────────────────────────

#[test]
fn interactive_emits_cursor_agent_binary() {
    let adapter = CursorAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.bin, "cursor-agent");
}

#[test]
fn interactive_sets_mesh_id_env() {
    let adapter = CursorAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.env.get("SHIP_ACTOR_ID").unwrap(), "my-job");
}

#[test]
fn interactive_model_flag_maps_correctly() {
    let adapter = CursorAdapter;
    let profile = LaunchProfile {
        model: Some("gpt-5".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--model").unwrap();
    assert_eq!(cmd.args[idx + 1], "gpt-5");
}

#[test]
fn headless_emits_print_force_command() {
    let adapter = CursorAdapter;
    let profile = LaunchProfile {
        autostart_prompt: Some("test prompt".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_headless("test-slug", &profile);
    assert_eq!(cmd.bin, "cursor-agent");
    assert!(cmd.args.contains(&"-p".to_string()));
    assert!(cmd.args.contains(&"--force".to_string()));
    // stream-json is what makes parse_output see anything.
    let fmt = cmd.args.iter().position(|a| a == "--output-format").expect("--output-format");
    assert_eq!(cmd.args[fmt + 1], "stream-json");
    assert_eq!(cmd.args.last().unwrap(), "test prompt");
    assert_eq!(
        cmd.env.get("SHIP_ACTOR_ID").map(String::as_str),
        Some("test-slug")
    );
}

#[test]
fn headless_includes_model_when_specified() {
    let adapter = CursorAdapter;
    let profile = LaunchProfile {
        model: Some("gpt-5".to_string()),
        autostart_prompt: Some("p".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_headless("s", &profile);
    let model_idx = cmd
        .args
        .iter()
        .position(|a| a == "--model")
        .expect("--model flag");
    assert_eq!(cmd.args[model_idx + 1], "gpt-5");
}

#[test]
fn headless_falls_back_to_default_prompt() {
    let adapter = CursorAdapter;
    let cmd = adapter.build_headless("s", &default_profile());
    assert_eq!(
        cmd.args.last().unwrap(),
        "Read .artifacts/job-spec.md and execute it."
    );
}

// ── Adapter identity ────────────────────────────────────────────────────────

#[test]
fn id_is_cursor() {
    assert_eq!(CursorAdapter.id(), "cursor");
}

// ── parse_output: stream-json → HarnessEvent ─────────────────────────────────
// Fixture lines follow the documented --output-format stream-json schema
// (cursor.com/docs/cli/headless). Live-stream verification needs `agent
// login`; these pin the mapping against the documented shapes.

#[test]
fn parse_assistant_message_collects_text() {
    let line = r#"{"type":"assistant","timestamp_ms":1,"message":{"content":[{"text":"Hello "},{"text":"world"}]}}"#;
    match CursorAdapter.parse_output(line) {
        Some(HarnessEvent::Message { text }) => assert_eq!(text, "Hello world"),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn parse_read_tool_call_started() {
    let line = r#"{"type":"tool_call","subtype":"started","tool_call":{"readToolCall":{"args":{"path":"src/main.rs"}}}}"#;
    match CursorAdapter.parse_output(line) {
        Some(HarnessEvent::ToolUse { tool, input_preview }) => {
            assert_eq!(tool, "read");
            assert_eq!(input_preview, "src/main.rs");
        }
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn parse_write_tool_call_started() {
    let line = r#"{"type":"tool_call","subtype":"started","tool_call":{"writeToolCall":{"args":{"path":"hi.txt"}}}}"#;
    match CursorAdapter.parse_output(line) {
        Some(HarnessEvent::ToolUse { tool, input_preview }) => {
            assert_eq!(tool, "write");
            assert_eq!(input_preview, "hi.txt");
        }
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn parse_tool_call_completed_is_dropped() {
    // `completed` events would double-count the tool use; only `started`
    // produces a ToolUse.
    let line = r#"{"type":"tool_call","subtype":"completed","tool_call":{"readToolCall":{"result":{"success":{"totalLines":10}}}}}"#;
    assert!(CursorAdapter.parse_output(line).is_none());
}

#[test]
fn parse_result_ends_turn() {
    let line = r#"{"type":"result","duration_ms":1234}"#;
    match CursorAdapter.parse_output(line) {
        Some(HarnessEvent::TurnComplete { stop_reason }) => assert_eq!(stop_reason, "end_turn"),
        other => panic!("expected TurnComplete, got {other:?}"),
    }
}

#[test]
fn parse_result_error_flags_stop_reason() {
    let line = r#"{"type":"result","duration_ms":5,"is_error":true}"#;
    match CursorAdapter.parse_output(line) {
        Some(HarnessEvent::TurnComplete { stop_reason }) => assert_eq!(stop_reason, "error"),
        other => panic!("expected TurnComplete, got {other:?}"),
    }
}

#[test]
fn parse_system_init_yields_nothing() {
    // model is present but there's no session id; cursor emits no
    // SessionStarted (no resume).
    let line = r#"{"type":"system","subtype":"init","model":"sonnet-4"}"#;
    assert!(CursorAdapter.parse_output(line).is_none());
}

#[test]
fn parse_non_json_or_unknown_is_none() {
    assert!(CursorAdapter.parse_output("not json").is_none());
    assert!(CursorAdapter.parse_output("").is_none());
    assert!(CursorAdapter.parse_output(r#"{"type":"user","message":{}}"#).is_none());
}

// ── Capabilities ────────────────────────────────────────────────────────────

#[test]
fn caps_match_declared() {
    let c = CursorAdapter.caps();
    assert!(c.headless);
    assert!(c.interactive);
    assert!(!c.server_mode);
    assert!(c.json_stream, "cursor: --output-format stream-json");
    assert!(!c.mcp, "cursor: no documented MCP injection");
    assert!(c.hooks, "cursor: .cursor/hooks.json");
    assert!(!c.session_resume);
}

#[test]
fn build_resume_args_returns_unsupported() {
    use ship_runner::harness_adapter::ResumeError;
    let adapter = CursorAdapter;
    let err = adapter
        .build_resume_args("my-job", "any-id", &default_profile())
        .expect_err("cursor has no resume mechanism");
    match err {
        ResumeError::Unsupported { harness } => assert_eq!(harness, "cursor"),
        other => panic!("expected Unsupported, got {other:?}"),
    }
}
