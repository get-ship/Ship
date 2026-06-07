// -- Compile aggregate --------------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompileCompleted {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_agent: Option<String>,
    pub compiled_at: String,
    pub ship_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins_installed: Vec<String>,
}
