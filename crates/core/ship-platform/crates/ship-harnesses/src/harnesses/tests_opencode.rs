//! Tests for OpenCodeAdapter — the OpenCode CLI harness adapter.

use agent_driver::dispatch_ports::LaunchProfile;
use agent_driver::AgentDriver;
use super::OpenCodeAdapter;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── Launch command tests ────────────────────────────────────────────────────

#[test]
fn interactive_emits_opencode_binary() {
    let adapter = OpenCodeAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.bin, "opencode");
}

#[test]
fn interactive_sets_mesh_id_env() {
    let adapter = OpenCodeAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.env.get("SHIP_ACTOR_ID").unwrap(), "my-job");
}

#[test]
fn interactive_model_flag_maps_correctly() {
    let adapter = OpenCodeAdapter;
    let profile = LaunchProfile {
        model: Some("anthropic/claude-sonnet-4".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--model").unwrap();
    assert_eq!(cmd.args[idx + 1], "anthropic/claude-sonnet-4");
}

#[test]
fn headless_emits_run_command_with_prompt() {
    let adapter = OpenCodeAdapter;
    let profile = LaunchProfile {
        autostart_prompt: Some("test prompt".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_headless("test-slug", &profile);
    assert_eq!(cmd.bin, "opencode");
    assert_eq!(cmd.args[0], "run");
    assert!(
        !cmd.args.contains(&"--yolo".to_string()),
        "opencode CLI has no --yolo flag; removed in April 2026",
    );
    assert!(
        cmd.args.contains(&"--dangerously-skip-permissions".to_string()),
        "opencode 0.x has --dangerously-skip-permissions for auto-approve",
    );
    assert!(
        cmd.args.contains(&"--format".to_string())
            && cmd.args.contains(&"json".to_string()),
        "opencode headless uses --format json for structured stream",
    );
    assert_eq!(cmd.args.last().unwrap(), "test prompt");
    assert_eq!(
        cmd.env.get("SHIP_ACTOR_ID").map(String::as_str),
        Some("test-slug")
    );
}

#[test]
fn headless_includes_model_when_specified() {
    let adapter = OpenCodeAdapter;
    let profile = LaunchProfile {
        model: Some("anthropic/claude-sonnet-4".to_string()),
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
    assert_eq!(cmd.args[model_idx + 1], "anthropic/claude-sonnet-4");
}

#[test]
fn headless_falls_back_to_default_prompt() {
    let adapter = OpenCodeAdapter;
    let cmd = adapter.build_headless("s", &default_profile());
    assert_eq!(
        cmd.args.last().unwrap(),
        "Read .artifacts/job-spec.md and execute it."
    );
}

// ── Adapter identity ────────────────────────────────────────────────────────

#[test]
fn id_is_opencode() {
    assert_eq!(OpenCodeAdapter.id(), "opencode");
}

#[test]
fn parse_output_returns_none_for_garbage() {
    let adapter = OpenCodeAdapter;
    assert!(adapter.parse_output("anything").is_none());
    assert!(adapter.parse_output("{}").is_none());
}

#[test]
fn parse_output_captures_session_id_from_step_start() {
    use agent_driver::HarnessEvent;
    let adapter = OpenCodeAdapter;
    let line = r#"{"type":"step_start","sessionID":"ses_abc123","part":{"id":"prt_x","type":"step-start"}}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::SessionStarted { session_id }) => {
            assert_eq!(session_id, "ses_abc123");
        }
        other => panic!("expected SessionStarted, got {other:?}"),
    }
}

#[test]
fn parse_output_emits_message_for_text() {
    use agent_driver::HarnessEvent;
    let adapter = OpenCodeAdapter;
    let line = r#"{"type":"text","sessionID":"ses_x","part":{"type":"text","text":"hello"}}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::Message { text }) => assert_eq!(text, "hello"),
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn parse_output_emits_turn_usage_from_step_finish() {
    use agent_driver::HarnessEvent;
    let adapter = OpenCodeAdapter;
    let line = r#"{"type":"step_finish","sessionID":"ses_x","part":{"type":"step-finish","reason":"stop","tokens":{"total":19596,"input":100,"output":73,"reasoning":95,"cache":{"write":12,"read":19328}}}}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::TurnUsage {
            input_tokens,
            output_tokens,
            reasoning_tokens,
            cache_creation_tokens,
            cache_read_tokens,
            stop_reason,
            ..
        }) => {
            assert_eq!(input_tokens, 100);
            assert_eq!(output_tokens, 73);
            assert_eq!(reasoning_tokens, 95);
            assert_eq!(cache_creation_tokens, 12);
            assert_eq!(cache_read_tokens, 19328);
            assert_eq!(stop_reason.as_deref(), Some("stop"));
        }
        other => panic!("expected TurnUsage, got {other:?}"),
    }
}

#[test]
fn parse_output_emits_tool_use_for_tool_part() {
    use agent_driver::HarnessEvent;
    let adapter = OpenCodeAdapter;
    let line = r#"{"type":"tool_use","sessionID":"ses_x","part":{"type":"tool","tool":"Bash","state":{"status":"completed","input":{"command":"ls"}}}}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::ToolUse { tool, .. }) => assert_eq!(tool, "Bash"),
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

// ── Capabilities ────────────────────────────────────────────────────────────

#[test]
fn caps_match_declared() {
    let c = OpenCodeAdapter.caps();
    assert!(c.headless);
    assert!(c.interactive);
    assert!(
        !c.server_mode,
        "opencode server_mode is off pending OpenCodeSession rewrite — API shape changed (parts vs role/content, sync vs SSE)",
    );
    assert!(c.json_stream);
    assert!(c.mcp, "opencode: opencode.json mcp");
    assert!(!c.hooks, "opencode: hooks via JS plugin file, not wired");
    assert!(c.session_resume, "opencode: REST sessions persist");
}

// ── build_resume_args ───────────────────────────────────────────────────────

#[test]
fn build_resume_args_emits_session_flag() {
    let adapter = OpenCodeAdapter;
    let cmd = adapter
        .build_resume_args("my-job", "sess-uuid-9", &default_profile())
        .expect("opencode supports resume");
    assert_eq!(cmd.bin, "opencode");
    assert_eq!(cmd.args[0], "run");
    let idx = cmd.args.iter().position(|a| a == "--session").unwrap();
    assert_eq!(cmd.args[idx + 1], "sess-uuid-9");
}

#[test]
fn build_resume_args_rejects_empty_session_id() {
    use agent_driver::ResumeError;
    let adapter = OpenCodeAdapter;
    let err = adapter
        .build_resume_args("my-job", "", &default_profile())
        .expect_err("empty session id must be rejected");
    assert!(matches!(err, ResumeError::BadSession { .. }));
}
