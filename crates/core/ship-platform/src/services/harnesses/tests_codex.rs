//! Tests for CodexAdapter — the Codex CLI harness adapter.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::harness_adapter::HarnessAdapter;
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
    assert!(cmd.args.contains(&"--full-auto".to_string()));
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
fn parse_output_returns_none() {
    let adapter = CodexAdapter;
    assert!(adapter.parse_output("anything").is_none());
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
