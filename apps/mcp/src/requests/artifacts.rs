use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

/// Upload a new artifact.
#[derive(Deserialize, JsonSchema)]
pub struct ArtifactUploadRequest {
    /// Storage scope: run | project | user
    pub scope: String,
    /// Required when scope = "run"
    #[serde(default)]
    pub run_id: Option<String>,
    /// File content, base64-encoded
    pub bytes_b64: String,
    /// File name
    pub name: String,
    /// MIME type (auto-detected from name if omitted)
    #[serde(default)]
    pub mime: Option<String>,
    /// ISO-8601 expiry timestamp (run-scope gets a default TTL if omitted)
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Any request that only needs an artifact ID.
#[derive(Deserialize, JsonSchema)]
pub struct ArtifactIdRequest {
    pub id: String,
}

/// List artifacts with optional filters.
#[derive(Deserialize, JsonSchema)]
pub struct ArtifactListRequest {
    /// Scope to list: run | project | user (default project)
    #[serde(default)]
    pub scope: Option<String>,
    /// Additional filters as a JSON object string, e.g. `{"mime":"image/png"}`
    #[serde(default)]
    pub filter_json: Option<String>,
}

/// Promote an artifact to a new scope.
#[derive(Deserialize, JsonSchema)]
pub struct ArtifactPromoteRequest {
    pub id: String,
    /// Target scope: project | user
    pub target_scope: String,
}
