//! `/api/actors/*` — unified read/control surface for every running
//! actor (chat agent, pipeline step, shell, service).
//!
//! Read endpoints project from the `actors` table maintained by
//! `runtime::projections::ActorProjection`. Liveness derives from
//! `last_event_at` (bumped by SQLite triggers on every event correlated
//! to the actor) — no heartbeat loop. The input endpoint emits a
//! kernel event with `target_actor_id` set so directed delivery routes
//! it to the actor's mailbox without broadcast filtering.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use runtime::events::EventEnvelope;
use runtime::events::types::event_types;
use runtime::projections::actor::{
    ListActorsFilter, actor_status, get_actor, list_actor_events, list_actors,
};
use serde::Serialize;
use serde::Deserialize;
use serde_json::json;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize)]
pub struct ListQuery {
    pub workspace_id: Option<String>,
    pub kind: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

/// `GET /api/actors` — list all actors, ordered by recency. The
/// `live` field on each row is overlaid from the kernel's mailbox
/// map at query time — the mesh IS the source of truth for "alive
/// and reachable." If an actor is in `kernel.list_actors()`, you can
/// send to it and it will receive. If not, you can't. Period.
pub async fn list_actors_handler(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Query(q): Query<ListQuery>,
) -> impl IntoResponse {
    let filter = ListActorsFilter {
        workspace_id: q.workspace_id,
        kind: q.kind,
        status: q.status,
        limit: q.limit.unwrap_or(0),
    };
    let mut rows = match list_actors(&filter) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
            )
                .into_response();
        }
    };
    let on_mesh: std::collections::HashSet<String> = state
        .kernel()
        .lock()
        .await
        .list_actors()
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    for r in rows.iter_mut() {
        r.live = on_mesh.contains(&r.id);
    }
    (
        StatusCode::OK,
        Json(MeshResponse { ok: true, data: json!({ "actors": rows }) }),
    )
        .into_response()
}

#[derive(Serialize)]
pub struct MeshActor {
    pub actor_id: String,
    pub subscribe_namespaces: Vec<String>,
    pub write_namespaces: Vec<String>,
}

/// `GET /api/mesh` — every actor with a live mailbox in the kernel,
/// straight from `kernel.list_actors()`. This is the heartbeat: if
/// it's here, it's reachable. Used by the dashboard and by anyone
/// who needs to know "what can I send to right now."
pub async fn list_mesh_handler(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
) -> impl IntoResponse {
    let actors: Vec<MeshActor> = state
        .kernel()
        .lock()
        .await
        .list_actors()
        .into_iter()
        .map(|(id, cfg)| MeshActor {
            actor_id: id,
            subscribe_namespaces: cfg.subscribe_namespaces,
            write_namespaces: cfg.write_namespaces,
        })
        .collect();
    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: json!({ "count": actors.len(), "actors": actors }),
        }),
    )
        .into_response()
}

/// `GET /api/actors/{id}` — single actor detail.
pub async fn get_actor_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match get_actor(&id) {
        Ok(Some(a)) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: json!(a) }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: json!(format!("actor '{id}' not found")),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct EventsQuery {
    pub from_id: Option<String>,
    pub limit: Option<i64>,
}

/// `GET /api/actors/{id}/events?from_id=<event_id>` — replay events
/// correlated to the actor in causal order. Used by the SSE reconnect
/// path: client tracks last seen event id, asks for everything after.
pub async fn list_actor_events_handler(
    State(_state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Query(q): Query<EventsQuery>,
) -> impl IntoResponse {
    match list_actor_events(&id, q.from_id.as_deref(), q.limit.unwrap_or(0)) {
        Ok(events) => (
            StatusCode::OK,
            Json(MeshResponse { ok: true, data: json!({ "events": events }) }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
        )
            .into_response(),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ActorInputReq {
    pub text: String,
}

/// `GET /api/v1/mesh/snapshot` — single-shot bundle for the mesh view.
///
/// Returns:
/// - `actors`: every row from the projection, enriched with live process
///   stats (rss, threads, state, cpu_ticks) where the pid is alive
/// - `mesh`: the kernel router's view of what's currently mailboxable
///   (count + per-actor subscribe namespaces) — source of truth for
///   "can I reach this right now"
/// - `system`: daemon uptime, total event count, kernel actor count
///
/// One read for cold load. The view tails for diffs via SSE separately.
pub async fn mesh_snapshot_handler(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
) -> impl IntoResponse {
    use runtime::supervisor::process_stats;

    let mut rows = match list_actors(&ListActorsFilter::default()) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: json!(format!("query: {e}")) }),
            )
                .into_response();
        }
    };

    let kernel_actors = state.kernel().lock().await.list_actors();
    let on_mesh: std::collections::HashSet<String> =
        kernel_actors.iter().map(|(id, _)| id.clone()).collect();

    // Enrich each row with live OS stats. Best-effort; returns null on
    // any read miss (process exited, no pid, non-Linux).
    let enriched: Vec<serde_json::Value> = rows
        .iter_mut()
        .map(|r| {
            r.live = on_mesh.contains(&r.id);
            let stats = r
                .pid
                .and_then(|p| (p > 0).then(|| process_stats::read_pid(p as u32)).flatten());
            json!({
                "id": r.id,
                "kind": r.kind,
                "status": r.status,
                "workspace_id": r.workspace_id,
                "agent_id": r.agent_id,
                "provider": r.provider,
                "model": r.model,
                "transport": r.transport,
                "persistence": r.persistence,
                "pid": r.pid,
                "parent_actor_id": r.parent_actor_id,
                "restart_count": r.restart_count,
                "exit_code": r.exit_code,
                "duration_ms": r.duration_ms,
                "created_at": r.created_at,
                "last_event_at": r.last_event_at,
                "live": r.live,
                "process": stats.map(|s| json!({
                    "rss_bytes": s.rss_bytes,
                    "vsize_bytes": s.vsize_bytes,
                    "threads": s.threads,
                    "state": s.state.to_string(),
                    "cpu_ticks": s.cpu_ticks,
                })),
            })
        })
        .collect();

    let mesh: Vec<MeshActor> = kernel_actors
        .into_iter()
        .map(|(id, cfg)| MeshActor {
            actor_id: id,
            subscribe_namespaces: cfg.subscribe_namespaces,
            write_namespaces: cfg.write_namespaces,
        })
        .collect();

    let system = json!({
        "uptime_secs": state.uptime().as_secs(),
        "kernel_actor_count": mesh.len(),
        "projection_actor_count": enriched.len(),
        "live_process_count": enriched
            .iter()
            .filter(|a| a.get("process").map(|p| !p.is_null()).unwrap_or(false))
            .count(),
    });

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: json!({
                "actors": enriched,
                "mesh": mesh,
                "system": system,
            }),
        }),
    )
        .into_response()
}

/// `POST /api/v1/actors/{id}/kill-tree` — stop the actor and every
/// descendant by `parent_actor_id`. Walks the projection to collect
/// the tree, stops each actor via the appropriate registry (actor
/// then legacy session), and reports counts.
pub async fn kill_tree_handler(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshSend) {
        return e.into_response();
    }

    // BFS over parent_actor_id. Cap depth to avoid pathological loops
    // (parent cycles shouldn't happen but defense-in-depth).
    let mut to_visit: Vec<String> = vec![id.clone()];
    let mut targets: Vec<String> = Vec::new();
    let mut depth = 0usize;
    while !to_visit.is_empty() && depth < 32 {
        let mut next: Vec<String> = Vec::new();
        for actor_id in to_visit.drain(..) {
            if targets.contains(&actor_id) {
                continue;
            }
            targets.push(actor_id.clone());
            // Find children: actors whose parent_actor_id == this id.
            let children = list_actors(&ListActorsFilter::default())
                .unwrap_or_default()
                .into_iter()
                .filter(|r| r.parent_actor_id.as_deref() == Some(&actor_id))
                .map(|r| r.id)
                .collect::<Vec<_>>();
            next.extend(children);
        }
        to_visit = next;
        depth += 1;
    }

    // Stop deepest first so children unwind before parents notice.
    let mut stopped: Vec<String> = Vec::new();
    for actor_id in targets.iter().rev() {
        let mut hit = false;
        if let Some(entry) = state.actor_sessions().get(actor_id) {
            let _ = entry.stop();
            state.actor_sessions().remove(actor_id);
            hit = true;
        }
        if !hit {
            // Fall through to the legacy session registry.
            if crate::sessions::lifecycle::stop(state.pty_sessions(), actor_id) {
                hit = true;
            }
        }
        if hit {
            stopped.push(actor_id.clone());
        }
    }

    (
        StatusCode::OK,
        Json(MeshResponse {
            ok: true,
            data: json!({
                "root": id,
                "tree": targets,
                "stopped": stopped,
            }),
        }),
    )
        .into_response()
}

/// `POST /api/actors/{id}/input` — push a turn into the actor's mailbox
/// via directed-delivery on `studio.chat_turn`. 404 if the actor isn't
/// in the projection; 409 if it's stopped or crashed.
pub async fn actor_input_handler(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(req): Json<ActorInputReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshSend) {
        return e.into_response();
    }
    let text = req.text.trim().to_string();
    if text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MeshResponse { ok: false, data: json!("empty text") }),
        )
            .into_response();
    }

    // Source of truth for "can I send to this": the kernel's mailbox
    // map. If the actor isn't there RIGHT NOW, no message will reach
    // it. The projection's status field can lag (events not yet
    // applied, or stale rows from old runs); kernel state is now.
    let on_mesh = state.kernel().lock().await.has_actor(&id);
    if !on_mesh {
        return (
            StatusCode::NOT_FOUND,
            Json(MeshResponse {
                ok: false,
                data: json!({
                    "error": "actor not on mesh",
                    "hint": "the actor isn't currently registered in the kernel — \
                            it may have exited, crashed, or never registered",
                    "actor_id": id,
                }),
            }),
        )
            .into_response();
    }
    // Workspace_id only used for context tagging. Best-effort lookup.
    let workspace_id = actor_status(&id).ok().flatten().and_then(|(_, w)| w);

    let payload = json!({ "actor_id": id, "text": text });
    let envelope = match EventEnvelope::new(event_types::STUDIO_CHAT_TURN, &id, &payload) {
        Ok(e) => e
            .with_context(workspace_id.as_deref(), Some(&id))
            .with_target(&id),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MeshResponse { ok: false, data: json!(format!("envelope: {e}")) }),
            )
                .into_response();
        }
    };
    let ctx = auth.to_daemon_emit_context();
    if let Err(e) = state.kernel().lock().await.emit(envelope, &ctx).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MeshResponse { ok: false, data: json!(format!("emit: {e}")) }),
        )
            .into_response();
    }
    (StatusCode::ACCEPTED, Json(MeshResponse { ok: true, data: json!({}) }))
        .into_response()
}
