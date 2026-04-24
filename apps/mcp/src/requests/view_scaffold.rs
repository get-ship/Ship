use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ViewScaffoldRequest {
    /// View name — used in the suggested path (.ship/views/<name>/) and the
    /// generated chrome title. Required.
    pub name: String,
    /// Collection id the view operates against. Required for the list,
    /// list-edit-delete, form, and detail shapes.
    pub collection: Option<String>,
    /// Shape of the scaffold. One of: blank, list, list-edit-delete, form,
    /// detail, tabs. Defaults to `blank`.
    pub shape: Option<String>,
}
