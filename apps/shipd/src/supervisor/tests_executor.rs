//! Executor behavior tests.
//!
//! Executors are harness-agnostic process managers. They receive a
//! HarnessAdapter and delegate all harness-specific behavior to it.

use ship_platform::services::harness_adapter::{adapter_for, adapter_for_or_default};

// ── Source helpers ───────────────────────────────────────────────────────────

fn executor_sources() -> String {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/core/ship-supervisor/src/executors");
    let mut combined = String::new();
    for name in &["tmux.rs", "process.rs", "pty.rs", "routing.rs", "mod.rs"] {
        let path = base.join(name);
        if path.exists() {
            combined.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
        }
    }
    combined
}

fn lifecycle_src() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/core/ship-supervisor/src/agent_lifecycle.rs");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("could not read agent_lifecycle.rs: {e}"))
}

// ── Executor wiring ─────────────────────────────────────────────────────────

#[test]
fn executors_resolve_adapter_from_harness_adapter_module() {
    let src = executor_sources();
    assert!(
        src.contains("harness_adapter") || src.contains("adapter_for"),
        "executor code must use harness_adapter, not hardcoded harness logic"
    );
}

#[test]
fn executors_do_not_use_deprecated_harness_launchers() {
    let src = executor_sources();
    assert!(
        !src.contains("harness_launchers"),
        "executor code must not reference the deprecated harness_launchers module"
    );
}

#[test]
fn executors_detect_readiness_via_adapter() {
    let src = executor_sources();
    assert!(
        !src.contains("contains('❯')"),
        "executors must use adapter.ready_pattern(), not hardcode '❯'"
    );
}

#[test]
fn executors_exit_gracefully_via_adapter() {
    let src = lifecycle_src();
    assert!(
        src.contains("exit_command"),
        "agent_lifecycle must call adapter.exit_command()"
    );
    assert!(
        !src.contains(r#""/exit""#),
        "agent_lifecycle must not hardcode \"/exit\""
    );
}

#[test]
fn harness_launchers_module_does_not_exist() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().and_then(|p| p.parent())
        .expect("could not resolve workspace root");
    let path = workspace_root.join("crates/core/runtime/src/services/harness_launchers.rs");
    assert!(
        !path.exists(),
        "harness_launchers.rs must be deleted — harnesses/ replaces it"
    );
}

// ── Adapter contract ────────────────────────────────────────────────────────

#[test]
fn adapter_for_none_defaults_to_claude_ready_pattern() {
    // Mode-marker line at the real input prompt — not the menu selector `❯`
    // which false-matched the channels-menu row and broke dispatch.
    let adapter = adapter_for_or_default(None).unwrap();
    assert_eq!(adapter.ready_pattern(), "bypass permissions on");
}

#[test]
fn adapter_for_claude_has_exit_command() {
    let adapter = adapter_for("claude").unwrap();
    assert_eq!(adapter.exit_command(), "/exit");
}

#[test]
fn adapter_for_codex_has_distinct_exit_command() {
    let codex = adapter_for("codex").unwrap();
    let claude = adapter_for_or_default(None).unwrap();
    assert_ne!(codex.exit_command(), claude.exit_command());
}
