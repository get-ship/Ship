//! The enumerated `/api/v1/*` route group, bound via `ShipRouterExt`.

use axum::{Router, routing};

#[cfg(feature = "ship-code")]
use crate::sessions;
use crate::routes::{artifacts_api, sandbox_api};
use crate::ship_runtime::ApiState;

/// All enumerated routes, registered via `ShipRouterExt::ship_route` against
/// the canonical `Route` enum. Paths come from `routes/mod.rs` — this function
/// only binds handlers.
pub(super) fn build_ship_routes(st: ApiState) -> Router {
    use crate::routes::{Route, ShipRouterExt};
    // Base routes: always compiled regardless of feature flags.
    // Note: optionB's actor_spawn / list_actors / daemon_status / serve_schema
    // / workspace_pty route bindings were removed — the corresponding Route
    // variants are gone from the canonical `Route` enum kept here, and the
    // handler modules were deleted by the harness team. The op surface they
    // implemented is reachable exclusively over ConnectRPC (ship.ops.v1).
    let router = Router::new()
        // Isolation reads are served by the proto IsolationService
        // (ship.ops.v1) — the one op-surface path.
        .ship_route(Route::SandboxList, sandbox_api::sandbox_list)
        .ship_route(Route::SandboxGet_template(), sandbox_api::sandbox_get)
        // The `/_deck/mutation/{name}` 410 tombstone moved to `deck::router()`
        // — it's a `/_deck/*` serving concern (deck composes that form `action`
        // URL via `deck_core::routes::mutation_action`), not an RPC route.
        // AgentsService / PipelineService / RuleService / PermissionService /
        // SkillService — all Connect wires (ship.ops.v1); no REST shells here.
        // Events / projections — the kernel event bus is served over Connect
        // `StreamService.Events` (ship.ops.v1); the SSE route was retired.

        // Artifacts platform (three-scope)
        .merge(
            Router::<ApiState>::new()
                .ship_route(Route::ArtifactsUpload, artifacts_api::upload_artifact)
                .layer(axum::extract::DefaultBodyLimit::max(
                    artifacts_api::MAX_UPLOAD_BYTES,
                )),
        )
        .ship_route(Route::ArtifactDownload_template(), artifacts_api::download_artifact)
        .route(
            "/api/v1/artifacts/{id}",
            routing::head(artifacts_api::head_artifact),
        )
        .ship_route(Route::ArtifactMetadata_template(), artifacts_api::artifact_metadata)
;

    // Sessions (PTY) — harness-driver stack; only compiled with ship-code.
    #[cfg(feature = "ship-code")]
    let router = router
        .ship_route(Route::SpawnPtySession, sessions::api::spawn_session)
        .ship_route(Route::ListPtySessions, sessions::api::list_pty_sessions)
        .ship_route(Route::GetPtySession_template(), sessions::api::get_session)
        .ship_route(Route::StopPtySession_template(), sessions::api::stop_session)
        .ship_route(Route::PtySessionInput_template(), sessions::api::session_input)
        .ship_route(Route::PtySessionResize_template(), sessions::api::session_resize)
        .ship_route(Route::PtySessionMessage_template(), sessions::api::session_message)
        .ship_route(Route::PtySessionReply_template(), sessions::api::session_reply)
        .ship_route(Route::StreamPtySession_template(), sessions::api::stream_session)
        .ship_route(Route::StreamPtySessionEvents_template(), sessions::api::stream_session_events)
        .ship_route(Route::InstalledProviders, sessions::api::installed_harnesses);

    router.with_state(st)
}
