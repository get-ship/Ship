//! Tests for PiAdapter — the Pi CLI harness adapter.
//! Fixture lines follow the documented `--mode json` schema
//! (pi.dev/docs/latest/json). Live-stream verification needs a Pi install
//! + provider API key; these pin the mapping against the documented shapes.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::harness_adapter::{HarnessAdapter, HarnessEvent};
use super::PiAdapter;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── Launch commands ─────────────────────────────────────────────────────────

#[test]
fn id_and_binary_are_pi() {
    assert_eq!(PiAdapter.id(), "pi");
    assert_eq!(PiAdapter.binary(), "pi");
}

#[test]
fn interactive_sets_mesh_id_env() {
    let cmd = PiAdapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.bin, "pi");
    assert_eq!(cmd.env.get("SHIP_ACTOR_ID").unwrap(), "my-job");
}

#[test]
fn headless_runs_print_mode_json_with_prompt() {
    let profile = LaunchProfile {
        autostart_prompt: Some("do the thing".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = PiAdapter.build_headless("slug-x", &profile);
    assert_eq!(cmd.bin, "pi");
    // --print is required for non-interactive exit; --mode json alone keeps
    // the TUI running.
    assert!(cmd.args.contains(&"--print".to_string()), "missing --print: {:?}", cmd.args);
    let mode = cmd.args.iter().position(|a| a == "--mode").expect("--mode flag");
    assert_eq!(cmd.args[mode + 1], "json");
    assert_eq!(cmd.args.last().unwrap(), "do the thing");
    assert_eq!(cmd.env.get("SHIP_ACTOR_ID").map(String::as_str), Some("slug-x"));
}

#[test]
fn headless_includes_model_when_specified() {
    let profile = LaunchProfile {
        model: Some("anthropic/claude-sonnet-4".to_string()),
        autostart_prompt: Some("p".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = PiAdapter.build_headless("s", &profile);
    let m = cmd.args.iter().position(|a| a == "--model").expect("--model flag");
    assert_eq!(cmd.args[m + 1], "anthropic/claude-sonnet-4");
}

#[test]
fn headless_falls_back_to_default_prompt() {
    let cmd = PiAdapter.build_headless("s", &default_profile());
    assert_eq!(cmd.args.last().unwrap(), "Read .artifacts/job-spec.md and execute it.");
}

#[test]
fn resume_uses_session_flag() {
    let cmd = PiAdapter
        .build_resume_args("s", "019e49eb-8a84", &default_profile())
        .expect("pi supports resume");
    let sess = cmd.args.iter().position(|a| a == "--session").expect("--session flag");
    assert_eq!(cmd.args[sess + 1], "019e49eb-8a84");
    assert!(cmd.args.contains(&"--print".to_string()));
}

#[test]
fn resume_rejects_empty_session_id() {
    use crate::services::harness_adapter::ResumeError;
    let err = PiAdapter
        .build_resume_args("s", "  ", &default_profile())
        .expect_err("empty session id must error");
    matches!(err, ResumeError::BadSession { .. });
}

// ── Capabilities ─────────────────────────────────────────────────────────────

#[test]
fn caps_match_declared() {
    let c = PiAdapter.caps();
    assert!(c.headless);
    assert!(c.json_stream);
    assert!(!c.server_mode, "RPC server mode not wired yet");
    assert!(c.session_resume, "pi resumes via --session");
    assert!(c.persistent_session);
}

// ── parse_output: --mode json → HarnessEvent ─────────────────────────────────

#[test]
fn parse_session_header_starts_session() {
    let line = r#"{"type":"session","version":3,"id":"abc-123","cwd":"/x"}"#;
    match PiAdapter.parse_output(line) {
        Some(HarnessEvent::SessionStarted { session_id }) => assert_eq!(session_id, "abc-123"),
        other => panic!("expected SessionStarted, got {other:?}"),
    }
}

#[test]
fn parse_message_end_collects_text() {
    let line = r#"{"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"Hello "},{"type":"text","text":"world"}]}}"#;
    match PiAdapter.parse_output(line) {
        Some(HarnessEvent::Message { text }) => assert_eq!(text, "Hello world"),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn parse_message_update_delta_is_skipped() {
    // Streaming deltas would emit one Message per token; only message_end
    // produces a Message.
    let line = r#"{"type":"message_update","message":{"content":[]},"assistantMessageEvent":{"type":"text_delta","delta":"He"}}"#;
    assert!(PiAdapter.parse_output(line).is_none());
}

#[test]
fn parse_tool_execution_start_is_tool_use() {
    let line = r#"{"type":"tool_execution_start","toolCallId":"t1","toolName":"read_file","args":{"path":"src/main.rs"}}"#;
    match PiAdapter.parse_output(line) {
        Some(HarnessEvent::ToolUse { tool, input_preview }) => {
            assert_eq!(tool, "read_file");
            assert_eq!(input_preview, "src/main.rs");
        }
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn parse_tool_execution_end_is_dropped() {
    let line = r#"{"type":"tool_execution_end","toolCallId":"t1","toolName":"read_file","result":{},"isError":false}"#;
    assert!(PiAdapter.parse_output(line).is_none());
}

#[test]
fn parse_turn_end_completes_turn() {
    let line = r#"{"type":"turn_end","message":{},"toolResults":[]}"#;
    match PiAdapter.parse_output(line) {
        Some(HarnessEvent::TurnComplete { stop_reason }) => assert_eq!(stop_reason, "end_turn"),
        other => panic!("expected TurnComplete, got {other:?}"),
    }
}

#[test]
fn parse_lifecycle_and_non_json_are_none() {
    assert!(PiAdapter.parse_output(r#"{"type":"agent_start"}"#).is_none());
    assert!(PiAdapter.parse_output(r#"{"type":"queue_update","steering":[]}"#).is_none());
    assert!(PiAdapter.parse_output("not json").is_none());
    assert!(PiAdapter.parse_output("").is_none());
}
