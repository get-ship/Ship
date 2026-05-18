//! Tests for ClaudeAdapter — the Claude Code CLI harness adapter.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::harness_adapter::{HarnessAdapter, HarnessEvent};
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
    let idx = cmd.args.iter().position(|a| a == "--channels").unwrap();
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
    assert_eq!(cmd.env.get("SHIP_ACTOR_ID").unwrap(), "my-job");
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
    assert!(cmd.args.contains(&"--channels".to_string()));
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
fn parse_system_init_extracts_session_id() {
    let adapter = ClaudeAdapter;
    // Current stream-json format: a `system` event with `subtype: init`
    // carries the session id (replaces the legacy `stream_event/message_start`
    // shape; see commit 8158d5a0).
    let line = r#"{"type":"system","subtype":"init","session_id":"sess-xyz-789"}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::SessionStarted { session_id }) => {
            assert_eq!(session_id, "sess-xyz-789");
        }
        other => panic!("expected SessionStarted, got {other:?}"),
    }
}

#[test]
fn parse_tool_use_extracts_tool_name() {
    let adapter = ClaudeAdapter;
    // Current format: tool_use blocks come inside an `assistant` envelope,
    // not the legacy `stream_event/content_block_start` wrapper.
    let line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"tu_1","name":"Read","input":{}}]}}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::ToolUse { tool, .. }) => {
            assert_eq!(tool, "Read");
        }
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn parse_system_event_becomes_custom() {
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"system","subtype":"api_retry","attempt":1,"max_retries":3,"retry_delay_ms":1000,"error_status":429,"error":"rate_limit","uuid":"u5","session_id":"s1"}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::Custom { kind, .. }) => {
            assert_eq!(kind, "api_retry");
        }
        other => panic!("expected Custom, got {other:?}"),
    }
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
fn parse_unknown_top_level_type_returns_none() {
    // Modern stream-json uses top-level `assistant`/`result`/`system` events.
    // The wrapping `stream_event` shape is gone — anything that still emits it
    // should be ignored rather than dispatched.
    let adapter = ClaudeAdapter;
    let line = r#"{"type":"stream_event","event":{"type":"content_block_delta"}}"#;
    assert!(adapter.parse_output(line).is_none());
}

// ── Shell string ────────────────────────────────────────────────────────────

#[test]
fn to_shell_string_includes_env_and_flags() {
    let adapter = ClaudeAdapter;
    let cmd = adapter.build_interactive("test-slug", &default_profile());
    let shell = cmd.to_shell_string();
    assert!(shell.contains("SHIP_ACTOR_ID=test-slug"));
    assert!(shell.contains("claude"));
    assert!(shell.contains("--dangerously-skip-permissions"));
    assert!(shell.contains("--channels"));
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
    assert!(c.session_resume, "claude: --resume <uuid>");
}

// ── build_resume_args ───────────────────────────────────────────────────────

#[test]
fn build_resume_args_emits_resume_flag() {
    let adapter = ClaudeAdapter;
    let cmd = adapter
        .build_resume_args("my-job", "sess-abc-123", &default_profile())
        .expect("claude supports resume");
    assert_eq!(cmd.bin, "claude");
    let idx = cmd.args.iter().position(|a| a == "--resume").unwrap();
    assert_eq!(cmd.args[idx + 1], "sess-abc-123");
}

#[test]
fn build_resume_args_includes_fork_session() {
    // Resumed runs must include --fork-session so the new turn's MCP
    // config (--mcp-config inline) is re-bound. See base_args comment.
    let adapter = ClaudeAdapter;
    let cmd = adapter
        .build_resume_args("my-job", "sess-abc-123", &default_profile())
        .expect("claude supports resume");
    assert!(cmd.args.contains(&"--fork-session".to_string()));
}

#[test]
fn build_resume_args_rejects_empty_session_id() {
    use crate::services::harness_adapter::ResumeError;
    let adapter = ClaudeAdapter;
    let err = adapter
        .build_resume_args("my-job", "", &default_profile())
        .expect_err("empty session id must be rejected");
    match err {
        ResumeError::BadSession { harness, reason } => {
            assert_eq!(harness, "claude");
            assert!(reason.contains("empty"));
        }
        other => panic!("expected BadSession, got {other:?}"),
    }
}
