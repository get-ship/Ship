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
///
/// See the [errors reference](/reference/http-api/errors/) for the full
/// set of codes emitted by the daemon and their remediation.
// ErrorResponse moved to `crate::wire::ErrorResponse` (Phase 3 — auth
// needed to reference it without an `auth → routes` DAG violation).
// Re-exported from here so existing handler imports keep working.
pub use crate::wire::ErrorResponse;

/// Minimal acknowledgement returned by webhook ingress.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({ "ok": true }))]
pub struct WebhookAck {
    /// Always `true` when the trigger accepted the request.
    pub ok: bool,
}

/// Generic OK acknowledgement for endpoints whose only signal is success.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({ "ok": true }))]
pub struct GenericOk {
    pub ok: bool,
}

/// List of paths that must be advertised as public (no bearer required).
/// The `SecurityAndServer` modifier consults this table when finalizing
/// the generated spec.
pub const PUBLIC_PATHS: &[(&str, &str)] = &[
    ("get", "/api/health"),
    ("post", "/api/hooks/webhook/{path}"),
    ("get", "/api/v1/mcp/servers"),
    ("post", "/auth/pair/init"),
    ("get", "/auth/pair/status"),
    ("post", "/auth/server-attest"),
    ("get", "/"),
    ("get", "/setup"),
    ("get", "/api/v1/runtime/assets/datastar.js"),
    ("get", "/api/v1/runtime/assets/view-tokens.css"),
    ("get", "/api/v1/runtime/assets/ship-elements.js"),
    ("get", "/api/v1/runtime/assets/ship-code-editor.js"),
    ("get", "/api/v1/runtime/assets/ship-markdown-editor.js"),
    ("get", "/api/v1/runtime/assets/ship-terminal.js"),
    ("get", "/api/v1/runtime/assets/custom-elements.json"),
    ("get", "/api/v1/runtime/assets/ship-elements.md"),
    ("get", "/api/v1/runtime/assets/theme.css"),
    ("get", "/api/v1/runtime/themes"),
    ("get", "/api/openapi.json"),
];
