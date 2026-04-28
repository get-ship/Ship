use super::*;
use runtime::project::init_project;
use runtime::workspace::{
    CreateWorkspaceRequest as RuntimeCreateWorkspaceRequest,
    create_workspace as runtime_create_workspace,
};
use runtime::{AgentProfile, get_config, save_config};
use tempfile::tempdir;

fn install_active_agent(project_dir: &std::path::Path, agent: AgentProfile) {
    let mut config = get_config(Some(project_dir.to_path_buf())).expect("get config");
    let id = agent.id.clone();
    config.modes.push(agent);
    config.active_agent = Some(id);
    save_config(&config, Some(project_dir.to_path_buf())).expect("save config");
}

#[test]
fn mode_gate_normalizes_and_blocks_disallowed_tools() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    install_active_agent(
        &project_dir,
        AgentProfile {
            id: "mode-gate-test".to_string(),
            name: "Mode Gate Test".to_string(),
            active_tools: vec!["ship_list_notes".to_string()],
            ..Default::default()
        },
    );

    ShipServer::enforce_mode_tool_gate(&project_dir, "list_notes").expect("list_notes allowed");
    ShipServer::enforce_mode_tool_gate(&project_dir, "ship_list_notes_tool")
        .expect("prefixed note tool allowed");
    ShipServer::enforce_mode_tool_gate(&project_dir, "create_workspace")
        .expect("create_workspace must be core");
    ShipServer::enforce_mode_tool_gate(&project_dir, "complete_workspace")
        .expect("complete_workspace must be core");
    ShipServer::enforce_mode_tool_gate(&project_dir, "set_skill_var")
        .expect("set_skill_var must be core");

    let blocked = ShipServer::enforce_mode_tool_gate(&project_dir, "create_spec")
        .expect_err("create_spec should be blocked");
    assert!(
        blocked.contains("blocked by active mode"),
        "unexpected mode gate message: {}",
        blocked
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_workspace_control_plane_round_trip() {
    let tmp = tempdir().expect("tempdir");
    let project_dir = init_project(tmp.path().to_path_buf()).expect("init project");

    let server = ShipServer::new();
    *server.active_project.lock().await = Some(project_dir.clone());

    runtime_create_workspace(
        &project_dir,
        RuntimeCreateWorkspaceRequest {
            branch: "feature/mode-control-plane".to_string(),
            ..Default::default()
        },
    )
    .expect("create workspace for test");

    let fetched = server
        .resolve_resource_uri("ship://workspaces/feature/mode-control-plane", &project_dir)
        .await
        .expect("workspace resource");
    assert!(
        fetched.contains("\"id\": \"feature-mode-control-plane\""),
        "unexpected get workspace response: {}",
        fetched
    );

    let sessions_before = server
        .resolve_resource_uri("ship://sessions/feature/mode-control-plane", &project_dir)
        .await
        .expect("sessions resource");
    assert_eq!(
        sessions_before.trim(),
        "[]",
        "expected no sessions before start, got {}",
        sessions_before
    );
}

// ── notification peer lifecycle ─────────────────────────────────────

#[tokio::test]
async fn notification_peer_starts_as_none() {
    let server = ShipServer::new();
    assert!(
        server.notification_peer.lock().await.is_none(),
        "notification_peer should be None after construction"
    );
}

#[tokio::test]
async fn notify_resources_changed_is_noop_without_peer() {
    let server = ShipServer::new();
    // Should not panic when no peer is stored
    server.notify_resources_changed().await;
}

// ── normalize_mode_tool_id ──────────────────────────────────────────

#[test]
fn strips_ship_prefix() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("ship_create_note"),
        "create_note"
    );
}

#[test]
fn strips_tool_suffix() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("list_notes_tool"),
        "list_notes"
    );
}

#[test]
fn strips_both_prefix_and_suffix() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("ship_create_workspace_tool"),
        "create_workspace"
    );
}

#[test]
fn lowercases_and_replaces_hyphens() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("Ship-Create-Note"),
        "create_note"
    );
}

#[test]
fn already_normalized_unchanged() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("create_note"),
        "create_note"
    );
}

#[test]
fn trims_whitespace() {
    assert_eq!(
        ShipServer::normalize_mode_tool_id("  ship_foo_tool  "),
        "foo"
    );
}

// ── is_core_tool ────────────────────────────────────────────────────

#[test]
fn core_tools_are_recognized() {
    let core_tools = [
        "create_workspace",
        "complete_workspace",
        "set_skill_var",
    ];
    for tool in core_tools {
        assert!(
            ShipServer::is_core_tool(tool),
            "{tool} should be a core tool"
        );
    }
}

#[test]
fn core_tool_with_prefix_and_suffix() {
    assert!(ShipServer::is_core_tool("ship_create_workspace_tool"));
    assert!(ShipServer::is_core_tool("ship_set_skill_var_tool"));
}

#[test]
fn create_adr_not_core_after_removal() {
    assert!(
        !ShipServer::is_core_tool("create_adr"),
        "create_adr removed — should not be core"
    );
}

#[test]
fn non_core_tool_is_not_core() {
    assert!(!ShipServer::is_core_tool("create_spec"));
    assert!(!ShipServer::is_core_tool("random_tool"));
}

// ── mode_allows_tool ────────────────────────────────────────────────

#[test]
fn empty_active_tools_allows_everything_at_tool_level() {
    // mode_allows_tool is the low-level check; role defaults are applied
    // at the enforce_mode_tool_gate level. The raw function still returns
    // true for empty lists.
    assert!(ShipServer::mode_allows_tool("anything", &[]));
    assert!(ShipServer::mode_allows_tool("create_spec", &[]));
}

#[test]
fn tool_in_active_tools_allowed() {
    let active = vec!["create_spec".to_string(), "list_notes".to_string()];
    assert!(ShipServer::mode_allows_tool("create_spec", &active));
    assert!(ShipServer::mode_allows_tool("list_notes", &active));
}

#[test]
fn tool_not_in_active_tools_blocked() {
    let active = vec!["create_spec".to_string()];
    assert!(!ShipServer::mode_allows_tool("delete_everything", &active));
}

#[test]
fn normalization_applied_to_both_sides() {
    let active = vec!["ship_create_spec_tool".to_string()];
    assert!(ShipServer::mode_allows_tool("create_spec", &active));
    assert!(ShipServer::mode_allows_tool(
        "ship_create_spec_tool",
        &active
    ));
}

// ── tool registration surface ──────────────────────────────────────

#[test]
fn registers_canonical_verb_first_tool_names() {
    let server = ShipServer::new();
    let names = server.registered_tool_names();
    let expected: &[&str] = &[
        "create_workspace",
        "complete_workspace",
        "log_progress",
        "set_skill_var",
        "write_session_file",
        "create_job",
        "update_job",
        "send_mesh_message",
        "broadcast_mesh_message",
        "discover_mesh_agents",
        "update_mesh_status",
        "upload_artifact",
        "get_artifact",
        "list_artifacts",
        "delete_artifact",
        "promote_artifact",
        "open_view",
        "patch_view",
        "append_view",
        "signal_view",
        "close_view",
        "emit_event",
        "dispatch_agent",
        "list_agents",
        "stop_agent",
        "steer_agent",
        "lint_view",
        "scaffold_view",
        "query_sdk",
        "get_view_errors",
    ];
    for tool in expected {
        assert!(
            names.iter().any(|n| n == tool),
            "{tool} missing from router; have: {names:?}"
        );
    }
}

#[test]
fn deprecated_aliases_resolve_to_canonical_names() {
    // Aliases are kept for one release with a deprecation prefix in the
    // description. They must be present alongside the canonical names.
    let server = ShipServer::new();
    let names = server.registered_tool_names();
    let aliases: &[&str] = &[
        "mesh_send",
        "mesh_broadcast",
        "mesh_discover",
        "mesh_status",
        "ship_view_errors",
        "ship_view_lint",
        "ship_sdk_query",
        "ship_view_scaffold",
        "artifact_upload",
        "artifact_get",
        "artifact_list",
        "artifact_delete",
        "artifact_promote",
        "view_open",
        "view_patch",
        "view_append",
        "view_signal",
        "view_close",
        "event",
    ];
    for alias in aliases {
        assert!(
            names.iter().any(|n| n == alias),
            "deprecated alias {alias} missing — must register for one release"
        );
    }
}

#[test]
fn deprecated_tools_not_registered() {
    let server = ShipServer::new();
    let names = server.registered_tool_names();
    // These tools were removed in earlier cleanup and must NOT register on
    // the agent-facing ShipServer. Tools that look related (e.g. list_skills)
    // live on StudioServer instead.
    let removed: &[&str] = &[
        "open_project",
        "set_agent",
        "activate_workspace",
        "list_workspaces",
        "list_stale_worktrees",
        "start_session",
        "end_session",
        "get_session",
        "list_sessions",
        "list_session_files",
        "read_session_file",
        "list_skills",
        "get_skill_vars",
        "list_skill_vars",
        "list_jobs",
        "get_job",
    ];
    for tool in removed {
        assert!(
            !names.iter().any(|n| n == tool),
            "{tool} should be removed from router"
        );
    }
}

#[test]
fn every_build_registers_emit_event() {
    // Compile-time gating of tool registration is gone. emit_event (formerly
    // gated behind the `unstable` feature) is now always present, with an
    // **EXPERIMENTAL** prefix on its description as the runtime marker.
    let server = ShipServer::new();
    let names = server.registered_tool_names();
    assert!(
        names.iter().any(|n| n == "emit_event"),
        "emit_event must be registered on every build (runtime stability marker, not compile-time)"
    );
    // Deprecation alias still resolves for one release.
    assert!(
        names.iter().any(|n| n == "event"),
        "deprecated alias `event` must still register"
    );
}

#[test]
fn experimental_tools_carry_marker_in_description() {
    use crate::tools::stability::EXPERIMENTAL_PREFIX;
    let server = ShipServer::new();
    let tools = server.tool_router.list_all();
    for tool_name in crate::tools::stability::EXPERIMENTAL_TOOLS {
        let tool = tools
            .iter()
            .find(|t| t.name.as_ref() == *tool_name)
            .unwrap_or_else(|| panic!("experimental tool {tool_name} not registered"));
        let desc = tool.description.clone().unwrap_or_default();
        assert!(
            desc.starts_with(EXPERIMENTAL_PREFIX),
            "{tool_name} description missing EXPERIMENTAL prefix: {desc:?}"
        );
    }
}

#[test]
fn create_adr_not_registered_in_any_build() {
    let server = ShipServer::new();
    let names = server.registered_tool_names();
    assert!(
        !names.iter().any(|n| n == "create_adr"),
        "create_adr should be removed from tool registration"
    );
}
