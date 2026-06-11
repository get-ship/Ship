//! Health check + panic-to-500 conversion.

use axum::{Json, body::Body, response::Response};
use std::sync::Arc;

use crate::auth::pairing;
use crate::ship_runtime;

/// Convert a handler panic into a structured 500 response instead of
/// aborting the connection. Logs the panic for diagnostics.
pub(crate) fn handle_panic(info: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let msg = if let Some(s) = info.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    };
    tracing::error!(panic = %msg, "request handler panicked");
    Response::builder()
        .status(500)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({"ok": false, "error": "internal server error"}).to_string(),
        ))
        .unwrap_or_else(|_| Response::new(Body::from("internal server error")))
}

/// GET /api/health — unauthenticated health check.
///
/// Returns daemon uptime, PID, and status of all supervised background tasks.
/// Any dead task sets the top-level `healthy` flag to false.
pub(crate) async fn health_check(rt: Arc<ship_runtime::ShipRuntime>) -> Json<serde_json::Value> {
    let tasks = rt.health_registry().snapshot();
    let all_healthy = tasks.iter().all(|t| t.alive);
    // Resolved project dir: workspace if attached, else the global dir
    // (which is what the runtime falls back to for global-only mode).
    let ship_dir = rt
        .workspace_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| rt.ship_dir().display().to_string());
    let bi = build_info::build_info!();
    Json(serde_json::json!({
        "ok": all_healthy,
        "healthy": all_healthy,
        "version": env!("CARGO_PKG_VERSION"),
        "commit": bi.short(),
        "build": bi.full(),
        "ship_dir": ship_dir,
        "pid": std::process::id(),
        "dev_mode": pairing::is_dev_mode(),
        "tasks": tasks,
    }))
}
