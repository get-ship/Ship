//! Tests for GeminiAdapter — the Gemini CLI harness adapter.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::harness_adapter::HarnessAdapter;
use super::GeminiAdapter;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── Launch command tests ────────────────────────────────────────────────────

#[test]
fn interactive_emits_gemini_binary() {
    let adapter = GeminiAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.bin, "gemini");
}

#[test]
fn interactive_sets_mesh_id_env() {
    let adapter = GeminiAdapter;
    let cmd = adapter.build_interactive("my-job", &default_profile());
    assert_eq!(cmd.env.get("SHIP_ACTOR_ID").unwrap(), "my-job");
}

#[test]
fn interactive_model_flag_maps_correctly() {
    let adapter = GeminiAdapter;
    let profile = LaunchProfile {
        model: Some("gemini-2.5-pro".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_interactive("j", &profile);
    let idx = cmd.args.iter().position(|a| a == "--model").unwrap();
    assert_eq!(cmd.args[idx + 1], "gemini-2.5-pro");
}

#[test]
fn headless_emits_yolo_prompt_command() {
    let adapter = GeminiAdapter;
    let profile = LaunchProfile {
        autostart_prompt: Some("test prompt".to_string()),
        persist_session: true,
        ..Default::default()
    };
    let cmd = adapter.build_headless("test-slug", &profile);
    assert_eq!(cmd.bin, "gemini");
    assert!(cmd.args.contains(&"--yolo".to_string()));
    let p_idx = cmd.args.iter().position(|a| a == "-p").expect("-p flag");
    assert_eq!(cmd.args[p_idx + 1], "test prompt");
    assert_eq!(cmd.args.last().unwrap(), "test prompt");
    assert_eq!(
        cmd.env.get("SHIP_ACTOR_ID").map(String::as_str),
        Some("test-slug")
    );
}

#[test]
fn headless_includes_model_when_specified() {
    let adapter = GeminiAdapter;
    let profile = LaunchProfile {
        model: Some("gemini-2.5-pro".to_string()),
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
    assert_eq!(cmd.args[model_idx + 1], "gemini-2.5-pro");
}

#[test]
fn headless_falls_back_to_default_prompt() {
    let adapter = GeminiAdapter;
    let cmd = adapter.build_headless("s", &default_profile());
    assert_eq!(
        cmd.args.last().unwrap(),
        "Read .artifacts/job-spec.md and execute it."
    );
}

// ── Adapter identity ────────────────────────────────────────────────────────

#[test]
fn id_is_gemini() {
    assert_eq!(GeminiAdapter.id(), "gemini");
}

#[test]
fn parse_output_returns_none() {
    let adapter = GeminiAdapter;
    assert!(adapter.parse_output("anything").is_none());
}

// ── Capabilities ────────────────────────────────────────────────────────────

#[test]
fn caps_match_declared() {
    let c = GeminiAdapter.caps();
    assert!(c.headless);
    assert!(c.interactive);
    assert!(!c.server_mode);
    assert!(c.json_stream, "gemini: --output-format json");
    assert!(c.mcp, "gemini: .gemini/settings.json mcpServers");
    assert!(c.hooks, "gemini: .gemini/settings.json hooks");
    assert!(!c.session_resume);
}
