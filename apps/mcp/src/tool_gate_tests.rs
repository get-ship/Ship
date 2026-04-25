use super::*;

// ── Preset tool sets ───────────────────────────────────────────────

#[test]
fn worker_preset_contains_platform_tools() {
    let tools = ShipServer::tools_for_preset("worker");
    assert!(tools.contains(&"log_progress"), "worker missing log_progress");
    assert!(tools.contains(&"update_job"), "worker missing update_job");
    assert!(tools.contains(&"write_session_file"), "worker missing write_session_file");
    assert!(tools.contains(&"step_done"), "worker missing step_done — pipelines depend on it");
}

#[test]
fn worker_preset_excludes_orchestration_tools() {
    let tools = ShipServer::tools_for_preset("worker");
    assert!(!tools.contains(&"create_job"), "worker must not have create_job");
    assert!(
        !tools.contains(&"discover_mesh_agents"),
        "worker must not have discover_mesh_agents"
    );
    assert!(
        !tools.contains(&"update_mesh_status"),
        "worker must not have update_mesh_status"
    );
}

#[test]
fn orchestrator_preset_extends_worker() {
    let worker = ShipServer::tools_for_preset("worker");
    let orchestrator = ShipServer::tools_for_preset("orchestrator");
    for tool in worker {
        assert!(
            orchestrator.contains(&tool),
            "orchestrator should inherit worker tool {tool}"
        );
    }
    assert!(orchestrator.contains(&"create_job"), "orchestrator missing create_job");
    assert!(
        orchestrator.contains(&"discover_mesh_agents"),
        "orchestrator missing discover_mesh_agents"
    );
    assert!(
        orchestrator.contains(&"update_mesh_status"),
        "orchestrator missing update_mesh_status"
    );
}

#[test]
fn unknown_preset_falls_back_to_worker() {
    let unknown = ShipServer::tools_for_preset("nonexistent");
    let worker = ShipServer::tools_for_preset("worker");
    assert_eq!(unknown, worker, "unknown preset should fall back to worker");
}

// ── Legacy role → preset mapping (back-compat) ─────────────────────

#[test]
fn legacy_role_worker_maps_to_worker_preset() {
    assert_eq!(ShipServer::effective_preset(None, Some("worker")), "worker");
}

#[test]
fn legacy_role_lead_maps_to_orchestrator_preset() {
    assert_eq!(ShipServer::effective_preset(None, Some("lead")), "orchestrator");
}

#[test]
fn legacy_role_commander_maps_to_orchestrator_preset() {
    assert_eq!(ShipServer::effective_preset(None, Some("commander")), "orchestrator");
}

#[test]
fn no_role_and_no_preset_defaults_to_worker() {
    assert_eq!(ShipServer::effective_preset(None, None), "worker");
}

#[test]
fn tools_preset_takes_precedence_over_legacy_role() {
    assert_eq!(
        ShipServer::effective_preset(Some("orchestrator"), Some("worker")),
        "orchestrator",
        "explicit tools_preset must win over legacy role"
    );
}

// ── Dangerous tools never in any preset ────────────────────────────

#[test]
fn dangerous_tools_excluded_from_all_presets() {
    let dangerous = ["emit_event", "send_mesh_message", "broadcast_mesh_message"];
    for preset in &["worker", "orchestrator"] {
        let tools = ShipServer::tools_for_preset(preset);
        for tool in &dangerous {
            assert!(
                !tools.contains(tool),
                "{tool} must not be in {preset} preset"
            );
        }
    }
}

// ── Dispatched-context shortcut ────────────────────────────────────

#[test]
fn is_platform_tool_recognizes_step_done() {
    assert!(ShipServer::is_platform_tool("step_done"));
    assert!(ShipServer::is_platform_tool("log_progress"));
    assert!(ShipServer::is_platform_tool("update_job"));
    assert!(ShipServer::is_platform_tool("write_session_file"));
}

#[test]
fn is_platform_tool_rejects_orchestration() {
    assert!(!ShipServer::is_platform_tool("create_job"));
    assert!(!ShipServer::is_platform_tool("discover_mesh_agents"));
}

#[test]
fn dispatched_context_detects_filesystem_marker() {
    use std::fs;
    use tempfile::tempdir;
    let tmp = tempdir().expect("tempdir");
    let wt = tmp.path().join("ship-dev-testslug");
    fs::create_dir_all(wt.join(".artifacts")).unwrap();
    fs::write(wt.join(".artifacts/.dispatched"), "ship-dev-testslug").unwrap();

    let prev_cwd = std::env::current_dir().ok();
    let prev_env = runtime::env::read_raw(runtime::env::Var::MeshId).ok();
    // SAFETY: tests run sequentially per test binary; modifying env/cwd
    // here is benign — restored before assertion.
    unsafe { runtime::env::remove(runtime::env::Var::MeshId); }
    std::env::set_current_dir(&wt).unwrap();
    let detected = ShipServer::is_dispatched_context();
    if let Some(p) = prev_cwd {
        let _ = std::env::set_current_dir(p);
    }
    if let Some(v) = prev_env {
        unsafe { runtime::env::set(runtime::env::Var::MeshId, v); }
    }
    assert!(
        detected,
        "dispatched context must be detected via .artifacts/.dispatched marker \
         even when SHIP_MESH_ID is unset (codex strips env to MCP subprocesses)",
    );
}

#[test]
fn dispatched_context_ignores_artifacts_without_marker() {
    use std::fs;
    use tempfile::tempdir;
    let tmp = tempdir().expect("tempdir");
    let wt = tmp.path().join("not-a-dispatch");
    fs::create_dir_all(wt.join(".artifacts")).unwrap();
    // `.artifacts/` exists with random contents but NO `.dispatched` marker.
    fs::write(wt.join(".artifacts/spec.md"), "random spec").unwrap();
    fs::write(wt.join(".artifacts/job-spec.md"), "old job spec").unwrap();

    let prev_cwd = std::env::current_dir().ok();
    let prev_env = runtime::env::read_raw(runtime::env::Var::MeshId).ok();
    unsafe { runtime::env::remove(runtime::env::Var::MeshId); }
    std::env::set_current_dir(&wt).unwrap();
    let detected = ShipServer::is_dispatched_context();
    if let Some(p) = prev_cwd {
        let _ = std::env::set_current_dir(p);
    }
    if let Some(v) = prev_env {
        unsafe { runtime::env::set(runtime::env::Var::MeshId, v); }
    }
    assert!(
        !detected,
        "presence of .artifacts/ alone must not trigger dispatched mode — \
         only the .dispatched marker counts",
    );
}

// ── emit_event gating (unit) ───────────────────────────────────────
//
// emit_event is registered on every binary now (no compile-time gate). It is
// classified as core (bypasses agent-mode gating) but is never folded into a
// preset — clients reach it explicitly.

#[test]
fn emit_event_is_core_so_it_bypasses_mode_gate() {
    assert!(
        ShipServer::is_core_tool("emit_event"),
        "emit_event should be core (bypasses mode gating)"
    );
    // Old alias resolves to canonical and is also treated as core.
    assert!(
        ShipServer::is_core_tool("event"),
        "deprecated `event` alias must still resolve to core"
    );
}

#[test]
fn emit_event_not_in_any_preset() {
    for preset in &["worker", "orchestrator"] {
        let tools = ShipServer::tools_for_preset(preset);
        assert!(
            !tools.contains(&"emit_event"),
            "emit_event must not be in {preset} preset"
        );
    }
}

// ── core tools bypass gating ───────────────────────────────────────

#[test]
fn core_tools_bypass_gating() {
    let core = ["create_workspace", "complete_workspace", "set_skill_var"];
    for tool in core {
        assert!(
            ShipServer::is_core_tool(tool),
            "{tool} should be a core tool (bypass gating)"
        );
    }
}

#[test]
fn deprecated_tools_are_no_longer_core() {
    let removed = [
        "open_project",
        "activate_workspace",
        "list_workspaces",
        "set_agent",
        "start_session",
        "end_session",
        "get_session",
        "list_sessions",
        "list_skills",
        "get_skill_vars",
        "list_skill_vars",
        "list_jobs",
        "get_job",
        "list_stale_worktrees",
        "read_session_file",
        "list_session_files",
    ];
    for tool in removed {
        assert!(
            !ShipServer::is_core_tool(tool),
            "{tool} is deprecated and must not be a core tool"
        );
    }
}

// ── Preset + core effective tool set sizes ─────────────────────────

#[test]
fn worker_effective_tool_set_count() {
    let core: Vec<&str> = ShipServer::CORE_TOOLS.to_vec();
    let preset = ShipServer::tools_for_preset("worker");
    let mut all: Vec<&str> = core;
    all.extend(preset.iter().copied());
    all.sort_unstable();
    all.dedup();
    // 3 core + 4 protocol (step_done, log_progress, update_job,
    // write_session_file) + 5 view canvas tools (open/patch/append/signal/
    // close) = 12. View tools are now experimental (runtime marker) but
    // always part of the platform set.
    let expected = 12;
    assert_eq!(all.len(), expected, "worker effective set mismatch: {all:?}");
}

#[test]
fn orchestrator_effective_tool_set_count() {
    let core: Vec<&str> = ShipServer::CORE_TOOLS.to_vec();
    let preset = ShipServer::tools_for_preset("orchestrator");
    let mut all: Vec<&str> = core;
    all.extend(preset.iter().copied());
    all.sort_unstable();
    all.dedup();
    // 12 worker + 3 orchestration (create_job, discover_mesh_agents,
    // update_mesh_status) = 15.
    let expected = 15;
    assert_eq!(all.len(), expected, "orchestrator effective set mismatch: {all:?}");
}
