//! Tests for MCP legacy tool deprecation warnings.
//!
//! Legacy tools still function but responses are prefixed with a deprecation
//! warning naming the replacement. This is v0.2.0 prep for v0.3 removal.

use crate::server::deprecation::{
    deprecation_warning, prepend_deprecation_to_result, DEPRECATION_PREFIX,
};
use rmcp::model::{CallToolResult, Content, RawContent};

// ── deprecation_warning returns correct replacements ───────────────

#[test]
fn start_session_deprecated() {
    let warning = deprecation_warning("start_session").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("auto") || warning.contains("MCP connect"),
        "should mention auto on MCP connect, got: {warning}"
    );
}

#[test]
fn end_session_deprecated() {
    let warning = deprecation_warning("end_session").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("auto") || warning.contains("MCP disconnect"),
        "should mention auto on MCP disconnect, got: {warning}"
    );
}

#[test]
fn activate_workspace_deprecated() {
    let warning = deprecation_warning("activate_workspace").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("git branch"),
        "should mention git branch detection, got: {warning}"
    );
}

#[test]
fn open_project_deprecated() {
    let warning = deprecation_warning("open_project").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("CWD"),
        "should mention CWD detection, got: {warning}"
    );
}

#[test]
fn set_agent_deprecated() {
    let warning = deprecation_warning("set_agent").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship use"),
        "should mention `ship use` CLI, got: {warning}"
    );
}

#[test]
fn list_workspaces_deprecated() {
    let warning = deprecation_warning("list_workspaces").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://workspaces"),
        "should mention ship://workspaces resource, got: {warning}"
    );
}

#[test]
fn get_session_deprecated() {
    let warning = deprecation_warning("get_session").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://workspaces/") && warning.contains("/session"),
        "should mention ship://workspaces/{{branch}}/session resource, got: {warning}"
    );
}

#[test]
fn list_sessions_deprecated() {
    let warning = deprecation_warning("list_sessions").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://sessions"),
        "should mention ship://sessions resource, got: {warning}"
    );
}

#[test]
fn list_skills_deprecated() {
    let warning = deprecation_warning("list_skills").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://skills"),
        "should mention ship://skills resource, got: {warning}"
    );
}

#[test]
fn get_skill_vars_deprecated() {
    let warning = deprecation_warning("get_skill_vars").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://skills/"),
        "should mention ship://skills/{{id}} resource, got: {warning}"
    );
}

#[test]
fn list_skill_vars_deprecated() {
    let warning = deprecation_warning("list_skill_vars").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://skills"),
        "should mention ship://skills resource, got: {warning}"
    );
}

#[test]
fn list_jobs_deprecated() {
    let warning = deprecation_warning("list_jobs").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://jobs"),
        "should mention ship://jobs resource, got: {warning}"
    );
}

#[test]
fn get_job_deprecated() {
    let warning = deprecation_warning("get_job").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://jobs/"),
        "should mention ship://jobs/{{id}} resource, got: {warning}"
    );
}

#[test]
fn list_stale_worktrees_deprecated() {
    let warning = deprecation_warning("list_stale_worktrees").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://stale-worktrees") || warning.contains("CLI"),
        "should mention ship://stale-worktrees resource or CLI, got: {warning}"
    );
}

#[test]
fn read_session_file_deprecated() {
    let warning = deprecation_warning("read_session_file").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://session/"),
        "should mention ship://session/{{path}} resource, got: {warning}"
    );
}

#[test]
fn list_session_files_deprecated() {
    let warning = deprecation_warning("list_session_files").expect("should be deprecated");
    assert!(warning.contains(DEPRECATION_PREFIX));
    assert!(
        warning.contains("ship://session"),
        "should mention ship://session resource listing, got: {warning}"
    );
}

// ── non-deprecated tools return None ───────────────────────────────

#[test]
fn log_progress_not_deprecated() {
    assert!(
        deprecation_warning("log_progress").is_none(),
        "log_progress should not be deprecated"
    );
}

#[test]
fn create_workspace_not_deprecated() {
    assert!(
        deprecation_warning("create_workspace").is_none(),
        "create_workspace should not be deprecated"
    );
}

#[test]
fn complete_workspace_not_deprecated() {
    assert!(
        deprecation_warning("complete_workspace").is_none(),
        "complete_workspace should not be deprecated"
    );
}

#[test]
fn write_session_file_not_deprecated() {
    assert!(
        deprecation_warning("write_session_file").is_none(),
        "write_session_file should not be deprecated"
    );
}

#[test]
fn create_job_not_deprecated() {
    assert!(
        deprecation_warning("create_job").is_none(),
        "create_job should not be deprecated"
    );
}

#[test]
fn update_job_not_deprecated() {
    assert!(
        deprecation_warning("update_job").is_none(),
        "update_job should not be deprecated"
    );
}

#[test]
fn event_not_deprecated() {
    assert!(
        deprecation_warning("event").is_none(),
        "event should not be deprecated"
    );
}

#[test]
fn mesh_send_not_deprecated() {
    assert!(
        deprecation_warning("mesh_send").is_none(),
        "mesh_send should not be deprecated"
    );
}

#[test]
fn unknown_tool_not_deprecated() {
    assert!(
        deprecation_warning("nonexistent_tool").is_none(),
        "unknown tools should not be deprecated"
    );
}

// ── warning format ─────────────────────────────────────────────────

#[test]
fn warning_starts_with_prefix() {
    for tool in LEGACY_TOOLS {
        let warning = deprecation_warning(tool)
            .unwrap_or_else(|| panic!("{tool} should be deprecated"));
        assert!(
            warning.starts_with(DEPRECATION_PREFIX),
            "{tool} warning should start with DEPRECATION_PREFIX, got: {warning}"
        );
    }
}

#[test]
fn warning_mentions_removal_version() {
    for tool in LEGACY_TOOLS {
        let warning = deprecation_warning(tool)
            .unwrap_or_else(|| panic!("{tool} should be deprecated"));
        assert!(
            warning.contains("v0.3"),
            "{tool} warning should mention v0.3 removal, got: {warning}"
        );
    }
}

/// All legacy tools from the deprecation spec.
const LEGACY_TOOLS: &[&str] = &[
    "start_session",
    "end_session",
    "activate_workspace",
    "open_project",
    "set_agent",
    "list_workspaces",
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

#[test]
fn all_legacy_tools_have_deprecation_warnings() {
    for tool in LEGACY_TOOLS {
        assert!(
            deprecation_warning(tool).is_some(),
            "{tool} should have a deprecation warning"
        );
    }
}

#[test]
fn exactly_sixteen_legacy_tools() {
    assert_eq!(
        LEGACY_TOOLS.len(),
        16,
        "spec defines exactly 16 legacy tools"
    );
}

// ── prepend_deprecation_to_result integration ──────────────────────

#[test]
fn prepend_adds_warning_text_before_content() {
    let original = CallToolResult::success(vec![Content::text("some data")]);
    let result = prepend_deprecation_to_result(original, "start_session");

    // Should have 2 content items: warning + original data
    assert_eq!(
        result.content.len(),
        2,
        "expected warning + data, got {} items",
        result.content.len()
    );

    let first_text = extract_text(&result.content[0]);
    assert!(
        first_text.contains(DEPRECATION_PREFIX),
        "first content should be the deprecation warning, got: {first_text}"
    );

    let second_text = extract_text(&result.content[1]);
    assert_eq!(
        second_text, "some data",
        "original data should be preserved"
    );
}

#[test]
fn prepend_is_noop_for_non_deprecated_tool() {
    let original = CallToolResult::success(vec![Content::text("workspace created")]);
    let result = prepend_deprecation_to_result(original, "create_workspace");

    assert_eq!(
        result.content.len(),
        1,
        "non-deprecated tool should not get extra content"
    );
    let text = extract_text(&result.content[0]);
    assert_eq!(text, "workspace created");
}

#[test]
fn prepend_preserves_error_status() {
    let original = CallToolResult::error(vec![Content::text("some error")]);
    let result = prepend_deprecation_to_result(original, "list_workspaces");

    // Warning should still be prepended even on error results
    assert_eq!(result.content.len(), 2);
    assert!(result.is_error == Some(true), "error flag should be preserved");
}

#[test]
fn prepend_handles_empty_content() {
    let original = CallToolResult::success(vec![]);
    let result = prepend_deprecation_to_result(original, "open_project");

    assert_eq!(
        result.content.len(),
        1,
        "should have just the warning for empty content"
    );
    let text = extract_text(&result.content[0]);
    assert!(text.contains(DEPRECATION_PREFIX));
}

#[test]
fn prepend_handles_multi_content() {
    let original = CallToolResult::success(vec![
        Content::text("line 1"),
        Content::text("line 2"),
    ]);
    let result = prepend_deprecation_to_result(original, "list_sessions");

    assert_eq!(
        result.content.len(),
        3,
        "warning + 2 original items"
    );
    let first_text = extract_text(&result.content[0]);
    assert!(first_text.contains(DEPRECATION_PREFIX));
}

/// Extract text from a Content item. Panics if not a text content.
fn extract_text(content: &Content) -> String {
    match &content.raw {
        RawContent::Text(t) => t.text.clone(),
        other => panic!("expected text content, got: {other:?}"),
    }
}
