//! Legacy tool deprecation warnings for v0.2.0 → v0.3 migration.
//!
//! Tools listed here still function but their responses are prefixed with a
//! deprecation warning naming the replacement (resource URI or auto-behavior).

use rmcp::model::{CallToolResult, Content};

pub const DEPRECATION_PREFIX: &str = "DEPRECATED:";

/// Returns a deprecation warning for a legacy tool, or `None` if the tool
/// is not deprecated.
pub fn deprecation_warning(tool_name: &str) -> Option<String> {
    let replacement = match tool_name {
        "start_session" => "auto on MCP connect",
        "end_session" => "auto on MCP disconnect",
        "activate_workspace" => "auto from git branch detection",
        "open_project" => "auto from CWD detection",
        "set_agent" => "`ship use` CLI command",
        "list_workspaces" => "ship://workspaces resource",
        "get_session" => "ship://workspaces/{branch}/session resource",
        "list_sessions" => "ship://sessions resource",
        "list_skills" => "ship://skills resource",
        "get_skill_vars" => "ship://skills/{id} resource (with vars)",
        "list_skill_vars" => "ship://skills resource",
        "list_jobs" => "ship://jobs resource",
        "get_job" => "ship://jobs/{id} resource",
        "list_stale_worktrees" => "ship://stale-worktrees resource or CLI",
        "read_session_file" => "ship://session/{path} resource",
        "list_session_files" => "ship://session resource listing",
        _ => return None,
    };
    Some(format!(
        "{DEPRECATION_PREFIX} `{tool_name}` will be removed in v0.3. Use {replacement} instead."
    ))
}

/// Prepend a deprecation warning to a `CallToolResult` if the tool is deprecated.
/// Returns the result unchanged for non-deprecated tools.
pub fn prepend_deprecation_to_result(
    mut result: CallToolResult,
    tool_name: &str,
) -> CallToolResult {
    let Some(warning) = deprecation_warning(tool_name) else {
        return result;
    };
    result.content.insert(0, Content::text(warning));
    result
}
