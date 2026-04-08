//! Tests for CodexAdapter — the Codex CLI provider adapter.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::provider_adapter::ProviderAdapter;
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
    assert_eq!(cmd.env.get("SHIP_MESH_ID").unwrap(), "my-job");
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
fn headless_same_as_interactive() {
    let adapter = CodexAdapter;
    let profile = default_profile();
    let interactive = adapter.build_interactive("j", &profile);
    let headless = adapter.build_headless("j", &profile);
    assert_eq!(interactive.bin, headless.bin);
    assert_eq!(interactive.args, headless.args);
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
