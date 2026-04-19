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

    /// Core platform tools that always bypass gating.
    /// Only write operations that all roles need — reads are resources now.
    pub const CORE_TOOLS: &[&str] = &[
        "create_workspace",
        "complete_workspace",
        "set_skill_var",
    ];

    #[cfg(feature = "unstable")]
    const UNSTABLE_TOOLS: &[&str] = &["event"];

    // ── Tool tiers ──────────────────────────────────────────────────
    //
    // Two tiers, composed by preset. Replaces the old worker/lead/
    // commander roles (which conflated platform protocol with
    // orchestration capability).

    /// Platform protocol tools — every dispatched agent needs these to
    /// participate in the pipeline protocol. Not user-facing.
    pub const PLATFORM_TOOLS: &[&str] = &[
        "log_progress",
        "update_job",
        "write_session_file",
        // Pipeline step-completion signal. Without step_done, pipelines
        // fall back to process-exit / activity-timeout signals.
        "step_done",
        #[cfg(feature = "unstable")]
        "view_open",
        #[cfg(feature = "unstable")]
        "view_patch",
        #[cfg(feature = "unstable")]
        "view_append",
        #[cfg(feature = "unstable")]
        "view_signal",
        #[cfg(feature = "unstable")]
        "view_close",
    ];

    /// Orchestration tools — spawn child jobs, observe the mesh.
    /// Only agents with an orchestrator preset get these.
    pub const ORCHESTRATION_TOOLS: &[&str] = &["create_job", "mesh_discover", "mesh_status"];

    /// Resolve a preset name to its tool set. Unknown presets fall back
    /// to "worker" (platform tools only — the safe default).
    pub fn tools_for_preset(preset: &str) -> Vec<&'static str> {
        match preset {
            "orchestrator" => {
                let mut tools: Vec<&'static str> = Self::PLATFORM_TOOLS.to_vec();
                tools.extend_from_slice(Self::ORCHESTRATION_TOOLS);
                tools
            }
            _ => Self::PLATFORM_TOOLS.to_vec(),
        }
    }

    /// Resolve the effective preset for an agent profile:
    ///   1. Explicit `tools_preset` wins.
    ///   2. Otherwise map legacy `role`: worker→worker, lead|commander→orchestrator.
    ///   3. Otherwise "worker".
    pub fn effective_preset(tools_preset: Option<&str>, role: Option<&str>) -> &'static str {
        if let Some(p) = tools_preset {
            return match p {
                "orchestrator" => "orchestrator",
                _ => "worker",
            };
        }
        match role {
            Some("lead") | Some("commander") => "orchestrator",
            _ => "worker",
        }
    }

    /// True when this MCP server is serving a dispatched agent (SHIP_MESH_ID
    /// set by the dispatcher on spawn). Dispatched agents always get
    /// PLATFORM_TOOLS regardless of KV-stored active_agent state — the
    /// dispatch context is the source of truth.
    pub fn is_dispatched_context() -> bool {
        std::env::var("SHIP_MESH_ID")
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    /// True when `tool_name` is a platform-protocol tool (visible to every
    /// dispatched agent).
    pub fn is_platform_tool(tool_name: &str) -> bool {
        let n = Self::normalize_mode_tool_id(tool_name);
        Self::PLATFORM_TOOLS.contains(&n.as_str())
    }

    pub fn is_core_tool(tool_name: &str) -> bool {
        let normalized = Self::normalize_mode_tool_id(tool_name);
        if Self::CORE_TOOLS.contains(&normalized.as_str()) {
            return true;
        }
        #[cfg(feature = "unstable")]
        if Self::UNSTABLE_TOOLS.contains(&normalized.as_str()) {
            return true;
        }
        false
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
        if Self::is_core_tool(tool_name) {
            return Ok(());
        }
        if Self::is_project_workspace_tool(tool_name) {
            return Ok(());
        }
        // Dispatched agents (SHIP_MESH_ID set) always get platform tools.
        // This bypasses the KV-based active_agent lookup entirely — the
        // dispatch context is the source of truth.
        if Self::is_dispatched_context() && Self::is_platform_tool(tool_name) {
            return Ok(());
        }
        let active_agent =
            get_active_agent(Some(project_dir.to_path_buf())).map_err(|e| e.to_string())?;
        if let Some(ref mode) = active_agent {
            // Explicit active_tools is the complete allowlist when set —
            // lets profiles grant specific tools outside their preset
            // without adopting the whole preset.
            if !mode.active_tools.is_empty() {
                if Self::mode_allows_tool(tool_name, &mode.active_tools) {
                    return Ok(());
                }
                return Err(format!(
                    "Tool '{}' blocked by active mode '{}' (active_tools: {}).",
                    tool_name,
                    mode.id,
                    mode.active_tools.join(", ")
                ));
            }

            let preset = Self::effective_preset(mode.tools_preset.as_deref(), mode.role.as_deref());
            let preset_tools = Self::tools_for_preset(preset);
            let normalized = Self::normalize_mode_tool_id(tool_name);
            if preset_tools.contains(&normalized.as_str()) {
                return Ok(());
            }
            return Err(format!(
                "Tool '{}' blocked by active mode '{}' (preset '{}' allows: {}).",
                tool_name,
                mode.id,
                preset,
                preset_tools.join(", ")
            ));
        }
        Err(format!(
            "Tool '{}' is not in the core workflow surface. \
             Activate the service workspace ('ship') or a mode with this tool in its \
             active_tools list to use it.",
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
        let Some(root) = project_dir.parent() else {
            return Err("Error: Could not resolve project root".to_string());
        };
        current_branch(root).map_err(|e| e.to_string())
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
