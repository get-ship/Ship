use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct LintViewRequest {
    /// View name to lint. Resolves the view directory from the project's
    /// .ship/views/ tree. Use this for linting committed or saved views.
    pub name: Option<String>,
    /// Raw Jinja source to lint in-flight (no filesystem write required).
    /// Useful when the view is being drafted and not yet saved to disk.
    pub source: Option<String>,
}
