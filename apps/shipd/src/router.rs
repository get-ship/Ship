//! Axum router assembly: public + authenticated route groups, enumerated ship routes,
//! panic handler, and health check.

use axum::{
    Json, Router,
    body::Body,
    response::Response,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use utoipa::OpenApi;

use crate::{
    NetworkServer, actor_api, artifacts_api, attest, auth, cors, feedback_api, hooks_api,
    human_webhook, openapi, pairing, project_api, pty_handler, rest_api, runtime_api, session,
    settings_api, shell_assets, ship_runtime, specs_api, specs_view, supervisor,
    view_morph_target, view_sessions_api, views, collection_api, views_agents, views_chat,
    views_events, views_git, views_pipelines, views_raw, views_settings, views_skills,
};

/// Build the full Axum router with all route groups, auth, and CORS.
pub fn build_router(
    rt: Arc<ship_runtime::ShipRuntime>,
    token: Arc<String>,
    session_registry: auth::SessionRegistry,
    pairing_registry: pairing::SharedPairingRegistry,
    attestation_verifier: attest::AttestationVerifier,
    mcp_ct: CancellationToken,
    with_ui: bool,
) -> Router {
    use crate::routes::{Route, ShipRouterExt};
    let kernel_factory = rt.kernel().clone();
    let service: StreamableHttpService<NetworkServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(NetworkServer::new(kernel_factory.clone())),
            Default::default(),
            StreamableHttpServerConfig {
                cancellation_token: mcp_ct,
                sse_retry: None,
                sse_keep_alive: None,
                ..Default::default()
            },
        );

    let st = rt.clone();
    // Non-enumerated third-party routes (telegram webhook + hooks) stay on
    // `.route()` — they predate the canonical `Route` enum. Leave flat.
    let webhook_routes = Router::new()
        .route("/webhook/telegram", axum::routing::post(human_webhook::telegram_webhook))
        .with_state(rt.kernel().clone());
    let hook_routes = Router::new()
        .route("/hooks/checkout", axum::routing::post(rest_api::hooks_checkout))
        .route("/hooks/session-start", axum::routing::post(hooks_api::hook_session_start))
        .route("/hooks/before-tool", axum::routing::post(hooks_api::hook_before_tool))
        .route("/hooks/after-tool", axum::routing::post(hooks_api::hook_after_tool))
        .route("/hooks/session-end", axum::routing::post(hooks_api::hook_session_end));
    // Enumerated routes — one `ApiState`-typed router, registered via
    // `ship_route(Route::X, handler)` against the canonical full paths.
    let api_ship_routes = build_ship_routes(st);

    // Unauthenticated routes (before auth middleware)
    let register_registry = session_registry.clone();
    let health_rt = rt.clone();
    let pair_init_reg = pairing_registry.clone();
    let pair_init_sessions = session_registry.clone();
    let pair_status_reg = pairing_registry.clone();
    let attest_verifier = attestation_verifier.clone();
    let attest_sessions = session_registry.clone();
    let public_routes = Router::new()
        .route(
            "/auth/pair/init",
            axum::routing::post(move |body: Option<axum::Json<auth::PairInitRequest>>| {
                auth::pair_init(pair_init_reg.clone(), pair_init_sessions.clone(), body)
            }),
        )
        .route(
            "/auth/pair/status",
            axum::routing::get(move |q: axum::extract::Query<auth::PairStatusQuery>| {
                auth::pair_status(pair_status_reg.clone(), q)
            }),
        )
        .route(
            "/auth/server-attest",
            axum::routing::post(move |body: axum::Json<auth::AttestRequest>| {
                auth::server_attest(attest_verifier.clone(), attest_sessions.clone(), body)
            }),
        )
        .route("/api/health", axum::routing::get(move || {
            health_check(health_rt)
        }))
        // Shared static assets — no user-specific data, safe to serve unauth.
        .ship_route(Route::AssetDatastarJs, views::serve_datastar_js)
        .ship_route(Route::AssetViewTokensCss, views::serve_view_tokens)
        .ship_route(Route::AssetShipElementsJs, views::serve_ship_elements)
        .ship_route(Route::AssetCodeEditorJs, views::serve_code_editor)
        .ship_route(Route::AssetMarkdownEditorJs, views::serve_markdown_editor)
        .ship_route(Route::AssetTerminalJs, views::serve_terminal)
        .ship_route(Route::AssetElementsManifest, views::serve_elements_manifest)
        .ship_route(Route::AssetElementsCatalog, views::serve_elements_catalog)
        .ship_route(Route::AssetThemeCss, views::serve_theme_css)
        .ship_route(Route::ListThemes, views::serve_themes_list);

    let auth_token = token;
    let auth_registry = session_registry;
    // Authenticated pair confirm + session management — require daemon.token bearer.
    let pair_confirm_reg = pairing_registry.clone();
    let pair_confirm_sessions = auth_registry.clone();
    let list_sessions_registry = auth_registry.clone();
    let revoke_sessions_registry = auth_registry.clone();
    let pair_confirm_routes = Router::new()
        .route(
            "/auth/pair/confirm",
            axum::routing::post(move |auth_ext: axum::Extension<auth::AuthContext>, body: axum::Json<auth::PairConfirmRequest>| {
                auth::pair_confirm(
                    pair_confirm_reg.clone(),
                    pair_confirm_sessions.clone(),
                    auth_ext,
                    body,
                )
            }),
        )
        .route(
            "/auth/sessions",
            axum::routing::get(move |auth_ext: axum::Extension<auth::AuthContext>| {
                auth::list_sessions(list_sessions_registry.clone(), auth_ext)
            }),
        )
        .route(
            "/auth/sessions/revoke",
            axum::routing::post(move |auth_ext: axum::Extension<auth::AuthContext>, body: axum::Json<auth::RevokeSessionsRequest>| {
                auth::revoke_sessions(revoke_sessions_registry.clone(), auth_ext, body)
            }),
        )
        .route("/api/session/register", axum::routing::post(move |
            auth_ext: axum::Extension<auth::AuthContext>,
            body: axum::Json<auth::SessionRegisterRequest>,
        | {
            auth::session_register(register_registry, auth_ext, body)
        }));

    let shell_registry = auth_registry.clone();
    let session_issue_registry = auth_registry.clone();
    let session_issue_handler = move |auth: axum::Extension<auth::AuthContext>| {
        let reg = session_issue_registry.clone();
        async move { session::issue_session_handler(reg, auth.0).await }
    };

    let feedback_guard = feedback_api::FeedbackGuard::new();
    let authenticated_routes = Router::new()
        .nest("/api", pair_confirm_routes)
        .ship_route(Route::IssueSession, session_issue_handler)
        .merge(api_ship_routes)
        .nest("/api", hook_routes)
        .nest("/api", webhook_routes)
        .nest_service("/mcp", service)
        .layer(axum::Extension(feedback_guard))
        .layer(axum::middleware::from_fn(move |req, next| {
            auth::auth_middleware(auth_token.clone(), auth_registry.clone(), req, next)
        }));

    let mut root = Router::new()
        .merge(public_routes)
        .route("/api/openapi.json", axum::routing::get(|| async {
            Json(openapi::ApiDoc::openapi())
        }))
        .merge(authenticated_routes);
    if with_ui {
        // Shell routes are registered OUTSIDE the auth middleware — the
        // initial `/shell/` GET is what mints the browser cookie that
        // everything else uses. Put before any catch-all.
        root = root.merge(shell_assets::router(shell_registry));
    }
    root
        .layer(axum::middleware::from_fn(cors::cors_middleware))
        // Tag view responses with Datastar morph-target headers so cross-view
        // navigation inside the shell doesn't wipe shell chrome.
        .layer(axum::middleware::from_fn(view_morph_target::view_morph_target_middleware))
        // Outer request body cap. 50 MB matches the files-API upload limit.
        // JSON / Form extractors still enforce axum's `DefaultBodyLimit`
        // (2 MB) on their own — this layer only sets the outer ceiling.
        .layer(tower_http::limit::RequestBodyLimitLayer::new(
            crate::artifacts_api::MAX_UPLOAD_BYTES,
        ))
        // Catch handler panics → 500 instead of connection abort.
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(handle_panic))
}

/// All enumerated routes, registered via `ShipRouterExt::ship_route` against
/// the canonical `Route` enum. Paths come from `routes.rs` — this function
/// only binds handlers.
///
/// Two preserved non-enum `.route()` paths:
/// - `/api/workspaces/{id}/session-files/{*path}` — glob param, specific to
///   project_api's recursive file handler. Kept alongside the enum variant
///   that exposes the non-glob listing endpoint.
/// - `/api/workspaces/{id}/skills/{skillId}/files/{*path}` — same reason.
fn build_ship_routes(st: rest_api::ApiState) -> Router {
    use crate::routes::{Route, ShipRouterExt};
    Router::new()
        // Mesh / actors
        .ship_route(Route::MeshRegister, rest_api::mesh_register)
        .ship_route(Route::MeshSend, rest_api::mesh_send)
        .ship_route(Route::MeshBroadcast, rest_api::mesh_broadcast)
        .ship_route(Route::MeshDiscover, rest_api::mesh_discover)
        .ship_route(Route::MeshStatusUpdate, rest_api::mesh_status_update)
        .ship_route(Route::MeshEvents_template(), rest_api::mesh_events)
        .ship_route(Route::ActorSpawn, actor_api::actor_spawn)
        .ship_route(Route::EventRoute, actor_api::event_route)
        // Capabilities (no state required; ok against a stateful router)
        .ship_route(Route::Capabilities, rest_api::capabilities)
        // Runtime
        .ship_route(Route::ListWorkspaces, runtime_api::list_workspaces)
        .ship_route(Route::ListSessions, runtime_api::list_sessions)
        .ship_route(Route::ListAgents, runtime_api::list_agents)
        .ship_route(Route::StopAgent_template(), runtime_api::stop_agent)
        .ship_route(Route::SteerAgent_template(), runtime_api::steer_agent)
        .ship_route(Route::ListJobs, runtime_api::list_jobs)
        .ship_route(Route::CreateJob, runtime_api::create_job)
        .ship_route(Route::GetJob_template(), runtime_api::get_job_detail)
        .ship_route(Route::EndJob, runtime_api::end_job)
        .ship_route(Route::CancelJob_template(), runtime_api::cancel_job)
        .ship_route(Route::ListProcs, runtime_api::list_procs)
        .ship_route(Route::PollEvents, runtime_api::poll_events)
        .ship_route(Route::DaemonStatus, runtime_api::daemon_status)
        // Views
        .ship_route(Route::ListViews, views::list_views)
        .ship_route(Route::ServeView_template(), views::serve_view)
        .ship_route(Route::ServeViewManifest_template(), views::serve_view_manifest)
        .ship_route(Route::ServeViewBundle_template(), views::serve_view_bundle)
        .ship_route(Route::ServeViewRaw_template(), views_raw::serve_view_raw)
        .ship_route(Route::ServeViewSse_template(), views::generic_sse::serve_view_sse)
        // Agent-owned view sessions
        .ship_route(Route::OpenViewSession, view_sessions_api::open_session)
        .ship_route(Route::PatchViewSession_template(), view_sessions_api::patch_session)
        .ship_route(Route::AppendViewSession_template(), view_sessions_api::append_session)
        .ship_route(Route::SignalViewSession_template(), view_sessions_api::signal_session)
        .ship_route(Route::CloseViewSession_template(), view_sessions_api::close_session)
        .ship_route(Route::SubscribeViewSession_template(), view_sessions_api::subscribe_session)
        .ship_route(Route::ViewSessionRaw_template(), view_sessions_api::raw_session)
        // Feedback ingress
        .ship_route(Route::PostFeedback, feedback_api::post_feedback)
        .ship_route(Route::ViewGit, views_git::git_shell)
        .ship_route(Route::ViewGitSse, views_git::git_sse)
        .ship_route(Route::ViewPipelines, views_pipelines::pipelines_shell)
        .ship_route(Route::ViewPipelinesSse, views_pipelines::pipelines_sse)
        .ship_route(Route::ViewSettings, views_settings::settings_shell)
        .ship_route(Route::ViewSettingsSse, views_settings::settings_sse)
        .ship_route(Route::ViewAgents, views_agents::agents_shell)
        .ship_route(Route::ViewAgentDetail_template(), views_agents::agent_detail)
        .ship_route(Route::ViewAgentsCreate, views_agents::agents_create)
        .ship_route(Route::ViewSkills, views_skills::skills_shell)
        .ship_route(Route::ViewSkillsRail, views_skills::skills_rail)
        .ship_route(Route::ViewSkillDetail_template(), views_skills::skill_detail)
        .ship_route(Route::ViewChat, views_chat::chat_shell)
        .ship_route(Route::ViewChatSse, views_chat::chat_sse)
        .ship_route(Route::ViewChatSend, views_chat::chat_send)
        .ship_route(Route::ViewEvents, views_events::events_shell)
        .ship_route(Route::ViewEventsSse, views_events::events_sse)
        .ship_route(Route::ViewSpecs, specs_view::specs_shell)
        .ship_route(Route::ViewSpecsSse, specs_view::specs_view_sse)
        // Specs REST
        .ship_route(Route::ListSpecs, specs_api::list_specs)
        .ship_route(Route::CreateSpec, specs_api::create_spec)
        .ship_route(Route::GetSpec_template(), specs_api::get_spec)
        .ship_route(Route::UpdateSpec_template(), specs_api::update_spec)
        .ship_route(Route::DeleteSpec_template(), specs_api::delete_spec)
        .ship_route(Route::LaunchSpec_template(), specs_api::launch_spec)
        // Events / projections
        .ship_route(Route::EventStream, runtime_api::event_stream)
        .ship_route(Route::RebuildProjections, runtime_api::rebuild_projections)
        .ship_route(Route::WorkspacePty_template(), pty_handler::workspace_pty)
        // Settings
        .ship_route(Route::ListSettings, settings_api::list_settings)
        .ship_route(Route::UpdateSetting, settings_api::update_setting)
        .ship_route(Route::GetSetting_template(), settings_api::get_setting)
        .ship_route(Route::ListProviders, settings_api::list_providers)
        // Supervisor
        .ship_route(Route::StartWorkspace_template(), supervisor::start_workspace)
        // Project
        .ship_route(Route::ListSessionFiles_template(), project_api::list_session_files)
        .ship_route(Route::WorkspaceGitStatus_template(), project_api::git_status)
        .ship_route(Route::WorkspaceGitDiff_template(), project_api::git_diff)
        .ship_route(Route::WorkspaceGitLog_template(), project_api::git_log)
        .ship_route(Route::ListWorkspaceAgents_template(), project_api::list_agents)
        .ship_route(Route::WriteWorkspaceAgent_template(), project_api::write_agent)
        .ship_route(Route::ListWorkspaceSkills_template(), project_api::list_skills)
        .ship_route(Route::GetSkillVars_template(), project_api::get_skill_vars)
        .ship_route(Route::SetSkillVar_template(), project_api::set_skill_var)
        .ship_route(Route::DeleteWorkspace_template(), project_api::delete_workspace)
        .ship_route(Route::ActivateWorkspace_template(), project_api::activate_workspace)
        .ship_route(Route::ListWorkspacePipelines_template(), project_api::list_pipelines)
        .ship_route(Route::WritePipeline_template(), project_api::write_pipeline)
        .ship_route(Route::DeletePipeline_template(), project_api::delete_pipeline)
        .ship_route(Route::ProjectGitStatus, project_api::project_git_status)
        .ship_route(Route::ProjectGitDiff, project_api::project_git_diff)
        .ship_route(Route::ProjectGitLog, project_api::project_git_log)
        .ship_route(Route::EmitEvent, project_api::emit_event)
        .ship_route(Route::ArtifactsStatus_template(), project_api::artifacts_status)
        .ship_route(Route::PromoteArtifact_template(), project_api::promote_artifact)
        // Artifacts — upload route needs a relaxed `DefaultBodyLimit` so
        // Multipart can read the full 50 MB upload ceiling; other extractors
        // on this router continue to use the axum default.
        .merge(
            Router::<rest_api::ApiState>::new()
                .ship_route(Route::ArtifactsUpload, artifacts_api::upload_artifact)
                .ship_route(Route::ArtifactsList, artifacts_api::list_artifacts)
                .ship_route(Route::ArtifactDownload_template(), artifacts_api::download_artifact)
                .ship_route(Route::ArtifactDelete_template(), artifacts_api::delete_artifact)
                .layer(axum::extract::DefaultBodyLimit::max(
                    artifacts_api::MAX_UPLOAD_BYTES,
                )),
        )
        // Collections
        .ship_route(Route::ListCollections, collection_api::list_collections)
        .ship_route(Route::GetCollectionSchema_template(), collection_api::get_schema)
        .ship_route(Route::CreateCollectionItem_template(), collection_api::create_item)
        .ship_route(Route::GetCollectionItem_template(), collection_api::get_item)
        .ship_route(Route::UpdateCollectionItem_template(), collection_api::update_item)
        .ship_route(Route::DeleteCollectionItem_template(), collection_api::delete_item)
        .ship_route(Route::QueryCollection_template(), collection_api::query_items)
        // Glob-param file handlers (not in enum — glob syntax is project_api
        // internal; keep flat).
        .route(
            "/api/workspaces/{id}/session-files/{*path}",
            axum::routing::get(project_api::read_session_file)
                .put(project_api::write_session_file)
                .delete(project_api::delete_session_file),
        )
        .route(
            "/api/workspaces/{id}/skills/{skillId}/files/{*path}",
            axum::routing::put(project_api::write_skill_file)
                .delete(project_api::delete_skill_file),
        )
        .with_state(st)
}

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
    Json(serde_json::json!({
        "healthy": all_healthy,
        "pid": std::process::id(),
        "dev_mode": pairing::is_dev_mode(),
        "tasks": tasks,
    }))
}
