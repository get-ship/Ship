//! Actor spawning and event routing endpoints.
//!
//! These generalize mesh operations so the MCP process can delegate
//! actor lifecycle and event routing to the daemon's shared kernel.
//!
//! `actor_spawn` is register-or-refine: if the kernel doesn't yet have
//! an actor with this id, it's registered fresh (the historical path,
//! for agents launched outside substrate). If the kernel already has
//! one — substrate pre-registers every actor it spawns, including
//! agents — its config is refined in place via `KernelRouter::refine_actor`.
//! The mailbox sender is preserved across refinement, so any Mailbox
//! a caller holds (or that substrate stashed for SSE drain) stays
//! valid.

use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use ship_substrate::events::{EmitContext, EventEnvelope};
use serde::Deserialize;

use crate::auth::{AuthContext, Scope, require_scope};
use crate::routes::rest_api::{ApiState, MeshResponse};

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "actor_id": "agent-writer",
    "config": {
        "kind": "agent",
        "harness": "claude",
        "spec_path": ".ship/specs/write-readme.md"
    },
    "capabilities": ["writes.prose", "reads.repo"]
}))]
pub struct ActorSpawnReq {
    /// Optional client-suggested id. When present, must be a ULID
    /// (used by daemon-spawned agents that pre-mint via `gen_ulid`
    /// and propagate via SHIP_ACTOR_ID env). When absent, the daemon
    /// mints a fresh ULID — the canonical path. Synthesis like
    /// `agent.{name}.{branch}` is gone; the daemon is the only id
    /// minter.
    #[serde(default)]
    pub actor_id: Option<String>,
    #[schema(value_type = Object)]
    pub config: ship_substrate::events::ActorConfig,
    /// Optional mesh capabilities for auto-registration.
    pub capabilities: Option<Vec<String>>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "envelope": {
        "kind": "job.completed",
        "slug": "write-readme",
        "ts": "2026-04-22T10:15:00Z"
    },
    "workspace_id": "default",
    "session_id": "sess-2f9c"
}))]
pub struct EventRouteReq {
    #[schema(value_type = Object)]
    pub envelope: ship_substrate::events::EventEnvelope,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
}

#[utoipa::path(
    operation_id = "actorSpawn",
    post,
    path = "/api/v1/actor/spawn",
    request_body = ActorSpawnReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::wire::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 404, description = "actor not found", body = crate::wire::ErrorResponse),
        (status = 409, description = "actor already exists", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "actor"
)]
pub async fn actor_spawn(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<ActorSpawnReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::MeshSend) {
        return e;
    }

    // Resolve the actor id. Daemon is the only minter — clients
    // either send the ULID a daemon-spawn already minted (propagated
    // via SHIP_ACTOR_ID env), or omit it and we mint fresh. Any
    // other shape (synthesized `agent.X.Y` from old code paths, or
    // arbitrary strings) is accepted only as a backwards-compat
    // bridge and logged so we can clean up call sites.
    let actor_id = match req.actor_id.as_deref() {
        Some(id) if !id.is_empty() => {
            if !is_ulid_shape(id) {
                tracing::warn!(
                    actor_id = id,
                    "actor_spawn: client sent non-ULID id; backwards-compat path"
                );
            }
            id.to_string()
        }
        _ => ship_substrate::gen_ulid(),
    };

    {
        let mut kernel = state.kernel().lock().await;
        // Register-or-refine. If the actor isn't in the kernel yet
        // (e.g. agent launched outside substrate), spawn it fresh
        // with the caller's config — same as before. If it is already
        // registered (substrate pre-registered it, or the rmcp
        // framework is calling on_initialized twice), refine the
        // existing config in place: the kernel sender — and therefore
        // any Mailbox handle a caller is holding — stays valid.
        if !kernel.has_actor(&actor_id) {
            match kernel.spawn_actor(&actor_id, req.config) {
                Ok((_store, mailbox)) => {
                    state.agent_mailboxes().lock().await
                        .insert(actor_id.clone(), mailbox);
                }
                Err(e) => {
                    let status = match &e {
                        ship_substrate::errors::KernelError::ActorAlreadyExists { .. } => StatusCode::CONFLICT,
                        ship_substrate::errors::KernelError::ActorNotFound { .. } => StatusCode::NOT_FOUND,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    return (status, Json(MeshResponse {
                        ok: false, data: serde_json::json!(e.to_string()),
                    }));
                }
            }
        } else if let Err(e) = kernel.refine_actor(&actor_id, req.config) {
            let status = match &e {
                ship_substrate::errors::KernelError::ActorNotFound { .. } => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            return (status, Json(MeshResponse {
                ok: false, data: serde_json::json!(e.to_string()),
            }));
        }
    }

    // After refinement (or first registration), promote any mailbox
    // substrate stashed for this actor into the SSE-drain map. This
    // covers the case where substrate pre-registered the agent and
    // dropped its Mailbox into the stash; the MCP child's eventual
    // SSE connect on `/mesh/events/{actor_id}` consumes from there.
    if let Some(mailbox) = ship_substrate::supervisor::take_pre_registered_mailbox(&actor_id) {
        state
            .agent_mailboxes()
            .lock()
            .await
            .entry(actor_id.clone())
            .or_insert(mailbox);
    }

    if let Some(ref caps) = req.capabilities {
        // Directed at the registering actor so ONLY that actor's tui
        // sees the join confirmation. Self-confirmation, not broadcast.
        let envelope = match EventEnvelope::new(
            "mesh.register",
            &actor_id,
            &serde_json::json!({ "agent_id": &actor_id, "capabilities": caps }),
        )
        .map(|e| e.with_actor_id(&actor_id).with_target(&actor_id))
        {
            Ok(e) => e,
            Err(e) => return (StatusCode::BAD_REQUEST, Json(MeshResponse {
                ok: false, data: serde_json::json!(e.to_string()),
            })),
        };
        let ctx = auth.to_daemon_emit_context();
        if let Err(e) = state.kernel().lock().await.emit(envelope, &ctx).await {
            tracing::warn!(actor_id = %actor_id, "mesh.register emit failed: {e}");
        }
    }

    // Emit `actor.created` only if the projection has no row for this
    // id yet — i.e., the actor was NOT spawned by the daemon's own
    // lifecycle path (which emits its own rich metadata).
    let emit_actor_id = actor_id.clone();
    let capabilities = req.capabilities.clone().unwrap_or_default();
    std::thread::spawn(move || {
        let already_exists = ship_substrate::projections::actor::get_actor(&emit_actor_id)
            .map(|opt| opt.is_some())
            .unwrap_or(false);
        if already_exists {
            return;
        }
        let payload = ship_substrate::events::types::ActorCreated {
            kind: "agent".to_string(),
            environment_type: "local".to_string(),
            transport: Some("stdio".to_string()),
            persistence: Some("long-lived".to_string()),
            capabilities,
            ..Default::default()
        };
        if let Err(e) = ship_substrate::db::actor_events::emit_actor_created(
            &emit_actor_id,
            &payload,
            None,
            None,
        ) {
            tracing::warn!(actor_id = %emit_actor_id, "actor.created emit failed: {e}");
        }
    });

    // Return the canonical id the daemon registered the actor under.
    // Clients (the MCP child) MUST adopt this for SHIP_ACTOR_ID env
    // and any subsequent calls. Synthesis fallback is dead.
    (StatusCode::OK, Json(MeshResponse {
        ok: true,
        data: serde_json::json!({ "actor_id": actor_id }),
    }))
}

/// Crockford-base32 ULID — 26 chars from the alphabet
/// `0123456789ABCDEFGHJKMNPQRSTVWXYZ`. The daemon mints these via
/// `ship_substrate::gen_ulid()`; clients send them back verbatim.
fn is_ulid_shape(s: &str) -> bool {
    s.len() == 26 && s.bytes().all(|b| {
        matches!(b,
            b'0'..=b'9' | b'A'..=b'H' | b'J' | b'K' | b'M'..=b'N' |
            b'P'..=b'T' | b'V'..=b'Z'
        )
    })
}

#[utoipa::path(
    operation_id = "eventRoute",
    post,
    path = "/api/v1/events/route",
    request_body = EventRouteReq,
    responses(
        (status = 200, body = MeshResponse),
        (status = 400, description = "bad request", body = crate::wire::ErrorResponse),
        (status = 401, description = "unauthorized", body = crate::wire::ErrorResponse),
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    tag = "actor"
)]
pub async fn event_route(
    State(state): State<ApiState>,
    Extension(auth): Extension<AuthContext>,
    Json(req): Json<EventRouteReq>,
) -> impl IntoResponse {
    if let Err(e) = require_scope(&auth, Scope::EmitEvents) {
        return e;
    }
    // MCP tools route through the daemon's HTTP API. The MCP server is a
    // trusted Ship component — it validates caller permissions before emitting.
    // Use the authenticated caller kind but allow CLI callers (default) to route
    // system-namespace events (the bridge has already validated permissions).
    let ctx = EmitContext {
        caller_kind: auth.caller_kind,
        skill_id: None,
        project_id: req.workspace_id.or(auth.workspace_id),
        session_id: req.session_id.or(auth.session_id),
    };
    match state.kernel().lock().await.emit(req.envelope, &ctx).await {
        Ok(()) => (StatusCode::OK, Json(MeshResponse { ok: true, data: serde_json::json!("emitted") })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(MeshResponse { ok: false, data: serde_json::json!(e.to_string()) })),
    }
}
