//! Tests for CursorAdapter — the Cursor CLI provider adapter.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::provider_adapter::ProviderAdapter;
use super::CursorAdapter;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── Launch command tests ────────────────────────────────────────────────────

#[test]
fn interactive_emits_agent_binary() {
    let adapter = CursorAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.bin, "agent");
}

#[test]
fn interactive_sets_mesh_id_env() {
    let adapter = CursorAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.env.get("SHIP_MESH_ID").unwrap(), "my-job");
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
    assert_eq!(cmd.bin, "agent");
    assert!(cmd.args.contains(&"-p".to_string()));
    assert!(cmd.args.contains(&"--force".to_string()));
    assert_eq!(cmd.args.last().unwrap(), "test prompt");
    assert_eq!(
        cmd.env.get("SHIP_MESH_ID").map(String::as_str),
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

#[test]
fn parse_output_returns_none() {
    let adapter = CursorAdapter;
    assert!(adapter.parse_output("anything").is_none());
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
