use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

/// One generalized sui-lsp analysis request over the file-based LSP ops.
#[derive(Deserialize, JsonSchema)]
pub struct SuiRequest {
    /// Analysis to run: "diagnostics" | "hover" | "dependencies".
    pub op: String,
    /// Absolute path or workspace-relative path to a `.sui` file.
    pub file: String,
    /// For "hover" only: 0-based line offset.
    #[serde(default)]
    pub line: Option<u32>,
    /// For "hover" only: 0-based column offset.
    #[serde(default)]
    pub col: Option<u32>,
}
