use super::*;
use runtime::project::init_project;
use runtime::{AgentProfile, get_config, save_config};
use tempfile::tempdir;

fn install_active_agent(project_dir: &std::path::Path, agent: AgentProfile) {
    let mut config = get_config(Some(project_dir.to_path_buf())).expect("get config");
    let id = agent.id.clone();
    config.agents.push(agent);
    config.active_agent = Some(id);
    save_config(&config, Some(project_dir.to_path_buf())).expect("save config");
}

// ── role-based gating integration ──────────────────────────────────

#[test]
fn worker_role_allows_worker_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "worker-gate".to_string(),
            name: "Worker Gate".to_string(),
            role: Some("worker".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    );

    ShipServer::enforce_mode_tool_gate(&project_dir, "log_progress")
        .expect("log_progress allowed for worker");
    ShipServer::enforce_mode_tool_gate(&project_dir, "update_job")
        .expect("update_job allowed for worker");
    ShipServer::enforce_mode_tool_gate(&project_dir, "write_session_file")
        .expect("write_session_file allowed for worker");
    // step_done is the pipeline completion signal — every dispatched
    // worker agent needs it, or pipelines stall at activity-timeout.
    ShipServer::enforce_mode_tool_gate(&project_dir, "step_done")
        .expect("step_done allowed for worker");
}

#[test]
fn worker_role_blocks_lead_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "worker-block".to_string(),
            name: "Worker Block".to_string(),
            role: Some("worker".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    );

    let err = ShipServer::enforce_mode_tool_gate(&project_dir, "create_job")
        .expect_err("create_job should be blocked for worker");
    assert!(
        err.contains("blocked"),
        "expected 'blocked' in error: {err}"
    );
}

#[test]
fn lead_role_allows_create_job() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "lead-gate".to_string(),
            name: "Lead Gate".to_string(),
            role: Some("lead".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    );

    ShipServer::enforce_mode_tool_gate(&project_dir, "create_job")
        .expect("create_job allowed for lead");
    ShipServer::enforce_mode_tool_gate(&project_dir, "log_progress")
        .expect("log_progress allowed for lead (inherited)");
}

#[test]
fn commander_role_allows_mesh_discovery() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "cmdr-gate".to_string(),
            name: "Commander Gate".to_string(),
            role: Some("commander".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    );

    ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_discover")
        .expect("mesh_discover allowed for commander");
    ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_status")
        .expect("mesh_status allowed for commander");
    ShipServer::enforce_mode_tool_gate(&project_dir, "create_job")
        .expect("create_job allowed for commander (inherited)");
}

#[test]
fn no_role_defaults_to_worker() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "no-role".to_string(),
            name: "No Role".to_string(),
            active_tools: vec![],
            ..Default::default()
        },
    );

    ShipServer::enforce_mode_tool_gate(&project_dir, "log_progress")
        .expect("log_progress allowed (defaults to worker)");
    let err = ShipServer::enforce_mode_tool_gate(&project_dir, "create_job")
        .expect_err("create_job blocked (defaults to worker)");
    assert!(err.contains("blocked"), "expected 'blocked' in error: {err}");
}

// ── explicit active_tools overrides role defaults ──────────────────

#[test]
fn explicit_active_tools_overrides_role_defaults() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "override-test".to_string(),
            name: "Override".to_string(),
            role: Some("commander".to_string()),
            active_tools: vec!["create_job".to_string()],
            ..Default::default()
        },
    );

    ShipServer::enforce_mode_tool_gate(&project_dir, "create_job")
        .expect("create_job in explicit active_tools");

    let err = ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_discover")
        .expect_err("mesh_discover not in explicit active_tools");
    assert!(
        err.contains("blocked"),
        "expected 'blocked' in error: {err}"
    );
}

// ── dangerous tools require explicit active_tools ──────────────────

#[test]
fn mesh_send_blocked_for_all_roles_without_explicit_active_tools() {
    for (role, id) in &[
        ("worker", "mesh-w"),
        ("lead", "mesh-l"),
        ("commander", "mesh-c"),
    ] {
        let tmp = tempdir().expect("tempdir");
        let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

        install_active_agent(
            &project_dir,
            AgentProfile {
                id: id.to_string(),
                name: format!("Mesh {role}"),
                role: Some(role.to_string()),
                active_tools: vec![],
                ..Default::default()
            },
        );

        ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_send")
            .expect_err(&format!("mesh_send should be blocked for {role}"));
        ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_broadcast")
            .expect_err(&format!("mesh_broadcast should be blocked for {role}"));
    }
}

#[test]
fn mesh_send_allowed_with_explicit_active_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "mesh-explicit".to_string(),
            name: "Mesh Explicit".to_string(),
            role: Some("worker".to_string()),
            active_tools: vec![
                "mesh_send".to_string(),
                "mesh_broadcast".to_string(),
            ],
            ..Default::default()
        },
    );

    ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_send")
        .expect("mesh_send allowed with explicit active_tools");
    ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_broadcast")
        .expect("mesh_broadcast allowed with explicit active_tools");
}

// ── emit_event integration gating ──────────────────────────────────

#[test]
fn emit_event_is_core_and_callable_under_any_mode() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "event-explicit".to_string(),
            name: "Event Explicit".to_string(),
            role: Some("worker".to_string()),
            active_tools: vec!["log_progress".to_string()],
            ..Default::default()
        },
    );

    // emit_event is core — bypasses mode gating regardless of active_tools.
    ShipServer::enforce_mode_tool_gate(&project_dir, "emit_event")
        .expect("emit_event must bypass mode gate as a core tool");
    // The deprecated alias resolves to the same canonical name.
    ShipServer::enforce_mode_tool_gate(&project_dir, "event")
        .expect("`event` alias must resolve to canonical `emit_event` and bypass gate");
}

// ── canvas view tool gating (always registered, runtime-experimental) ─

#[test]
fn worker_role_allows_view_tools_by_default() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");
    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "view-worker".into(),
            name: "View Worker".into(),
            role: Some("worker".into()),
            active_tools: vec![],
            ..Default::default()
        },
    );

    for tool in &["open_view", "patch_view", "append_view", "signal_view", "close_view"] {
        ShipServer::enforce_mode_tool_gate(&project_dir, tool)
            .unwrap_or_else(|e| panic!("{tool} must be allowed for worker role: {e}"));
    }
    // Deprecated aliases resolve to the canonical names through the gate.
    for legacy in &["view_open", "view_patch", "view_append", "view_signal", "view_close"] {
        ShipServer::enforce_mode_tool_gate(&project_dir, legacy)
            .unwrap_or_else(|e| panic!("legacy alias {legacy} must still pass: {e}"));
    }
}

#[test]
fn explicit_active_tools_without_view_blocks_view_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");
    // A "gate" mode declares its allowed tools explicitly — and omits view.*.
    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "gate-mode".into(),
            name: "Gate Mode".into(),
            role: Some("worker".into()),
            active_tools: vec!["log_progress".into(), "update_job".into()],
            ..Default::default()
        },
    );

    assert!(
        ShipServer::enforce_mode_tool_gate(&project_dir, "open_view").is_err(),
        "explicit active_tools without open_view must block it"
    );
}
