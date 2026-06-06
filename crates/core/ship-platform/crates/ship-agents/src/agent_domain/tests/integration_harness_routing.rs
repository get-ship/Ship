/// Integration tests — harness adapter dispatch and registry.
///
/// Exercises `get_harness`, `list_harnesses`, `enable_harness`, `disable_harness`,
/// `detect_binary`, and `adapter_for` routing against an ephemeral project.
/// No process is spawned — PATH detection (`detect_binary`) is tested with
/// known-present binaries (e.g. `sh`) and known-absent ones.
use anyhow::Result;
use tempfile::tempdir;

use crate::agent_domain::harnesses::{
    detect_binary, disable_harness, enable_harness, get_harness, list_harnesses,
};
use ship_harnesses::harness_adapter::adapter_for;
use crate::testutil::init_project;

// ── REGISTRY lookup ───────────────────────────────────────────────────────────

#[test]
fn agents_harness_routing_get_harness_returns_claude_descriptor() {
    let descriptor = get_harness("claude").expect("claude must be in HARNESSES");
    assert_eq!(descriptor.id, "claude");
    assert_eq!(descriptor.binary, "claude");
}

#[test]
fn agents_harness_routing_get_harness_returns_codex_descriptor() {
    let descriptor = get_harness("codex").expect("codex must be in HARNESSES");
    assert_eq!(descriptor.id, "codex");
    assert_eq!(descriptor.binary, "codex");
}

#[test]
fn agents_harness_routing_get_harness_returns_none_for_unknown() {
    assert!(
        get_harness("no-such-harness").is_none(),
        "get_harness must return None for an unregistered id"
    );
}

// ── adapter_for routing ───────────────────────────────────────────────────────

#[test]
fn agents_harness_routing_adapter_for_resolves_all_registered() {
    for id in ["claude", "codex", "cursor", "opencode", "pi"] {
        match adapter_for(id) {
            Ok(adapter) => assert_eq!(
                adapter.id(),
                id,
                "adapter_for('{id}') resolved to wrong adapter '{}'",
                adapter.id()
            ),
            Err(err) => panic!("adapter_for('{id}') must resolve; got: {err}"),
        }
    }
}

#[test]
fn agents_harness_routing_adapter_for_rejects_unknown() {
    let err = match adapter_for("typo-harness") {
        Ok(_) => panic!("unknown harness must return Err"),
        Err(e) => e,
    };
    assert!(
        err.contains("unknown harness"),
        "error message must name the unknown harness; got: {err}"
    );
}

// ── enable / disable harness in project config ────────────────────────────────

#[test]
fn agents_harness_routing_enable_returns_ok_for_known_harness() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    // Pins the in-memory behavior: enable_harness validates the id and
    // succeeds for a known harness. NOTE: The audit (Open Call #1 +
    // commentary on `write_project_core_config`) documents that the
    // change does NOT round-trip through ship.jsonc — `save_config` only
    // persists project identity. This test therefore asserts the
    // function's return contract, not durable persistence. A separate
    // fix (writing harnesses into ship.jsonc canonically) is required
    // before adding a list-after-enable assertion here.
    let _changed = enable_harness(&ship_dir, "codex")?;
    Ok(())
}

#[test]
fn agents_harness_routing_disable_returns_ok_for_known_harness() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    // Same persistence caveat as enable; this is a return-contract test.
    let _ = disable_harness(&ship_dir, "codex")?;
    Ok(())
}

#[test]
fn agents_harness_routing_enable_rejects_unknown_harness() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let err = enable_harness(&ship_dir, "fake-harness")
        .expect_err("enable_harness must reject unregistered harness ids");
    assert!(
        err.to_string().contains("fake-harness") || err.to_string().contains("unknown"),
        "error must name the bad harness; got: {err}"
    );
    Ok(())
}

#[test]
fn agents_harness_routing_list_harnesses_includes_all_registered() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let harnesses = list_harnesses(&ship_dir)?;
    let ids: Vec<&str> = harnesses.iter().map(|h| h.id.as_str()).collect();
    // HARNESSES slice only includes claude + codex (Open Call #3 in audit).
    assert!(ids.contains(&"claude"), "claude must appear in list_harnesses");
    assert!(ids.contains(&"codex"), "codex must appear in list_harnesses");
    Ok(())
}

// ── PATH detection ────────────────────────────────────────────────────────────

#[test]
fn agents_harness_routing_detect_binary_finds_sh_on_unix() {
    // `sh` is universally present on the target platform (Linux / macOS).
    assert!(
        detect_binary("sh"),
        "detect_binary('sh') must return true — sh is in PATH on this platform"
    );
}

#[test]
fn agents_harness_routing_detect_binary_returns_false_for_absent() {
    assert!(
        !detect_binary("this-binary-does-not-exist-in-any-path-ever"),
        "detect_binary must return false for an absent binary"
    );
}
