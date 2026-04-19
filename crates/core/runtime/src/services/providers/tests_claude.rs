//! Tests for ClaudeAdapter — the Claude Code CLI provider adapter.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::provider_adapter::{ProviderAdapter, ProviderEvent};
use super::ClaudeAdapter;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── Launch command tests ────────────────────────────────────────────────────

#[test]
fn interactive_emits_claude_binary() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.bin, "claude");
}

#[test]
fn interactive_skips_permissions() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert!(cmd.args.contains(&"--dangerously-skip-permissions".to_string()));
}

#[test]
fn interactive_loads_ship_channel() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    let idx = cmd.args.iter().position(|a| a == "--dangerously-load-development-channels").unwrap();
    assert_eq!(cmd.args[idx + 1], "server:ship");
}

#[test]
fn interactive_sets_session_name_from_slug() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("fix-auth", &default_profile());
    let idx = cmd.args.iter().position(|a| a == "--name").unwrap();
    assert_eq!(cmd.args[idx + 1], "job-fix-auth");
}

#[test]
fn interactive_custom_session_name_overrides_slug() {
    let adapter = ClaudeAdapter;
    let profile = LaunchProfile {
        session_name: Some("custom-session".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("my-job", &profile);
    let idx = cmd.args.iter().position(|a| a == "--name").unwrap();
    assert_eq!(cmd.args[idx + 1], "custom-session");
}

#[test]
fn interactive_does_not_include_print_flag() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert!(!cmd.args.contains(&"--print".to_string()));
}

#[test]
fn interactive_sets_mesh_id_env() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.env.get("SHIP_MESH_ID").unwrap(), "my-job");
}

#[test]
fn headless_includes_print_and_prompt() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_headless("my-job", &default_profile());
    assert!(cmd.args.contains(&"--print".to_string()));
    assert!(cmd.args.contains(&"-p".to_string()));
    assert!(cmd.args.contains(&"go".to_string()));
}

#[test]
fn headless_always_uses_stream_json() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_headless("my-job", &default_profile());
    let idx = cmd.args.iter().position(|a| a == "--output-format").unwrap();
    assert_eq!(cmd.args[idx + 1], "stream-json");
}

#[test]
fn headless_also_loads_ship_channel() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_headless("my-job", &default_profile());
    assert!(cmd.args.contains(&"--dangerously-load-development-channels".to_string()));
}

// ── LaunchProfile flag mapping ──────────────────────────────────────────────

#[test]
fn model_flag_maps_correctly() {
    let adapter = ClaudeAdapter;
    let profile = LaunchProfile {
        model: Some("opus".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--model").unwrap();
    assert_eq!(cmd.args[idx + 1], "opus");
}

#[test]
fn effort_flag_maps_correctly() {
    let adapter = ClaudeAdapter;
    let profile = LaunchProfile {
        effort: Some("high".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--effort").unwrap();
    assert_eq!(cmd.args[idx + 1], "high");
}

#[test]
fn budget_flag_maps_correctly() {
    let adapter = ClaudeAdapter;
    let profile = LaunchProfile {
        max_budget_usd: Some(10.5),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--max-budget-usd").unwrap();
    assert_eq!(cmd.args[idx + 1], "10.5");
}

#[test]
fn permission_mode_maps_correctly() {
    let adapter = ClaudeAdapter;
    let profile = LaunchProfile {
        permission_mode: Some("plan".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--permission-mode").unwrap();
    assert_eq!(cmd.args[idx + 1], "plan");
}

#[test]
fn resume_session_maps_correctly() {
    let adapter = ClaudeAdapter;
    let profile = LaunchProfile {
        resume_session: Some("sess-abc-123".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--resume").unwrap();
    assert_eq!(cmd.args[idx + 1], "sess-abc-123");
}

#[test]
fn ephemeral_session_includes_no_persistence_flag() {
    let adapter = ClaudeAdapter;
    let profile = LaunchProfile {
        persist_session: false,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    assert!(cmd.args.contains(&"--no-session-persistence".to_string()));
}

#[test]
fn persistent_session_omits_no_persistence_flag() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("j", &default_profile());
    assert!(!cmd.args.contains(&"--no-session-persistence".to_string()));
}

// ── Lifecycle hooks ─────────────────────────────────────────────────────────

#[test]
fn ready_pattern_is_prompt_mode_marker() {
    // Must NOT be "❯" alone — that matches the channels-menu selector and
    // fires autostart into a menu that hasn't been dismissed. Instead use
    // the mode-marker line that only appears at the real input prompt.
    let adapter = ClaudeAdapter;
    assert_eq!(adapter.ready_pattern(), "bypass permissions on");
}

#[test]
fn autostart_input_reads_job_spec() {
    let adapter = ClaudeAdapter;
    assert_eq!(
        adapter.autostart_input(),
        "Read .artifacts/job-spec.md and execute it autonomously."
    );
}

#[test]
fn exit_command_is_slash_exit() {
    let adapter = ClaudeAdapter;
    assert_eq!(adapter.exit_command(), "/exit");
}

// ── Output parsing (stream-json) ────────────────────────────────────────────

#[test]
fn parse_message_start_extracts_session_id() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"message_start","message":{"id":"msg_1","role":"assistant","content":[],"model":"claude-sonnet-4-6","usage":{"input_tokens":100,"output_tokens":0}}},"uuid":"u1","session_id":"sess-xyz-789"}"#;
    match adapter.parse_output(line) {
        Some(ProviderEvent::SessionStarted { session_id }) => {
            assert_eq!(session_id, "sess-xyz-789");
        }
        other => panic!("expected SessionStarted, got {other:?}"),
    }
}

#[test]
fn parse_tool_use_extracts_tool_name() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"tu_1","name":"Read","input":{}}},"uuid":"u2","session_id":"s1"}"#;
    match adapter.parse_output(line) {
        Some(ProviderEvent::ToolUse { tool, .. }) => {
            assert_eq!(tool, "Read");
        }
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn parse_text_delta_extracts_text() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello world"}},"uuid":"u3","session_id":"s1"}"#;
    match adapter.parse_output(line) {
        Some(ProviderEvent::Message { text }) => {
            assert_eq!(text, "Hello world");
        }
        other => panic!("expected Message, got {other:?}"),
    }
}

#[test]
fn parse_message_delta_extracts_usage() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":147}},"uuid":"u4","session_id":"s1"}"#;
    match adapter.parse_output(line) {
        Some(ProviderEvent::Usage { output_tokens, .. }) => {
            assert_eq!(output_tokens, 147);
        }
        other => panic!("expected Usage, got {other:?}"),
    }
}

#[test]
fn parse_system_event_becomes_custom() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"system","subtype":"api_retry","attempt":1,"max_retries":3,"retry_delay_ms":1000,"error_status":429,"error":"rate_limit","uuid":"u5","session_id":"s1"}"#;
    match adapter.parse_output(line) {
        Some(ProviderEvent::Custom { kind, .. }) => {
            assert_eq!(kind, "api_retry");
        }
        other => panic!("expected Custom, got {other:?}"),
    }
}

#[test]
fn parse_content_block_stop_returns_none() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"content_block_stop","index":0},"uuid":"u6","session_id":"s1"}"#;
    assert!(adapter.parse_output(line).is_none());
}

#[test]
fn parse_message_stop_returns_none() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"message_stop"},"uuid":"u7","session_id":"s1"}"#;
    assert!(adapter.parse_output(line).is_none());
}

#[test]
fn parse_malformed_json_returns_none() {
    let adapter = ClaudeAdapter;
    assert!(adapter.parse_output("not json at all {{{").is_none());
}

#[test]
fn parse_empty_line_returns_none() {
    let adapter = ClaudeAdapter;
    assert!(adapter.parse_output("").is_none());
}

#[test]
fn parse_text_block_start_returns_none() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}},"uuid":"u8","session_id":"s1"}"#;
    assert!(adapter.parse_output(line).is_none());
}

#[test]
fn parse_input_json_delta_returns_none() {
    // Tool input deltas are partial JSON — not useful as standalone events.
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"path\":"}},"uuid":"u9","session_id":"s1"}"#;
    assert!(adapter.parse_output(line).is_none());
}

// ── Shell string ────────────────────────────────────────────────────────────

#[test]
fn to_shell_string_includes_env_and_flags() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("test-slug", &default_profile());
    let shell = cmd.to_shell_string();
    assert!(shell.contains("SHIP_MESH_ID=test-slug"));
    assert!(shell.contains("claude"));
    assert!(shell.contains("--dangerously-skip-permissions"));
    assert!(shell.contains("--dangerously-load-development-channels"));
}

// ── Adapter identity ────────────────────────────────────────────────────────

#[test]
fn id_is_claude() {
    assert_eq!(ClaudeAdapter.id(), "claude");
}

// ── Capabilities ────────────────────────────────────────────────────────────

#[test]
fn caps_match_declared() {
    let c = ClaudeAdapter.caps();
    assert!(c.headless);
    assert!(c.interactive);
    assert!(!c.server_mode, "claude: SDK is in-process, no server");
    assert!(c.json_stream, "claude: --output-format stream-json");
    assert!(c.mcp, "claude: .mcp.json");
    assert!(c.hooks, "claude: .claude/settings.json hooks");
    assert!(!c.session_resume);
}
