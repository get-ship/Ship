//! Tests for CodexAdapter — the Codex CLI harness adapter.

use agent_driver::dispatch_ports::LaunchProfile;
use agent_driver::AgentDriver;
use super::CodexAdapter;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── Launch command tests ────────────────────────────────────────────────────

#[test]
fn interactive_emits_codex_binary() {
    let adapter = CodexAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.bin, "codex");
}

#[test]
fn interactive_does_not_include_claude_flags() {
    let adapter = CodexAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert!(!cmd.args.contains(&"--dangerously-skip-permissions".to_string()));
    assert!(!cmd.args.contains(&"--dangerously-load-development-channels".to_string()));
    assert!(!cmd.args.contains(&"--print".to_string()));
}

#[test]
fn interactive_sets_mesh_id_env() {
    let adapter = CodexAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.env.get("SHIP_ACTOR_ID").unwrap(), "my-job");
}

#[test]
fn model_flag_maps_correctly() {
    let adapter = CodexAdapter;
    let profile = LaunchProfile {
        model: Some("o3".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--model").unwrap();
    assert_eq!(cmd.args[idx + 1], "o3");
}

#[test]
fn headless_emits_exec_subcommand() {
    let adapter = CodexAdapter;
    let profile = LaunchProfile {
        autostart_prompt: Some("test prompt".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_headless("test-slug", &profile);
    assert_eq!(cmd.bin, "codex");
    assert_eq!(cmd.args[0], "exec");
    assert!(cmd.args.contains(&"--sandbox".to_string()));
    assert!(cmd.args.contains(&"workspace-write".to_string()));
    // Guard against regressing to the deprecated flag.
    assert!(!cmd.args.contains(&"--full-auto".to_string()));
    assert!(cmd.args.contains(&"--skip-git-repo-check".to_string()));
    assert!(cmd.args.contains(&"--json".to_string()));
    assert!(cmd.args.contains(&"--ephemeral".to_string()));
    assert_eq!(cmd.args.last().unwrap(), "test prompt");
    assert_eq!(
        cmd.env.get("SHIP_ACTOR_ID").map(String::as_str),
        Some("test-slug")
    );
}

#[test]
fn headless_includes_model_when_specified() {
    let adapter = CodexAdapter;
    let profile = LaunchProfile {
        model: Some("o3".to_string()),
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
    assert_eq!(cmd.args[model_idx + 1], "o3");
}

#[test]
fn headless_includes_skip_git_repo_check() {
    let adapter = CodexAdapter;
    let cmd = adapter.build_headless("s", &default_profile());
    assert!(
        cmd.args.contains(&"--skip-git-repo-check".to_string()),
        "codex runs in ship worktrees that may not look like a canonical git repo",
    );
}

#[test]
fn headless_falls_back_to_default_prompt() {
    let adapter = CodexAdapter;
    let cmd = adapter.build_headless("s", &default_profile());
    assert_eq!(
        cmd.args.last().unwrap(),
        "Read .artifacts/job-spec.md and execute it."
    );
}

// ── Lifecycle hooks ─────────────────────────────────────────────────────────

#[test]
fn ready_pattern_is_shell_prompt() {
    assert_eq!(CodexAdapter.ready_pattern(), "$ ");
}

#[test]
fn autostart_input_is_go() {
    assert_eq!(CodexAdapter.autostart_input(), "go");
}

#[test]
fn exit_command_is_q() {
    assert_eq!(CodexAdapter.exit_command(), "q");
}

// ── Output parsing ──────────────────────────────────────────────────────────

#[test]
fn parse_output_returns_none_for_non_json() {
    let adapter = CodexAdapter;
    assert!(adapter.parse_output("anything").is_none());
}

#[test]
fn parse_output_extracts_session_id_from_thread_started() {
    use agent_driver::HarnessEvent;
    let adapter = CodexAdapter;
    // `codex exec --json` emits a `thread.started` event up front with the
    // thread_id; that's the id resume targets.
    let line = r#"{"type":"thread.started","thread_id":"thr-abc-789"}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::SessionStarted { session_id }) => {
            assert_eq!(session_id, "thr-abc-789");
        }
        other => panic!("expected SessionStarted, got {other:?}"),
    }
}

#[test]
fn parse_output_extracts_turn_usage_from_turn_completed() {
    use agent_driver::HarnessEvent;
    let adapter = CodexAdapter;
    // Shape per codex-rs/exec/src/exec_events.rs::TurnCompletedEvent.
    // cached_input_tokens maps to cache_read_tokens; codex doesn't
    // surface cache_creation or cost.
    let line = r#"{"type":"turn.completed","usage":{"input_tokens":1234,"cached_input_tokens":56,"output_tokens":78,"reasoning_output_tokens":9}}"#;
    match adapter.parse_output(line) {
        Some(HarnessEvent::TurnUsage {
            model,
            input_tokens,
            output_tokens,
            reasoning_tokens,
            cache_creation_tokens,
            cache_read_tokens,
            stop_reason,
            harness_reported_cost_usd,
        }) => {
            assert_eq!(model, None);
            assert_eq!(input_tokens, 1234);
            assert_eq!(output_tokens, 78);
            assert_eq!(reasoning_tokens, 9);
            assert_eq!(cache_creation_tokens, 0);
            assert_eq!(cache_read_tokens, 56);
            assert_eq!(stop_reason, None);
            assert_eq!(harness_reported_cost_usd, None);
        }
        other => panic!("expected TurnUsage, got {other:?}"),
    }
}

#[test]
fn parse_output_turn_completed_missing_usage_returns_none() {
    let adapter = CodexAdapter;
    // Defensive — if codex ever ships a turn.completed without usage we
    // shouldn't panic or emit zeros that pollute analytics.
    let line = r#"{"type":"turn.completed"}"#;
    assert!(adapter.parse_output(line).is_none());
}

// ── build_resume_args ───────────────────────────────────────────────────────

#[test]
fn build_resume_args_emits_exec_resume() {
    let adapter = CodexAdapter;
    let cmd = adapter
        .build_resume_args("my-job", "thr-abc-789", &default_profile())
        .expect("codex supports resume");
    assert_eq!(cmd.bin, "codex");
    assert_eq!(cmd.args[0], "exec");
    assert_eq!(cmd.args[1], "resume");
    assert_eq!(cmd.args[2], "thr-abc-789");
    assert!(cmd.args.contains(&"--sandbox".to_string()));
    assert!(cmd.args.contains(&"workspace-write".to_string()));
    assert!(!cmd.args.contains(&"--full-auto".to_string()));
    assert!(cmd.args.contains(&"--json".to_string()));
    // Resume should not pass --ephemeral; the rollout we're resuming has
    // to exist on disk.
    assert!(!cmd.args.contains(&"--ephemeral".to_string()));
}

#[test]
fn build_resume_args_rejects_empty_session_id() {
    use agent_driver::ResumeError;
    let adapter = CodexAdapter;
    let err = adapter
        .build_resume_args("my-job", "", &default_profile())
        .expect_err("empty session id must be rejected");
    assert!(matches!(err, ResumeError::BadSession { .. }));
}

// ── Adapter identity ────────────────────────────────────────────────────────

#[test]
fn id_is_codex() {
    assert_eq!(CodexAdapter.id(), "codex");
}

// ── Capabilities ────────────────────────────────────────────────────────────

#[test]
fn caps_match_declared() {
    let c = CodexAdapter.caps();
    assert!(c.headless);
    assert!(c.interactive);
    assert!(c.server_mode, "codex: app-server");
    assert!(c.json_stream, "codex: exec --json");
    assert!(c.mcp, "codex: .codex/config.toml mcp_servers");
    assert!(c.hooks, "codex: .codex/hooks.json");
    assert!(c.session_resume, "codex: exec resume");
}
