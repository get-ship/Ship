//! `view.*` MCP tool handlers — thin transport over the daemon's
//! `/api/v1/views/{name}/edit/tool` dispatch endpoint.
//!
//! Each handler:
//! 1. Reads the calling session_id from the MCP server connection state.
//! 2. POSTs `{ session_id, op, args }` to the daemon.
//! 3. Returns the daemon response (or formatted error) verbatim.
//!
//! Capability enforcement, snapshot atomicity, and SSE publishing live on
//! the daemon side — the MCP layer is transport only.

use crate::network_client;
use crate::requests::view_authoring::{
    ViewAnnotate, ViewCollectionsSchema, ViewListSnapshotsArgs, ViewName, ViewNoArgs,
    ViewRestoreSnapshotArgs, ViewWriteSource,
};

fn missing_session_err() -> String {
    "Error: no active session — call `start_session` first".to_string()
}

async fn dispatch(view: &str, session_id: Option<&str>, op: &str, args: serde_json::Value) -> String {
    let Some(session_id) = session_id else {
        return missing_session_err();
    };
    match network_client::view_edit_tool(view, session_id, op, args).await {
        Ok(body) => body,
        Err(e) => format!("Error: {e}"),
    }
}

pub async fn view_read_source(session_id: Option<&str>, req: ViewName) -> String {
    dispatch(&req.view, session_id, "read_source", serde_json::json!({})).await
}

pub async fn view_write_source(session_id: Option<&str>, req: ViewWriteSource) -> String {
    let args = serde_json::json!({
        "source": req.source,
        "turn_label": req.turn_label,
    });
    dispatch(&req.view, session_id, "write_source", args).await
}

pub async fn view_compile_check(session_id: Option<&str>, req: ViewName) -> String {
    dispatch(&req.view, session_id, "compile_check", serde_json::json!({})).await
}

pub async fn view_collections_schema(
    session_id: Option<&str>,
    req: ViewCollectionsSchema,
    active_view: Option<&str>,
) -> String {
    // collections_schema isn't view-bound but the dispatch endpoint is —
    // route through the currently-active view (any view the session has a
    // grant for works). Falls back to `__any__` when nothing is active so
    // the daemon returns a 403 rather than a misleading 404.
    let view = active_view.unwrap_or("__any__");
    let args = serde_json::json!({ "id": req.id });
    dispatch(view, session_id, "collections_schema", args).await
}

pub async fn view_list_siblings(
    session_id: Option<&str>,
    _req: ViewNoArgs,
    active_view: &str,
) -> String {
    dispatch(active_view, session_id, "list_siblings", serde_json::json!({})).await
}

pub async fn view_theme_tokens(
    session_id: Option<&str>,
    _req: ViewNoArgs,
    active_view: &str,
) -> String {
    dispatch(active_view, session_id, "theme_tokens", serde_json::json!({})).await
}

pub async fn view_preview_snapshot(session_id: Option<&str>, req: ViewName) -> String {
    dispatch(&req.view, session_id, "preview_snapshot", serde_json::json!({})).await
}

pub async fn view_annotate(session_id: Option<&str>, req: ViewAnnotate) -> String {
    let args = serde_json::json!({
        "selector": req.selector,
        "note": req.note,
        "kind": req.kind,
    });
    dispatch(&req.view, session_id, "annotate", args).await
}

pub async fn view_list_snapshots(session_id: Option<&str>, req: ViewListSnapshotsArgs) -> String {
    let args = serde_json::json!({
        "limit": req.limit,
        "cursor": req.cursor,
    });
    dispatch(&req.view, session_id, "list_snapshots", args).await
}

pub async fn view_restore_snapshot(session_id: Option<&str>, req: ViewRestoreSnapshotArgs) -> String {
    let args = serde_json::json!({
        "snapshot_id": req.snapshot_id,
    });
    dispatch(&req.view, session_id, "restore_snapshot", args).await
}
