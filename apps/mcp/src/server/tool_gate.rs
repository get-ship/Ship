use anyhow::Result;
use runtime::get_active_agent;
use std::path::Path;
use std::process::Command as ProcessCommand;

use super::ShipServer;

impl ShipServer {
    pub fn normalize_mode_tool_id(raw: &str) -> String {
        let mut normalized = raw.trim().to_ascii_lowercase().replace('-', "_");
        if let Some(stripped) = normalized.strip_prefix("ship_") {
            normalized = stripped.to_string();
        }
        if let Some(stripped) = normalized.strip_suffix("_tool") {
            normalized = stripped.to_string();
        }
        normalized
    }

    /// Tools available to all agents regardless of role.
    const CORE_TOOLS: &[&str] = &[
        "open_project",
        "log_progress",
        "update_job",
        "write_session_file",
        "read_session_file",
        "list_session_files",
    ];

    /// Additional tools for lead agents (job creation).
    const LEAD_TOOLS: &[&str] = &[
        "create_job",
    ];

    /// Additional tools for commander agents (full orchestration).
    const COMMANDER_TOOLS: &[&str] = &[
        "activate_workspace",
        "create_workspace",
        "complete_workspace",
        "list_stale_worktrees",
        "list_workspaces",
        "set_agent",
        "start_session",
        "end_session",
        "get_session",
        "list_sessions",
        "list_skills",
        "get_skill_vars",
        "set_skill_var",
        "list_skill_vars",
        "list_jobs",
        "get_job",
        "mesh_discover",
        "mesh_status",
    ];

    /// Debug-only tools — require explicit active_tools opt-in + unstable.
    #[cfg(feature = "unstable")]
    const DEBUG_TOOLS: &[&str] = &[
        "event",
        "mesh_send",
        "mesh_broadcast",
    ];

    pub fn is_core_tool(tool_name: &str) -> bool {
        let normalized = Self::normalize_mode_tool_id(tool_name);
        Self::CORE_TOOLS.contains(&normalized.as_str())
            || Self::LEAD_TOOLS.contains(&normalized.as_str())
            || Self::COMMANDER_TOOLS.contains(&normalized.as_str())
    }

    /// Resolve the effective tool set for a role.
    pub fn role_default_tools(role: &str) -> Vec<&'static str> {
        let mut tools: Vec<&str> = Self::CORE_TOOLS.to_vec();
        match role {
            "lead" => tools.extend_from_slice(Self::LEAD_TOOLS),
            "commander" => {
                tools.extend_from_slice(Self::LEAD_TOOLS);
                tools.extend_from_slice(Self::COMMANDER_TOOLS);
            }
            _ => {} // worker: core only
        }
        tools
    }

    pub fn is_project_workspace_tool(_tool_name: &str) -> bool {
        false
    }

    pub fn mode_allows_tool(tool_name: &str, active_tools: &[String]) -> bool {
        if active_tools.is_empty() {
            return true;
        }
        let normalized_tool = Self::normalize_mode_tool_id(tool_name);
        active_tools
            .iter()
            .map(|t| Self::normalize_mode_tool_id(t))
            .any(|allowed| allowed == normalized_tool)
    }

    pub fn enforce_mode_tool_gate(project_dir: &Path, tool_name: &str) -> Result<(), String> {
        let normalized = Self::normalize_mode_tool_id(tool_name);

        // Core tools always pass.
        if Self::CORE_TOOLS.contains(&normalized.as_str()) {
            return Ok(());
        }

        // Debug tools require explicit opt-in via active_tools.
        #[cfg(feature = "unstable")]
        let is_debug = Self::DEBUG_TOOLS.contains(&normalized.as_str());
        #[cfg(not(feature = "unstable"))]
        let is_debug = false;

        let active_agent =
            get_active_agent(Some(project_dir.to_path_buf())).map_err(|e| e.to_string())?;

        if let Some(ref agent) = active_agent {
            // Explicit active_tools overrides everything.
            if !agent.active_tools.is_empty() {
                if Self::mode_allows_tool(tool_name, &agent.active_tools) {
                    return Ok(());
                }
                return Err(format!(
                    "Tool '{}' blocked by agent '{}' (allowed: {}).",
                    tool_name, agent.id, agent.active_tools.join(", ")
                ));
            }

            // Debug tools never in role defaults — require explicit opt-in.
            if is_debug {
                return Err(format!(
                    "Tool '{}' is a debug tool. Add it to active_tools to use it.",
                    tool_name
                ));
            }

            // Role-based defaults: commander gets everything, lead gets jobs, worker gets core.
            let role = agent.role.as_deref().unwrap_or("commander");
            let defaults = Self::role_default_tools(role);
            if defaults.contains(&normalized.as_str()) {
                return Ok(());
            }
            return Err(format!(
                "Tool '{}' not available for role '{}'. Set role or active_tools in agent config.",
                tool_name, role
            ));
        }

        // No agent active — commander defaults (backward compat).
        if is_debug {
            return Err(format!(
                "Tool '{}' is a debug tool. Activate an agent with it in active_tools.",
                tool_name
            ));
        }
        let defaults = Self::role_default_tools("commander");
        if defaults.contains(&normalized.as_str()) {
            return Ok(());
        }

        Err(format!(
            "Tool '{}' is not available. Activate an agent with this tool in active_tools.",
            tool_name
        ))
    }

    pub(crate) fn resolve_workspace_branch_for_project(
        project_dir: &Path,
        branch: Option<&str>,
    ) -> Result<String, String> {
        if let Some(b) = branch {
            let trimmed = b.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
        current_branch(project_dir).map_err(|e| e.to_string())
    }
}

pub(crate) fn current_branch(project_root: &Path) -> Result<String> {
    let output = ProcessCommand::new("git")
        .args(["branch", "--show-current"])
        .current_dir(project_root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to determine current git branch");
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() {
        anyhow::bail!("Current HEAD is detached; cannot map to a feature branch");
    }
    Ok(branch)
}
