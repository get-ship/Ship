//! Shared OpenAPI schema types referenced by generated operation responses.
//!
//! `ErrorResponse` is the canonical 4xx/5xx body for every authenticated
//! endpoint. The modifier in `openapi.rs` injects references to it on every
//! operation at spec-generation time so individual `#[utoipa::path]`
//! annotations don't have to repeat them.

use serde::Serialize;
use utoipa::ToSchema;

/// Canonical error body returned by non-2xx responses.
///
/// Handlers today emit `{"ok": false, "error": "..."}` or
/// `{"ok": false, "data": "..."}`. The spec standardizes on this richer
/// shape so SDK consumers can branch on a machine-readable `error` code
/// without parsing the human-readable message.
#[derive(Debug, Serialize, ToSchema)]
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

/// Minimal acknowledgement returned by webhook ingress.
#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookAck {
    /// Always `true` when the trigger accepted the request.
    pub ok: bool,
}

/// Generic OK acknowledgement for endpoints whose only signal is success.
#[derive(Debug, Serialize, ToSchema)]
pub struct GenericOk {
    pub ok: bool,
}

/// List of paths that must be advertised as public (no bearer required).
/// The `SecurityAndServer` modifier consults this table when finalizing
/// the generated spec.
pub const PUBLIC_PATHS: &[(&str, &str)] = &[
    ("get", "/api/health"),
    ("post", "/api/hooks/webhook/{path}"),
    ("get", "/api/mcp/servers"),
    ("post", "/auth/pair/init"),
    ("get", "/auth/pair/status"),
    ("post", "/auth/server-attest"),
    ("get", "/"),
    ("get", "/setup"),
    ("get", "/api/runtime/assets/datastar.js"),
    ("get", "/api/runtime/assets/view-tokens.css"),
    ("get", "/api/runtime/assets/ship-elements.js"),
    ("get", "/api/runtime/assets/ship-code-editor.js"),
    ("get", "/api/runtime/assets/ship-markdown-editor.js"),
    ("get", "/api/runtime/assets/ship-terminal.js"),
    ("get", "/api/runtime/assets/custom-elements.json"),
    ("get", "/api/runtime/assets/ship-elements.md"),
    ("get", "/api/runtime/assets/theme.css"),
    ("get", "/api/runtime/themes"),
    ("get", "/api/openapi.json"),
];
