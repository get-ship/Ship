//! Structural and unit tests for the executor / provider-adapter wiring.
//!
//! These tests enforce that:
//! - `adapters.rs` resolves providers through `provider_adapter::adapter_for`
//! - No deprecated `provider_launchers` references survive in executor code
//! - The ready pattern and exit command come from the adapter, not hardcoded strings
//! - The `provider_launchers` module has been deleted from the runtime

use runtime::services::provider_adapter::adapter_for;

// ── Structural: adapters.rs source ───────────────────────────────────────────

fn adapters_src() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/supervisor/adapters.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read adapters.rs: {e}"))
}

fn pipeline_src() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/supervisor/dispatch_pipeline.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read dispatch_pipeline.rs: {e}"))
}

#[test]
fn executors_resolve_adapter_from_provider_adapter_module() {
    let src = adapters_src();
    assert!(
        src.contains("provider_adapter::adapter_for"),
        "adapters.rs must import provider_adapter::adapter_for; found:\n{src}"
    );
}

#[test]
fn executors_do_not_use_deprecated_provider_launchers() {
    let src = adapters_src();
    assert!(
        !src.contains("provider_launchers"),
        "adapters.rs must not reference provider_launchers; found:\n{src}"
    );
}

#[test]
fn executors_detect_readiness_via_adapter() {
    let src = adapters_src();
    assert!(
        src.contains("ready_pattern"),
        "adapters.rs must call ready_pattern() from the adapter; found:\n{src}"
    );
    assert!(
        !src.contains("contains('❯')"),
        "adapters.rs must not hardcode the '❯' ready pattern; found:\n{src}"
    );
}

#[test]
fn executors_exit_gracefully_via_adapter() {
    let src = pipeline_src();
    assert!(
        src.contains("exit_command"),
        "dispatch_pipeline.rs must call exit_command() from the adapter; found:\n{src}"
    );
    // Literal "/exit" must not appear as a hardcoded value (string slice).
    // We allow it only as a value returned by the adapter implementation,
    // which lives in the runtime crate, not in shipd.
    assert!(
        !src.contains(r#""/exit""#),
        "dispatch_pipeline.rs must not hardcode \"/exit\"; found:\n{src}"
    );
}

#[test]
fn provider_launchers_module_does_not_exist() {
    // Navigate from apps/shipd/ up to workspace root.
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest
        .parent() // apps/
        .and_then(|p| p.parent()) // workspace root
        .expect("could not resolve workspace root from CARGO_MANIFEST_DIR");
    let launchers_path = workspace_root
        .join("crates/core/runtime/src/services/provider_launchers.rs");
    assert!(
        !launchers_path.exists(),
        "provider_launchers.rs must not exist; found at {launchers_path:?}"
    );
}

// ── Unit tests for adapter_for ────────────────────────────────────────────────

#[test]
fn adapter_for_none_defaults_to_claude_ready_pattern() {
    let adapter = adapter_for(None);
    assert_eq!(
        adapter.ready_pattern(),
        "❯",
        "default adapter (Claude) must use ❯ as ready pattern"
    );
}

#[test]
fn adapter_for_claude_has_slash_exit_command() {
    let adapter = adapter_for(Some("claude"));
    assert_eq!(
        adapter.exit_command(),
        "/exit",
        "Claude adapter must use /exit as exit command"
    );
}

#[test]
fn adapter_for_codex_has_distinct_exit_command() {
    let codex = adapter_for(Some("codex"));
    let claude = adapter_for(None);
    assert_ne!(
        codex.exit_command(),
        claude.exit_command(),
        "Codex adapter must use a different exit command from Claude"
    );
}
