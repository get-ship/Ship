use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

/// One generalized collection-mutation request. Replaces the per-verb
/// create/update/delete request types — one effectful primitive, not CRUD.
#[derive(Deserialize, JsonSchema)]
pub struct CollectionRequest {
    /// Operation: "create", "update", or "delete".
    pub op: String,
    /// Collection ID (e.g. "adr", "task"). Must match a schema in .ship/collections/.
    pub collection: String,
    /// Item ID. Required for "update"/"delete"; optional for "create"
    /// (auto-generated when omitted).
    #[serde(default)]
    pub item_id: Option<String>,
    /// Item data (JSON object). Required for "create"/"update"; ignored for
    /// "delete". On update this is a partial merge — unspecified fields are
    /// preserved. Validated against the collection schema.
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct QueryDocsRequest {
    /// Collection ID.
    pub collection: String,
    /// Filter object using Django-style `field__op` keys.
    /// Operators: __ne, __gt, __gte, __lt, __lte, __in, __nin, __contains, __exists.
    /// Example: {"status": "accepted", "priority__gte": 2}
    #[serde(default)]
    pub filter: std::collections::BTreeMap<String, serde_json::Value>,
    /// Sort object: field names mapped to 1 (ascending) or -1 (descending).
    #[serde(default)]
    pub sort: std::collections::BTreeMap<String, i8>,
    /// Maximum items to return (default 50, max 500).
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    50
}
