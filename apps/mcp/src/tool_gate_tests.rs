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
        assert!(lead.contains(&tool), "lead should inherit worker tool {tool}");
    }
    assert!(lead.contains(&"create_job"), "lead missing create_job");
}

#[test]
fn commander_defaults_extend_lead() {
    let lead = ShipServer::default_tools_for_role("lead");
    let commander = ShipServer::default_tools_for_role("commander");
    for tool in lead {
        assert!(
            commander.contains(&tool),
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

// ── worker role sees exactly 6 tools via role defaults + core ──────

#[test]
fn worker_effective_tool_set_is_six_tools() {
    let core: Vec<&str> = ShipServer::CORE_TOOLS.to_vec();
    let role_defaults = ShipServer::default_tools_for_role("worker");
    let mut all: Vec<&str> = core;
    all.extend(role_defaults.iter().copied());
    all.sort_unstable();
    all.dedup();
    // create_workspace, complete_workspace, set_skill_var, log_progress, update_job, write_session_file
    assert_eq!(
        all.len(),
        6,
        "worker should see exactly 6 tools, got: {all:?}"
    );
}

// ── commander role sees exactly 9 tools ────────────────────────────

#[test]
fn commander_effective_tool_set_is_nine_tools() {
    let core: Vec<&str> = ShipServer::CORE_TOOLS.to_vec();
    let role_defaults = ShipServer::default_tools_for_role("commander");
    let mut all: Vec<&str> = core;
    all.extend(role_defaults.iter().copied());
    all.sort_unstable();
    all.dedup();
    // create_workspace, complete_workspace, set_skill_var,
    // log_progress, update_job, write_session_file,
    // create_job, mesh_discover, mesh_status
    assert_eq!(
        all.len(),
        9,
        "commander should see exactly 9 tools, got: {all:?}"
    );
}
