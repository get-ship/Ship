//! Shared wire types — request/response shapes used by every layer of
//! the daemon. Lives below `auth` in the module DAG so auth handlers
//! can reference response shapes without an upward `auth → routes`
//! import. See `scripts/lint/check-module-dag.sh` for the constraint.

use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

/// Envelope returned by every handler under `/api/v1/*` that isn't a
/// raw bytes / SSE / stream response. `ok` flags success; `data` is
/// either the payload or a structured error description.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({ "ok": true, "data": { "registered": "agent-writer" } }))]
pub struct MeshResponse {
    /// `true` on success; `false` when the handler returned a structured error.
    pub ok: bool,
    /// Response payload; shape varies by endpoint. See individual operation
    /// responses for concrete shapes. Represented as arbitrary JSON because
    /// this envelope is reused across dozens of handlers that return
    /// heterogeneous bodies (lists, single records, ack strings, error
    /// details, etc.).
    #[schema(value_type = Object, nullable = true)]
    pub data: serde_json::Value,
}

/// Structured error body. Returned with non-2xx HTTP statuses; OpenAPI
/// annotations on routes reference this for the schema.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "error": "not_found",
    "message": "spec not found: introductions",
    "details": null
}))]
pub struct ErrorResponse {
    /// Machine-readable error code. Examples: `not_found`, `unauthorized`,
    /// `bad_request`, `conflict`, `rate_limited`, `internal_error`.
    pub error: String,
    /// Human-readable error message. Safe to surface in UIs and logs.
    pub message: String,
    /// Optional structured details. Shape varies by endpoint — see the
    /// individual operation for specifics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
