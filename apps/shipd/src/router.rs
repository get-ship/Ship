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
    NetworkServer, actor_api, actors_api, annotations_api, artifacts_api, attest, auth, cors, feedback_api, hooks_api,
    human_webhook, lint_api, logs_api, mcp_registry, openapi, pairing, project_api, pty_handler,
    rest_api, runtime_api, sdk_manifest, services, session, settings_api, shell_assets,
    ship_runtime, specs_api, supervisor, view_errors_api, view_morph_target, view_sessions_api,
    views, collection_api, create_api, themes_api, sessions, projects_api, views_raw, webhook_ext,
};
use axum::routing;

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
        .route("/hooks/session-end", axum::routing::post(hooks_api::hook_session_end))
        .with_state(st.clone());
    // Enumerated routes — one `ApiState`-typed router, registered via
    // `ship_route(Route::X, handler)` against the canonical full paths.
    let api_ship_routes = build_ship_routes(st);

    // Public webhook route — unauthenticated; each trigger carries its own secret.
    let webhook_kernel = rt.kernel().clone();
    let webhook_source_route = if let Some(src) = rt.webhook_source() {
        Router::new()
            .route(
                "/api/hooks/webhook/{*path}",
                axum::routing::post(webhook_ext::handle_webhook),
            )
            .layer(axum::Extension(src))
            .layer(axum::Extension(webhook_kernel))
    } else {
        Router::new()
    };

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
        .ship_route(Route::AssetGitJs, views::serve_git)
        .ship_route(Route::AssetElementsManifest, views::serve_elements_manifest)
        .ship_route(Route::AssetElementsCatalog, views::serve_elements_catalog)
        .ship_route(Route::AssetJinjaMacrosJson, views::serve_jinja_macros_json)
        .ship_route(Route::AssetJinjaMacrosMd, views::serve_jinja_macros_md)
        .ship_route(Route::AssetThemeCss, views::serve_theme_css)
        .ship_route(Route::ListThemes, views::serve_themes_list)
        // First-run flow — both routes are unauthenticated so a fresh
        // install can reach the setup wizard before any browser is paired.
        .ship_route(Route::Index, views::setup::index_handler)
        .ship_route(Route::Setup, views::setup::setup_handler);

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
    let registry_state = mcp_registry::McpRegistryState::new();

    // MCP registry proxy is a public, read-only endpoint — match the
    // original Cloudflare Worker handler's "no auth required" contract
    // so the shell's agent builder can query it before pairing.
    let mcp_registry_routes = Router::new()
        .ship_route(Route::McpServers, mcp_registry::list_mcp_servers)
        .layer(axum::Extension(registry_state));

    let authenticated_routes = Router::new()
        .nest("/api", pair_confirm_routes)
        .ship_route(Route::IssueSession, session_issue_handler)
        .merge(api_ship_routes)
        .nest("/api", hook_routes)
        .nest("/api", webhook_routes)
        .nest_service("/mcp", service)
        .layer(axum::Extension(feedback_guard))
        // Dry-run view interceptor runs INSIDE auth (applied before the
        // auth layer here, which means outer->inner order wraps it with
        // auth) so requests must carry a valid bearer before short-circuit.
        .layer(axum::middleware::from_fn(move |req, next| {
            auth::auth_middleware(auth_token.clone(), auth_registry.clone(), req, next)
        }));

    let mut root = Router::new()
        .merge(public_routes)
        .merge(mcp_registry_routes)
        .merge(webhook_source_route)
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
/// - `/api/skills/{id}/files/{*path}` — same reason (SkillService sub-file route).
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
        .ship_route(Route::ListActors, actors_api::list_actors_handler)
        .ship_route(Route::GetActor_template(), actors_api::get_actor_handler)
        .ship_route(Route::ActorInput_template(), actors_api::actor_input_handler)
        // Capabilities (no state required; ok against a stateful router)
        .ship_route(Route::Capabilities, rest_api::capabilities)
        // Runtime
        .ship_route(Route::ListWorkspaces, runtime_api::list_workspaces)
        .ship_route(Route::ListSessions, runtime_api::list_sessions)
        .ship_route(Route::ListAgents, runtime_api::list_agents)
        .ship_route(Route::StopAgent_template(), runtime_api::stop_agent)
        .ship_route(Route::SteerAgent_template(), runtime_api::steer_agent)
        .ship_route(Route::ListJobs, runtime_api::list_jobs)
        .ship_route(Route::DispatchPipeline, runtime_api::dispatch_pipeline)
        .ship_route(Route::GetJob_template(), runtime_api::get_job_detail)
        .ship_route(Route::EndJob, runtime_api::end_job)
        .ship_route(Route::CancelJob_template(), runtime_api::cancel_job)
        .ship_route(Route::StreamJobLog_template(), logs_api::stream_job_log)
        .ship_route(Route::ListProcs, runtime_api::list_procs)
        .ship_route(Route::PollEvents, runtime_api::poll_events)
        .ship_route(Route::DaemonStatus, runtime_api::daemon_status)
        // Views
        .ship_route(Route::ListViews, views::list_views)
        .ship_route(Route::ShipSchema, services::schema::serve_schema)
        .ship_route(Route::ServeViewRouted_template(), views::routed_handler::serve_view_routed)
        .ship_route(Route::ServeViewSseRouted_template(), views::routed_sse::serve_view_sse_routed)
        .ship_route(Route::ServeViewSource_template(), views::source_api::serve_view_source)
        .ship_route(Route::CreateSurface_template(), create_api::create_surface)
        .ship_route(Route::WriteViewTemplate_template(), views::write_api::write_view_template)
        .ship_route(Route::WriteViewManifestApi_template(), views::write_api::write_view_manifest)
        .ship_route(Route::ForkView_template(), views::write_api::fork_view)
        .ship_route(Route::ThemeContrast, themes_api::theme_contrast)
        .ship_route(Route::SpawnPtySession, sessions::api::spawn_session)
        .ship_route(Route::ListPtySessions, sessions::api::list_sessions)
        .ship_route(Route::GetPtySession_template(), sessions::api::get_session)
        .ship_route(Route::StopPtySession_template(), sessions::api::stop_session)
        .ship_route(Route::PtySessionInput_template(), sessions::api::session_input)
        .ship_route(Route::PtySessionMessage_template(), sessions::api::session_message)
        .ship_route(Route::PtySessionReply_template(), sessions::api::session_reply)
        .ship_route(Route::StreamPtySession_template(), sessions::api::stream_session)
        .ship_route(Route::StreamPtySessionEvents_template(), sessions::api::stream_session_events)
        .ship_route(Route::InstalledProviders, sessions::api::installed_providers)
        .ship_route(Route::ListProjects, projects_api::list_projects)
        .ship_route(Route::AddProject, projects_api::add_project)
        .ship_route(Route::SwitchProject, projects_api::switch_project)
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
        .ship_route(Route::PostAnnotation, annotations_api::post_annotation)
        // AgentService — generic CRUD + lifecycle. First-party and
        // third-party views use this surface equally.
        .ship_route(Route::AgentsList, services::agents::list)
        .ship_route(Route::AgentsGet_template(), services::agents::get)
        .ship_route(Route::AgentsPut_template(), services::agents::put)
        .ship_route(Route::AgentsPatch_template(), services::agents::patch)
        .ship_route(Route::AgentsDelete_template(), services::agents::delete)
        .ship_route(Route::AgentsCreate, services::agents::create)
        .ship_route(Route::AgentsRename_template(), services::agents::rename)
        .ship_route(Route::AgentsDuplicate_template(), services::agents::duplicate)
        .ship_route(Route::AgentsMove_template(), services::agents::move_scope)
        // PipelineService — CRUD + run ops. Dispatch / cancel / log-stream
        // wrap the existing kernel-backed handlers so the write paths stay
        // single-sourced.
        .ship_route(Route::PipelinesList, services::pipelines::list)
        .ship_route(Route::PipelinesGet_template(), services::pipelines::get)
        .ship_route(Route::PipelinesPut_template(), services::pipelines::put)
        .ship_route(Route::PipelinesPatch_template(), services::pipelines::patch)
        .ship_route(Route::PipelinesDelete_template(), services::pipelines::delete)
        .ship_route(Route::PipelinesDispatch_template(), services::pipelines::dispatch)
        .ship_route(Route::PipelineRunCancel_template(), services::pipelines::cancel_run)
        .ship_route(Route::PipelineRunsList, services::pipelines::list_runs)
        .ship_route(Route::PipelineRunGet_template(), services::pipelines::get_run)
        .ship_route(Route::PipelineRunLogStream_template(), logs_api::stream_job_log)
        // RuleService — markdown rules.
        .ship_route(Route::RulesList, services::rules::list)
        .ship_route(Route::RulesGet_template(), services::rules::get)
        .ship_route(Route::RulesPut_template(), services::rules::put)
        .ship_route(Route::RulesDelete_template(), services::rules::delete)
        .ship_route(Route::RulesCreate, services::rules::create)
        // McpService — entries in mcp.jsonc as a collection surface.
        .ship_route(Route::McpList, services::mcp::list)
        .ship_route(Route::McpGet_template(), services::mcp::get)
        .ship_route(Route::McpPut_template(), services::mcp::put)
        .ship_route(Route::McpPatch_template(), services::mcp::patch)
        .ship_route(Route::McpDelete_template(), services::mcp::delete)
        .ship_route(Route::McpCreate, services::mcp::create)
        // PermissionService — preset entries + composed tool-patterns.
        .ship_route(Route::PermissionsList, services::permissions::list)
        .ship_route(Route::PermissionsToolPatterns, services::permissions::tool_patterns)
        .ship_route(Route::PermissionsGet_template(), services::permissions::get)
        .ship_route(Route::PermissionsPut_template(), services::permissions::put)
        .ship_route(Route::PermissionsPatch_template(), services::permissions::patch)
        .ship_route(Route::PermissionsDelete_template(), services::permissions::delete)
        .ship_route(Route::PermissionsCreate, services::permissions::create)
        // SkillService — service-shaped CRUD + file tree + vars. Replaces
        // the workspace-scoped skill endpoints that used to hang off
        // /api/workspaces/{id}/skills/*.
        .ship_route(Route::SkillsList, services::skills::list)
        .ship_route(Route::SkillsGet_template(), services::skills::get)
        .ship_route(Route::SkillsCreate, services::skills::create)
        .ship_route(Route::SkillsDelete_template(), services::skills::delete)
        .ship_route(Route::SkillsPutFrontmatter_template(), services::skills::put_frontmatter)
        .ship_route(Route::SkillsPutBody_template(), services::skills::put_body)
        .ship_route(Route::SkillsRename_template(), services::skills::rename)
        .ship_route(Route::SkillsDuplicate_template(), services::skills::duplicate)
        .ship_route(Route::SkillsMove_template(), services::skills::move_scope)
        .ship_route(Route::SkillsGetVars_template(), services::skills::get_vars)
        .ship_route(Route::SkillsPutVar_template(), services::skills::put_var)
        .ship_route(Route::SkillsDeleteVar_template(), services::skills::delete_var)
        .ship_route(Route::ChatSend, services::chat::send_message)
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
        .ship_route(Route::ListWorkspaceAgents_template(), project_api::list_agents)
        .ship_route(Route::WriteWorkspaceAgent_template(), project_api::write_agent)
        .ship_route(Route::DeleteWorkspace_template(), project_api::delete_workspace)
        .ship_route(Route::ActivateWorkspace_template(), project_api::activate_workspace)
        // GitService — canonical `/api/git/*` surface. Consolidates the
        // old workspace- and project-scoped git endpoints into one.
        .ship_route(Route::GitStatus, services::git::status_handler)
        .ship_route(Route::GitDiff, services::git::diff_handler)
        .ship_route(Route::GitLog, services::git::log_handler)
        .ship_route(Route::GitBranches, services::git::branches_handler)
        .ship_route(Route::GitStage, services::git::stage_handler)
        .ship_route(Route::GitUnstage, services::git::unstage_handler)
        .ship_route(Route::GitCommit, services::git::commit_handler)
        .ship_route(Route::GitPush, services::git::push_handler)
        .ship_route(Route::GitFetch, services::git::fetch_handler)
        .ship_route(Route::GitPull, services::git::pull_handler)
        .ship_route(Route::GitCheckout, services::git::checkout_handler)
        .ship_route(Route::GitSse, services::git::sse_handler)
        // commit-draft — GET/PUT/DELETE merged on one path (canonical path
        // matches `Route::GitCommitDraft{Get,Put,Delete}` in the enum).
        .route(
            "/api/git/commit-draft",
            axum::routing::get(services::git::draft_get_handler)
                .put(services::git::draft_put_handler)
                .delete(services::git::draft_delete_handler),
        )
        .ship_route(Route::EmitEvent, project_api::emit_event)
        // Artifacts platform (three-scope)
        .merge(
            Router::<rest_api::ApiState>::new()
                .ship_route(Route::ArtifactsUpload, artifacts_api::upload_artifact)
                .layer(axum::extract::DefaultBodyLimit::max(
                    artifacts_api::MAX_UPLOAD_BYTES,
                )),
        )
        .ship_route(Route::ArtifactsList, artifacts_api::list_artifacts)
        .ship_route(Route::ArtifactDownload_template(), artifacts_api::download_artifact)
        .route(
            "/api/artifacts/{id}",
            routing::head(artifacts_api::head_artifact),
        )
        .ship_route(Route::ArtifactMetadata_template(), artifacts_api::artifact_metadata)
        .ship_route(Route::ArtifactPatch_template(), artifacts_api::patch_artifact)
        .ship_route(Route::ArtifactDelete_template(), artifacts_api::delete_artifact)
        .ship_route(Route::ArtifactPromote_template(), artifacts_api::promote_artifact)
        // Collections
        // Collections — GET + POST on /api/collections merged on one route.
        .route(
            "/api/collections",
            axum::routing::get(collection_api::list_collections)
                .post(collection_api::create_collection),
        )
        .ship_route(Route::GetCollectionSchema_template(), collection_api::get_schema)
        .ship_route(Route::UpdateCollectionSchema_template(), collection_api::update_schema)
        .ship_route(Route::CreateCollectionItem_template(), collection_api::create_item)
        .ship_route(Route::GetCollectionItem_template(), collection_api::get_item)
        .ship_route(Route::UpdateCollectionItem_template(), collection_api::update_item)
        .ship_route(Route::DeleteCollectionItem_template(), collection_api::delete_item)
        .ship_route(Route::QueryCollection_template(), collection_api::query_items)
        // Lint
        .ship_route(Route::LintView, lint_api::lint_view)
        // SDK manifest (read-side)
        .ship_route(Route::ViewMeta_template(), sdk_manifest::get_view_meta)
        .ship_route(Route::SdkRoot, sdk_manifest::get_sdk_root)
        .ship_route(Route::SdkQuery, sdk_manifest::query_sdk)
        // Client-side view error capture — GET/POST/DELETE on one path,
        // merged into a single MethodRouter (axum panics on duplicate paths
        // across `.route()` calls). Canonical path comes from the Route enum
        // so URL builders (TS, Jinja) stay consistent.
        .route(
            "/api/view-errors",
            axum::routing::post(view_errors_api::post_error)
                .get(view_errors_api::list_errors)
                .delete(view_errors_api::clear_errors),
        )
        // Glob-param file handlers (not in enum — glob syntax is project_api
        // internal; keep flat).
        .route(
            "/api/workspaces/{id}/session-files/{*path}",
            axum::routing::get(project_api::read_session_file)
                .put(project_api::write_session_file)
                .delete(project_api::delete_session_file),
        )
        // SkillService files subresource — glob param keeps this flat (the
        // codegen Route enum doesn't model `{*path}`).
        .route(
            "/api/skills/{id}/files/{*path}",
            axum::routing::put(services::skills::put_file)
                .delete(services::skills::delete_file),
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
#[utoipa::path(
    operation_id = "healthCheck",
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Daemon health snapshot with supervised task status"),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "health"
)]
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
