use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct SuiDiagnosticsRequest {
    /// Absolute path or workspace-relative path to a `.sui` file.
    pub file: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SuiHoverRequest {
    /// Absolute path or workspace-relative path to a `.sui` file.
    pub file: String,
    /// 0-based line offset.
    pub line: u32,
    /// 0-based column offset.
    pub col: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct SuiDependenciesRequest {
    /// Absolute path or workspace-relative path to a `.sui` file.
    pub file: String,
}
