//! Tests for the provider adapter registry and trait contract.

use crate::services::dispatch_ports::LaunchProfile;
use crate::services::provider_adapter::adapter_for;

fn default_profile() -> LaunchProfile {
    LaunchProfile { persist_session: true, ..Default::default() }
}

// ── adapter_for() registry ──────────────────────────────────────────────────

#[test]
fn adapter_for_none_returns_claude() {
    let adapter = adapter_for(None);
    assert_eq!(adapter.id(), "claude");
}

#[test]
fn adapter_for_claude_returns_claude() {
    let adapter = adapter_for(Some("claude"));
    assert_eq!(adapter.id(), "claude");
}

#[test]
fn adapter_for_claude_code_returns_claude() {
    let adapter = adapter_for(Some("claude-code"));
    assert_eq!(adapter.id(), "claude");
}

#[test]
fn adapter_for_codex_returns_codex() {
    let adapter = adapter_for(Some("codex"));
    assert_eq!(adapter.id(), "codex");
}

#[test]
fn adapter_for_unknown_falls_back_to_claude() {
    let adapter = adapter_for(Some("unknown-provider"));
    assert_eq!(adapter.id(), "claude");
}

// ── Trait contract: every adapter produces valid LaunchCommands ──────────────

#[test]
fn all_adapters_set_mesh_id_env() {
    let providers: &[Option<&str>] = &[None, Some("claude"), Some("codex")];
    for provider in providers {
        let adapter = adapter_for(*provider);
        let cmd = adapter.build_interactive("test-slug", &default_profile());
        assert_eq!(
            cmd.env.get("SHIP_MESH_ID").map(|s| s.as_str()),
            Some("test-slug"),
            "adapter {} must set SHIP_MESH_ID",
            adapter.id()
        );
    }
}

#[test]
fn all_adapters_produce_non_empty_bin() {
    let providers: &[Option<&str>] = &[None, Some("claude"), Some("codex")];
    for provider in providers {
        let adapter = adapter_for(*provider);
        let cmd = adapter.build_interactive("s", &default_profile());
        assert!(
            !cmd.bin.is_empty(),
            "adapter {} must produce a non-empty binary name",
            adapter.id()
        );
    }
}

#[test]
fn all_adapters_to_shell_string_is_non_empty() {
    let providers: &[Option<&str>] = &[None, Some("claude"), Some("codex")];
    for provider in providers {
        let adapter = adapter_for(*provider);
        let cmd = adapter.build_interactive("s", &default_profile());
        assert!(
            !cmd.to_shell_string().is_empty(),
            "adapter {} shell string must be non-empty",
            adapter.id()
        );
    }
}
