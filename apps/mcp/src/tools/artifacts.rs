//! MCP tool implementations for the artifacts platform.
//!
//! Each tool calls the daemon's REST surface. Bytes are passed as base64
//! strings since MCP parameters are JSON.

use crate::network_client;

/// Upload an artifact. `bytes_b64` is the file content base64-encoded.
pub async fn artifact_upload(
    scope: &str,
    run_id: Option<&str>,
    bytes_b64: &str,
    name: &str,
    mime: Option<&str>,
    expires_at: Option<&str>,
) -> String {
    match network_client::artifact_upload(scope, run_id, bytes_b64, name, mime, expires_at).await {
        Ok(resp) => resp,
        Err(e) => format!("Error uploading artifact: {e}"),
    }
}

/// Download an artifact, returning the record + bytes as base64.
pub async fn artifact_get(id: &str) -> String {
    match network_client::artifact_get(id).await {
        Ok(resp) => resp,
        Err(e) => format!("Error getting artifact: {e}"),
    }
}

/// List artifacts with optional scope/filter.
pub async fn artifact_list(scope: Option<&str>, filter_json: Option<&str>) -> String {
    match network_client::artifact_list(scope, filter_json).await {
        Ok(resp) => resp,
        Err(e) => format!("Error listing artifacts: {e}"),
    }
}

/// Delete an artifact by ID.
pub async fn artifact_delete(id: &str) -> String {
    match network_client::artifact_delete(id).await {
        Ok(resp) => resp,
        Err(e) => format!("Error deleting artifact: {e}"),
    }
}

/// Promote an artifact to a new scope.
pub async fn artifact_promote(id: &str, target_scope: &str) -> String {
    match network_client::artifact_promote(id, target_scope).await {
        Ok(resp) => resp,
        Err(e) => format!("Error promoting artifact: {e}"),
    }
}
