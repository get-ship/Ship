use super::*;

// ── role-based default tool sets ───────────────────────────────────

#[test]
fn worker_defaults_contain_expected_tools() {
    let defaults = ShipServer::default_tools_for_role("worker");
    assert!(defaults.contains(&"log_progress"), "worker missing log_progress");
    assert!(defaults.contains(&"update_job"), "worker missing update_job");
    assert!(
        defaults.contains(&"write_session_file"),
        "worker missing write_session_file"
    );
}

#[test]
fn lead_defaults_extend_worker() {
    let worker = ShipServer::default_tools_for_role("worker");
    let lead = ShipServer::default_tools_for_role("lead");
    for tool in worker {
        assert!(lead.contains(tool), "lead should inherit worker tool {tool}");
    }
    assert!(lead.contains(&"create_job"), "lead missing create_job");
}

#[test]
fn commander_defaults_extend_lead() {
    let lead = ShipServer::default_tools_for_role("lead");
    let commander = ShipServer::default_tools_for_role("commander");
    for tool in lead {
        assert!(
            commander.contains(tool),
            "commander should inherit lead tool {tool}"
        );
    }
    assert!(
        commander.contains(&"mesh_discover"),
        "commander missing mesh_discover"
    );
    assert!(
        commander.contains(&"mesh_status"),
        "commander missing mesh_status"
    );
}

#[test]
fn dangerous_tools_excluded_from_all_role_defaults() {
    let dangerous = ["event", "mesh_send", "mesh_broadcast"];
    for role in &["worker", "lead", "commander"] {
        let defaults = ShipServer::default_tools_for_role(role);
        for tool in &dangerous {
            assert!(
                !defaults.contains(tool),
                "{tool} must not be in {role} defaults"
            );
        }
    }
}

#[test]
fn unknown_role_falls_back_to_worker_defaults() {
    let unknown = ShipServer::default_tools_for_role("nonexistent");
    let worker = ShipServer::default_tools_for_role("worker");
    assert_eq!(
        unknown, worker,
        "unknown role should fall back to worker defaults"
    );
}

// ── event gating (unit) ────────────────────────────────────────────

#[test]
#[cfg(not(feature = "unstable"))]
fn event_not_core_without_unstable() {
    assert!(
        !ShipServer::is_core_tool("event"),
        "event should not be core without unstable"
    );
}

#[test]
#[cfg(feature = "unstable")]
fn event_not_in_role_defaults_even_with_unstable() {
    let defaults = ShipServer::default_tools_for_role("commander");
    assert!(
        !defaults.contains(&"event"),
        "event must not be in any role defaults even with unstable"
    );
}

// ── core tools still bypass gating regardless of role ──────────────

#[test]
fn core_platform_tools_bypass_role_gating() {
    let always_core = [
        "open_project",
        "activate_workspace",
        "create_workspace",
        "complete_workspace",
        "list_workspaces",
        "set_agent",
        "start_session",
        "end_session",
        "get_session",
        "list_sessions",
        "list_skills",
        "list_jobs",
        "get_job",
        "read_session_file",
        "list_session_files",
    ];
    for tool in always_core {
        assert!(
            ShipServer::is_core_tool(tool),
            "{tool} should remain a core tool (bypass gating)"
        );
    }
}
