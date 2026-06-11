//! Shared wire types — request/response shapes used by every layer of
//! the daemon. Lives below `auth` in the module DAG so auth handlers
//! can reference response shapes without an upward `auth → routes`
//! import. See `scripts/lint/check-module-dag.sh` for the constraint.

use serde::Serialize;

/// Envelope returned by every handler under `/api/v1/*` that isn't a
/// raw bytes / SSE / stream response. `ok` flags success; `data` is
/// either the payload or a structured error description.
#[derive(Debug, Serialize)]
pub struct MeshResponse {
    /// `true` on success; `false` when the handler returned a structured error.
    pub ok: bool,
    /// Response payload; shape varies by endpoint. See individual operation
    /// responses for concrete shapes. Represented as arbitrary JSON because
    /// this envelope is reused across dozens of handlers that return
    /// heterogeneous bodies (lists, single records, ack strings, error
    /// details, etc.).
    pub data: serde_json::Value,
}

