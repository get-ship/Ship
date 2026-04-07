use super::*;
use runtime::project::init_project;
use runtime::{AgentProfile, add_agent};
use tempfile::tempdir;

// ── role-based gating integration ──────────────────────────────────

#[test]
fn worker_role_allows_worker_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "worker-gate".to_string(),
            name: "Worker Gate".to_string(),
            role: Some("worker".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("worker-gate"))
        .expect("set active agent");

    ShipServer::enforce_mode_tool_gate(&project_dir, "log_progress")
        .expect("log_progress allowed for worker");
    ShipServer::enforce_mode_tool_gate(&project_dir, "update_job")
        .expect("update_job allowed for worker");
    ShipServer::enforce_mode_tool_gate(&project_dir, "write_session_file")
        .expect("write_session_file allowed for worker");
}

#[test]
fn worker_role_blocks_lead_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "worker-block".to_string(),
            name: "Worker Block".to_string(),
            role: Some("worker".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("worker-block"))
        .expect("set active agent");

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

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "lead-gate".to_string(),
            name: "Lead Gate".to_string(),
            role: Some("lead".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("lead-gate"))
        .expect("set active agent");

    ShipServer::enforce_mode_tool_gate(&project_dir, "create_job")
        .expect("create_job allowed for lead");
    ShipServer::enforce_mode_tool_gate(&project_dir, "log_progress")
        .expect("log_progress allowed for lead (inherited)");
}

#[test]
fn commander_role_allows_mesh_discovery() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "cmdr-gate".to_string(),
            name: "Commander Gate".to_string(),
            role: Some("commander".to_string()),
            active_tools: vec![],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("cmdr-gate"))
        .expect("set active agent");

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

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "no-role".to_string(),
            name: "No Role".to_string(),
            active_tools: vec![],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("no-role"))
        .expect("set active agent");

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

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "override-test".to_string(),
            name: "Override".to_string(),
            role: Some("commander".to_string()),
            active_tools: vec!["create_job".to_string()],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("override-test"))
        .expect("set active agent");

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
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    for (role, id) in &[
        ("worker", "mesh-w"),
        ("lead", "mesh-l"),
        ("commander", "mesh-c"),
    ] {
        add_agent(
            Some(project_dir.clone()),
            AgentProfile {
                id: id.to_string(),
                name: format!("Mesh {role}"),
                role: Some(role.to_string()),
                active_tools: vec![],
                ..Default::default()
            },
        )
        .expect("add agent");
        runtime::set_active_agent(Some(project_dir.clone()), Some(id))
            .expect("set active agent");

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

    add_agent(
        Some(project_dir.clone()),
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
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("mesh-explicit"))
        .expect("set active agent");

    ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_send")
        .expect("mesh_send allowed with explicit active_tools");
    ShipServer::enforce_mode_tool_gate(&project_dir, "mesh_broadcast")
        .expect("mesh_broadcast allowed with explicit active_tools");
}

// ── event integration gating ───────────────────────────────────────

#[test]
#[cfg(feature = "unstable")]
fn event_allowed_with_unstable_and_explicit_active_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    add_agent(
        Some(project_dir.clone()),
        AgentProfile {
            id: "event-explicit".to_string(),
            name: "Event Explicit".to_string(),
            role: Some("worker".to_string()),
            active_tools: vec!["event".to_string()],
            ..Default::default()
        },
    )
    .expect("add agent");
    runtime::set_active_agent(Some(project_dir.clone()), Some("event-explicit"))
        .expect("set active agent");

    ShipServer::enforce_mode_tool_gate(&project_dir, "event")
        .expect("event allowed with unstable + explicit active_tools");
}
