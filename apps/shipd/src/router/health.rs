//! Health check + panic-to-500 conversion.

use axum::{Json, body::Body, response::Response};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;

use crate::auth::unsafe_skip_auth;
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
            json!({"ok": false, "error": "internal server error"}).to_string(),
        ))
        .unwrap_or_else(|_| Response::new(Body::from("internal server error")))
}

/// Per-store disk-probe result for `/api/health`.
///
/// `state` is one of:
/// - `"ok"`          — `SELECT 1` round-tripped within busy_timeout.
/// - `"unavailable"` — no active space at boot (or the daemon degraded
///   space-less per R3 item 1); no probe was attempted. NOT a failure.
/// - `"error"`       — the probe FAILED — the file may be locked, corrupt,
///   permission-denied, or open exceeded busy_timeout. THIS flips the
///   top-level `healthy` flag to false.
#[derive(Clone)]
struct StoreProbe {
    state: &'static str,
    detail: Option<String>,
}

impl StoreProbe {
    fn ok() -> Self {
        Self {
            state: "ok",
            detail: None,
        }
    }
    fn unavailable() -> Self {
        Self {
            state: "unavailable",
            detail: None,
        }
    }
    fn err(detail: String) -> Self {
        Self {
            state: "error",
            detail: Some(detail),
        }
    }
}

/// Probe a single sqlite db: open a fresh `DataService`, run `SELECT 1`,
/// drop the connection. Intentionally per-probe rather than cached — a
/// fresh connection is the cheapest signal of "this file is reachable and
/// the schema is openable." WAL + busy_timeout(5s) bound the worst-case
/// latency. We never run `PRAGMA integrity_check` here — that's the
/// boot-time gate in `ShipRuntime::new`; the health endpoint stays cheap
/// on the hot path.
///
/// Runs via `spawn_blocking` because `DataService::query` internally calls
/// `crate::db::block_on` — keeping it off the executor prevents a slow
/// probe from stalling other requests.
async fn probe_db(path: &Path, label: &'static str) -> StoreProbe {
    let path = path.to_path_buf();
    let res = tokio::task::spawn_blocking({
        let label = label;
        move || {
            use ship_substrate::data::DataService;
            let mut svc = match DataService::open_at(&path) {
                Ok(s) => s,
                Err(e) => {
                    return Err(format!("open {label} ({path}): {e}", path = path.display()));
                }
            };
            svc.query("SELECT 1 AS ok", &[])
                .map(|_| ())
                .map_err(|e| format!("SELECT 1 {label} ({path}): {e}", path = path.display()))
        }
    })
    .await;

    match res {
        Ok(Ok(())) => StoreProbe::ok(),
        Ok(Err(detail)) => StoreProbe::err(detail),
        // spawn_blocking itself failed (panic, join error) — surface as
        // probe failure; the panic is already logged by the supervisor.
        Err(e) => StoreProbe::err(format!("probe task {label} join: {e}")),
    }
}

/// GET /api/health — unauthenticated health check.
///
/// Returns daemon uptime, PID, per-task supervisor health, AND a cheap
/// disk probe against `runtime.db` (always) and the active-space
/// `spaces/<id>/space.db` (when bound; `"unavailable"` otherwise).
/// Each store's probe runs `SELECT 1` against a fresh sqlite connection —
/// open + query + drop, bounded by the WAL busy_timeout(5s). A failed
/// probe flips the top-level `healthy` flag false and surfaces the
/// per-store `state: "error"` + `detail` so an operator can tell at a
/// glance which store wedged. Items 1+5: this is the surface that makes
/// the R3 space-less degradation visible — `space_db.state == "unavailable"`
/// means either no active space was resolved OR the `space.db` open at
/// boot failed (item 1's `tracing::error!` carries the cause).
pub(crate) async fn health_check(rt: Arc<ship_runtime::ShipRuntime>) -> Json<serde_json::Value> {
    let tasks = rt.health_registry().snapshot();
    let all_tasks_healthy = tasks.iter().all(|t| t.alive);

    let runtime_db_path = ship_substrate::db::runtime_db_path(rt.ship_dir());
    let runtime_probe = probe_db(&runtime_db_path, "runtime.db").await;

    let active_space_probe = match rt.active_space_id() {
        Some(space_id) => {
            let space_db_path = ship_substrate::db::space_db_path(rt.ship_dir(), &space_id);
            if space_db_path.exists() {
                probe_db(&space_db_path, "space.db").await
            } else {
                StoreProbe::unavailable()
            }
        }
        None => StoreProbe::unavailable(),
    };

    let stores_ok = matches!(runtime_probe.state, "ok")
        && matches!(active_space_probe.state, "ok" | "unavailable");
    let all_healthy = all_tasks_healthy && stores_ok;

    let ship_dir = rt
        .workspace_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| rt.ship_dir().display().to_string());
    let bi = build_info::build_info!();
    Json(json!({
        "ok": all_healthy,
        "healthy": all_healthy,
        "version": env!("CARGO_PKG_VERSION"),
        "commit": bi.short(),
        "build": bi.full(),
        "ship_dir": ship_dir,
        "pid": std::process::id(),
        "unsafe_skip_auth_mode": unsafe_skip_auth::is_unsafe_skip_auth_mode(),
        "stores": {
            "runtime_db": store_probe_to_json(&runtime_probe),
            "space_db": store_probe_to_json(&active_space_probe),
        },
        "tasks": tasks,
    }))
}

fn store_probe_to_json(p: &StoreProbe) -> serde_json::Value {
    let mut out = json!({ "state": p.state });
    if let Some(detail) = &p.detail {
        if let Some(map) = out.as_object_mut() {
            map.insert("detail".to_string(), json!(detail));
        }
    }
    out
}
