//! Legacy tool deprecation warnings for v0.2.0 → v0.3 migration.
//!
//! Tools listed here still function but their responses are prefixed with a
//! deprecation warning naming the replacement (resource URI or auto-behavior).

use rmcp::model::{CallToolResult, Content};

pub const DEPRECATION_PREFIX: &str = "DEPRECATED:";

/// Returns a deprecation warning for a legacy tool, or `None` if the tool
/// is not deprecated.
pub fn deprecation_warning(_tool_name: &str) -> Option<String> {
    // TODO: implement legacy tool → warning mapping
    None
}

/// Prepend a deprecation warning to a `CallToolResult` if the tool is deprecated.
/// Returns the result unchanged for non-deprecated tools.
pub fn prepend_deprecation_to_result(result: CallToolResult, tool_name: &str) -> CallToolResult {
    let Some(_warning) = deprecation_warning(tool_name) else {
        return result;
    };
    // TODO: prepend warning Content::text to result.content
    result
}
