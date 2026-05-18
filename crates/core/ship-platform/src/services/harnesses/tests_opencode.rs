//! Tests for OpenCodeAdapter — the OpenCode CLI harness adapter.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::harness_adapter::HarnessAdapter;
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
fn parse_output_returns_none() {
    let adapter = OpenCodeAdapter;
    assert!(adapter.parse_output("anything").is_none());
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
    use crate::services::harness_adapter::ResumeError;
    let adapter = OpenCodeAdapter;
    let err = adapter
        .build_resume_args("my-job", "", &default_profile())
        .expect_err("empty session id must be rejected");
    assert!(matches!(err, ResumeError::BadSession { .. }));
}
