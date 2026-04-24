use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct CreateDocRequest {
    /// Collection ID (e.g. "adr", "task"). Must match a schema in .ship/collections/.
    pub collection: String,
    /// Item data as a JSON object. Fields are validated against the collection schema.
    pub data: serde_json::Value,
    /// Optional item ID. Auto-generated if omitted.
    pub id: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateDocRequest {
    /// Collection ID.
    pub collection: String,
    /// Item ID to update.
    pub item_id: String,
    /// Fields to update (partial merge — unspecified fields are preserved).
    pub data: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetDocRequest {
    /// Collection ID.
    pub collection: String,
    /// Item ID to retrieve.
    pub item_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct DeleteDocRequest {
    /// Collection ID.
    pub collection: String,
    /// Item ID to delete.
    pub item_id: String,
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
