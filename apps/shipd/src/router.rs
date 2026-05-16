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
    NetworkServer, auth, project_api, services, ship_runtime,
    sessions, views,
};
use crate::routes::{
    actor_api, actors_api, agents_api, annotations_api, artifacts_api, collection_api,
    commands_api, create_api, feedback_api, git_api, hooks_api, human_webhook, logs_api,
    mcp_api, openapi, permissions_api, pipelines_api, projects_api, rest_api, rules_api,
    runtime_api, sdk_manifest, search_api, settings_api, skills_api, specs_api,
    themes_api, view_edit_mode_api, view_edit_tool_api, view_errors_api, view_sessions_api,
    webhook_ext,
};
use crate::auth::{attest, pairing, session};
use crate::transport::{cors, pty as pty_handler};
use crate::routes::mcp_registry as mcp_registry;
use axum::routing;

/// Build the full Axum router with all route groups, auth, and CORS.
pub fn build_router(
    rt: Arc<ship_runtime::ShipRuntime>,
    token: Arc<String>,
    session_registry: auth::SessionRegistry,
    pairing_registry: pairing::SharedPairingRegistry,
    attestation_verifier: attest::AttestationVerifier,
    mcp_ct: CancellationToken,
    daemon_mode: ship_sui::DaemonMode,
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
        // Drift-detection endpoint — returns daemon + compiler build
        // identity so the shell can compare its loaded `_compilerVersion`
        // marker against what the daemon was built with. Unauthenticated
        // for the same reason `/api/health` is: shell needs it before
        // any session exists. Restored alongside the `/_sui/*` wiring the
        // phase2-substrate-split merge dropped (originally from 0f679ca0).
        .route("/api/_meta", axum::routing::get({
            let bi = build_info::build_info!();
            let dbi = ship_sui::DaemonBuildInfo {
                cargo: env!("CARGO_PKG_VERSION"),
                git: bi.git_sha,
                dirty: bi.git_dirty,
                built_at: bi.timestamp,
            };
            move || ship_sui::serve_meta(Some(axum::Extension(dbi)))
        }))
        // Shared static assets — no user-specific data, safe to serve unauth.
        .ship_route(Route::AssetViewTokensCss, views::serve_view_tokens)
        .ship_route(Route::AssetSuiElementsManifest, views::serve_sui_elements_manifest)
        .ship_route(Route::AssetThemeCss, views::serve_theme_css)
        .ship_route(Route::ListThemes, views::serve_themes_list)
        // SUI framework hosting — runtime/dom/sdk/components bundles +
        // per-view template/payload/css + data/mutation/patch endpoints.
        // These sit under `/_sui/*` (plus `/view-host` and `/sui/{name}`);
        // the handlers live in `ship_sui` (asset serving) and
        // `apps/shipd/src/views/sui_*.rs` (project-scoped, state-aware).
        // State plumbs through `with_state(rt.clone())` at the bottom of
        // this builder. HMR routes stay dev-only — see further below.
        //
        // Restored after the phase2-substrate-split merge dropped the
        // wiring originally added in 30313c8f; without it every `/_sui/*`
        // request 404s and shell2 cannot boot.
        .ship_route(
            Route::AssetSuiCustomElementsManifest,
            views::serve_sui_elements_manifest,
        )
        .ship_route(Route::SuiRuntimeJs, ship_sui::serve_sui_runtime)
        .ship_route(Route::SuiDomJs, ship_sui::serve_sui_dom)
        .ship_route(Route::SuiSdkClientJs, ship_sui::serve_sui_sdk_client)
        .ship_route(Route::SuiComponentsJs, ship_sui::serve_sui_components)
        .ship_route(Route::SuiLucideIconsJson, ship_sui::serve_sui_lucide_icons)
        .ship_route(Route::SuiComponentBundle_template(), ship_sui::serve_sui_island)
        .ship_route(
            Route::SuiViewTemplate_template(),
            ship_sui::serve_sui_view_template,
        )
        .ship_route(
            Route::SuiViewSetup_template(),
            ship_sui::serve_sui_view_setup,
        )
        .ship_route(
            Route::SuiViewPayload_template(),
            ship_sui::serve_view_payload,
        )
        .ship_route(
            Route::SuiViewCss_template(),
            ship_sui::serve_sui_view_css,
        )
        .ship_route(Route::SuiViewSse_template(), views::sui_sse::serve_sui_sse)
        .ship_route(Route::SuiViewHost, ship_sui::serve_view_host)
        .ship_route(
            Route::SuiViewHostForName_template(),
            ship_sui::serve_view_host,
        )
        .ship_route(Route::SuiData, views::sui_data::serve_data)
        .ship_route(
            Route::SuiMutation_template(),
            views::sui_mutation::serve_sui_mutation,
        )
        // NOTE: `SuiPatchElements` is intentionally NOT registered here —
        // its handler requires `Scope::Full` and extracts
        // `Extension<AuthContext>`, so it must sit behind the auth
        // middleware. A public registration left that extension
        // unfulfilled and every POST 500'd. It is registered in
        // `build_ship_routes` (the authenticated group) instead.
        .ship_route(Route::SuiSource_template(), views::sui_source::serve_sui_source)
        // HMR routes (SuiHmr / SuiHmrClientJs / SuiDevHmr) are NOT wired
        // here — they stay development-only and are merged into `root`
        // inside the `daemon_mode == Development` block further below.
        // Glob-param SUI fallbacks — the Route enum doesn't model `{*sub_path}`,
        // so these stay as direct `.route()` calls to preserve the donor
        // surface (view-host + payload sub-path + studio /sui/{name}/{*sub_path}).
        .route(
            "/_sui/views/{name}/{*sub_path}",
            axum::routing::get(ship_sui::serve_view_payload_subpath),
        )
        .route(
            "/view-host/{*rest}",
            axum::routing::get(ship_sui::serve_view_host),
        )
        .route(
            "/sui/{name}/{*sub_path}",
            axum::routing::get(ship_sui::serve_view_host),
        )
        // First-run flow — both routes are unauthenticated so a fresh
        // install can reach the setup wizard before any browser is paired.
        .ship_route(Route::Index, views::setup::index_handler)
        .ship_route(Route::Setup, views::setup::setup_handler)
        // SUI state-using handlers (sui_sse, sui_data, sui_mutation,
        // sui_agent_push) extract `State<ApiState>`. Bind the state here
        // so the resulting `Router<()>` merges cleanly with the root
        // `Router::new()` at the bottom of this builder.
        .with_state(rt.clone());

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
                auth::list_auth_sessions(list_sessions_registry.clone(), auth_ext)
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
        .nest("/api/v1", hook_routes)
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
    // HMR routes are development-only; not registered in production mode.
    if daemon_mode == ship_sui::DaemonMode::Development {
        let hmr_routes: Router<()> = Router::new()
            .ship_route(Route::SuiHmr, views::sui_hmr::serve_hmr)
            .ship_route(Route::SuiHmrClientJs, views::sui_hmr::serve_hmr_client_js)
            .ship_route(Route::SuiDevHmr, views::sui_hmr::serve_bundle_hmr);
        root = root.merge(hmr_routes);
    }
    root
        // Daemon-mode Extension — Sui handlers read it via
        // `Option<Extension<DaemonMode>>` without router-state plumbing.
        .layer(axum::Extension(daemon_mode))
        // X-Shipd-Build header on every response: build SHA + dirty flag.
        // CLI/shell/browser read this to detect daemon-vs-client skew.
        .layer(axum::middleware::from_fn(build_header_middleware))
        .layer(axum::middleware::from_fn(cors::cors_middleware))
        // Outer request body cap. 50 MB matches the files-API upload limit.
        // JSON / Form extractors still enforce axum's `DefaultBodyLimit`
        // (2 MB) on their own — this layer only sets the outer ceiling.
        .layer(tower_http::limit::RequestBodyLimitLayer::new(
            crate::routes::artifacts_api::MAX_UPLOAD_BYTES,
        ))
        // Catch handler panics → 500 instead of connection abort.
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(handle_panic))
}

/// Stamp `X-Shipd-Build` on every response so clients can detect daemon
/// version skew. On the way in, also log a one-time WARN per distinct
/// `X-Shipd-Client-Build` value when the client's SHA diverges from
/// ours — daemon-side proof that a stale CLI is talking to us.
async fn build_header_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let bi = build_info::build_info!();

    // Inbound: warn once per distinct client SHA on skew.
    if let Some(client_value) = req
        .headers()
        .get("x-shipd-client-build")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
    {
        let server_short = bi.short();
        let client_sha = client_value.split('+').next().unwrap_or("").to_string();
        if client_value != server_short && client_sha != bi.git_sha {
            static WARNED: std::sync::OnceLock<
                std::sync::Mutex<std::collections::HashSet<String>>,
            > = std::sync::OnceLock::new();
            let warned = WARNED.get_or_init(|| std::sync::Mutex::new(Default::default()));
            let mut set = warned.lock().unwrap();
            if set.insert(client_value.clone()) {
                tracing::warn!(
                    client = %client_value,
                    server = %server_short,
                    "client build differs from daemon — restart the client to avoid API skew"
                );
            }
        }
    }

    let mut response = next.run(req).await;
    if let Ok(value) = axum::http::HeaderValue::from_str(&bi.short()) {
        response.headers_mut().insert("X-Shipd-Build", value);
    }
    response
}

/// All enumerated routes, registered via `ShipRouterExt::ship_route` against
/// the canonical `Route` enum. Paths come from `routes.rs` — this function
/// only binds handlers.
///
/// Two preserved non-enum `.route()` paths:
/// - `/api/v1/projects/{id}/session-files/{*path}` — glob param, specific to
///   project_api's recursive file handler. Kept alongside the enum variant
///   that exposes the non-glob listing endpoint.
/// - `/api/v1/skills/{id}/files/{*path}` — same reason (SkillService sub-file route).
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
        .ship_route(Route::ListMesh, actors_api::list_mesh_handler)
        .ship_route(Route::GetActor_template(), actors_api::get_actor_handler)
        .ship_route(Route::ActorEvents_template(), actors_api::list_actor_events_handler)
        .ship_route(Route::ActorInput_template(), actors_api::actor_input_handler)
        .ship_route(Route::MeshSnapshot, actors_api::mesh_snapshot_handler)
        .ship_route(Route::MeshEventsStream, actors_api::mesh_events_handler)
        .ship_route(Route::KillTree_template(), actors_api::kill_tree_handler)
        // Capabilities (no state required; ok against a stateful router)
        .ship_route(Route::Capabilities, rest_api::capabilities)
        // Runtime
        .ship_route(Route::ListSessions, runtime_api::list_workspace_sessions)
        .ship_route(Route::ListAgents, runtime_api::list_agents)
        .ship_route(Route::StopAgent_template(), runtime_api::stop_agent)
        .ship_route(Route::SteerAgent_template(), runtime_api::steer_agent)
        .ship_route(Route::ListJobs, runtime_api::list_jobs)
        .ship_route(Route::DispatchPipeline, runtime_api::dispatch_pipeline)
        .ship_route(Route::GetJob_template(), runtime_api::get_job_detail)
        .ship_route(Route::EndJob, runtime_api::end_job)
        .ship_route(Route::CancelJob_template(), runtime_api::cancel_job)
        .ship_route(Route::JobStatusUpdate_template(), project_api::update_job_status)
        .ship_route(Route::JobStepDone_template(), project_api::job_step_done)
        .ship_route(Route::StreamJobLog_template(), logs_api::stream_job_log)
        .ship_route(Route::ListProcs, runtime_api::list_procs)
        .ship_route(Route::PollEvents, runtime_api::poll_events)
        .ship_route(Route::DaemonStatus, runtime_api::daemon_status)
        // Views
        .ship_route(Route::ListViews, views::list_views)
        .ship_route(Route::ShipSchema, services::schema::serve_schema)
        .ship_route(Route::ServeViewSource_template(), views::source_api::serve_view_source)
        .ship_route(Route::CreateSurface_template(), create_api::create_surface)
        .ship_route(Route::WriteViewTemplate_template(), views::write_api::write_view_template)
        .ship_route(Route::WriteViewManifestApi_template(), views::write_api::write_view_manifest)
        .ship_route(Route::ForkView_template(), views::write_api::fork_view)
        .ship_route(Route::ThemeContrast, themes_api::theme_contrast)
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
        .ship_route(Route::InstalledProviders, sessions::api::installed_providers)
        .ship_route(Route::ListProjects, projects_api::list_projects)
        .ship_route(Route::AddProject, projects_api::add_project)
        .ship_route(Route::SwitchProject, projects_api::switch_project)
        .ship_route(Route::ActivateProject_template(), projects_api::activate_project)
        .ship_route(Route::GetActiveProject, projects_api::get_active_project)
        .ship_route(Route::ListSessionsAcrossProjects, runtime_api::list_sessions_across_projects)
        .ship_route(Route::ServeViewManifest_template(), views::serve_view_manifest)
        .ship_route(Route::ServeViewBundle_template(), views::serve_view_bundle)
        // `POST /_sui/views/{name}/patch-elements` — agent-push DOM patches.
        // Authenticated (requires `Scope::Full`); the rest of the `/_sui/*`
        // surface is public and lives in `public_routes`. The handler takes
        // no router state, so it composes cleanly into this stateful group.
        .ship_route(
            Route::SuiPatchElements_template(),
            views::sui_agent_push::serve_patch_elements,
        )
        // Agent-owned view sessions
        .ship_route(Route::OpenViewSession, view_sessions_api::open_session)
        .ship_route(Route::PatchViewSession_template(), view_sessions_api::patch_session)
        .ship_route(Route::AppendViewSession_template(), view_sessions_api::append_session)
        .ship_route(Route::SignalViewSession_template(), view_sessions_api::signal_session)
        .ship_route(Route::CloseViewSession_template(), view_sessions_api::close_session)
        .ship_route(Route::SubscribeViewSession_template(), view_sessions_api::subscribe_session)
        // Feedback ingress
        .ship_route(Route::PostFeedback, feedback_api::post_feedback)
        .ship_route(Route::PostAnnotation, annotations_api::post_annotation)
        // View authoring — chrome edit-mode handshake + agent tool dispatch.
        .ship_route(Route::EnterViewEditMode_template(), view_edit_mode_api::enter_edit_mode)
        .ship_route(Route::ExitViewEditMode_template(), view_edit_mode_api::exit_edit_mode)
        .ship_route(
            Route::DispatchViewEditTool_template(),
            view_edit_tool_api::dispatch_tool,
        )
        // AgentService — generic CRUD + lifecycle. First-party and
        // third-party views use this surface equally.
        .ship_route(Route::AgentsList, agents_api::list)
        .ship_route(Route::AgentsGet_template(), agents_api::get)
        .ship_route(Route::AgentsPut_template(), agents_api::put)
        .ship_route(Route::AgentsPatch_template(), agents_api::patch)
        .ship_route(Route::AgentsDelete_template(), agents_api::delete)
        .ship_route(Route::AgentsCreate, agents_api::create)
        .ship_route(Route::AgentsRename_template(), agents_api::rename)
        .ship_route(Route::AgentsDuplicate_template(), agents_api::duplicate)
        .ship_route(Route::AgentsMove_template(), agents_api::move_scope)
        // PipelineService — CRUD + run ops. Dispatch / cancel / log-stream
        // wrap the existing kernel-backed handlers so the write paths stay
        // single-sourced.
        .ship_route(Route::PipelinesList, pipelines_api::list)
        .ship_route(Route::PipelinesGet_template(), pipelines_api::get)
        .ship_route(Route::PipelinesPut_template(), pipelines_api::put)
        .ship_route(Route::PipelinesPatch_template(), pipelines_api::patch)
        .ship_route(Route::PipelinesDelete_template(), pipelines_api::delete)
        .ship_route(Route::PipelinesDispatch_template(), pipelines_api::dispatch)
        .ship_route(Route::PipelineRunCancel_template(), pipelines_api::cancel_run)
        .ship_route(Route::PipelineRunsList, pipelines_api::list_runs)
        .ship_route(Route::PipelineRunGet_template(), pipelines_api::get_run)
        .ship_route(Route::PipelineRunLogStream_template(), logs_api::stream_job_log)
        // RuleService — markdown rules.
        .ship_route(Route::RulesList, rules_api::list)
        .ship_route(Route::RulesGet_template(), rules_api::get)
        .ship_route(Route::RulesPut_template(), rules_api::put)
        .ship_route(Route::RulesDelete_template(), rules_api::delete)
        .ship_route(Route::RulesCreate, rules_api::create)
        // McpService — entries in mcp.jsonc as a collection surface.
        .ship_route(Route::McpList, mcp_api::list)
        .ship_route(Route::McpGet_template(), mcp_api::get)
        .ship_route(Route::McpPut_template(), mcp_api::put)
        .ship_route(Route::McpPatch_template(), mcp_api::patch)
        .ship_route(Route::McpDelete_template(), mcp_api::delete)
        .ship_route(Route::McpCreate, mcp_api::create)
        // PermissionService — preset entries + composed tool-patterns.
        .ship_route(Route::PermissionsList, permissions_api::list)
        .ship_route(Route::PermissionsToolPatterns, permissions_api::tool_patterns)
        .ship_route(Route::PermissionsGet_template(), permissions_api::get)
        .ship_route(Route::PermissionsPut_template(), permissions_api::put)
        .ship_route(Route::PermissionsPatch_template(), permissions_api::patch)
        .ship_route(Route::PermissionsDelete_template(), permissions_api::delete)
        .ship_route(Route::PermissionsCreate, permissions_api::create)
        // SkillService — service-shaped CRUD + file tree + vars. Replaces
        // the project-scoped skill endpoints that used to hang off
        // /api/v1/projects/{id}/skills/*.
        .ship_route(Route::SkillsList, skills_api::list)
        .ship_route(Route::SkillsGet_template(), skills_api::get)
        .ship_route(Route::SkillsCreate, skills_api::create)
        .ship_route(Route::SkillsDelete_template(), skills_api::delete)
        .ship_route(Route::SkillsPutFrontmatter_template(), skills_api::put_frontmatter)
        .ship_route(Route::SkillsPutBody_template(), skills_api::put_body)
        .ship_route(Route::SkillsRename_template(), skills_api::rename)
        .ship_route(Route::SkillsDuplicate_template(), skills_api::duplicate)
        .ship_route(Route::SkillsMove_template(), skills_api::move_scope)
        .ship_route(Route::SkillsGetVars_template(), skills_api::get_vars)
        .ship_route(Route::SkillsPutVar_template(), skills_api::put_var)
        .ship_route(Route::SkillsDeleteVar_template(), skills_api::delete_var)
        .ship_route(Route::SkillsInvoke_template(), skills_api::invoke)
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
        .ship_route(Route::ProjectPty_template(), pty_handler::workspace_pty)
        // Settings
        .ship_route(Route::ListSettings, settings_api::list_settings)
        .ship_route(Route::UpdateSetting, settings_api::update_setting)
        .ship_route(Route::GetSetting_template(), settings_api::get_setting)
        .ship_route(Route::ListProviders, settings_api::list_providers)
        // Project
        .ship_route(Route::ListSessionFiles_template(), project_api::list_session_files)
        .ship_route(Route::ListProjectAgents_template(), project_api::list_agents)
        // GitService — canonical `/api/v1/git/*` surface. Consolidates the
        // old workspace- and project-scoped git endpoints into one.
        .ship_route(Route::GitStatus, git_api::status_handler)
        .ship_route(Route::GitDiff, git_api::diff_handler)
        .ship_route(Route::GitLog, git_api::log_handler)
        .ship_route(Route::GitBranches, git_api::branches_handler)
        .ship_route(Route::GitStage, git_api::stage_handler)
        .ship_route(Route::GitUnstage, git_api::unstage_handler)
        .ship_route(Route::GitCommit, git_api::commit_handler)
        .ship_route(Route::GitPush, git_api::push_handler)
        .ship_route(Route::GitFetch, git_api::fetch_handler)
        .ship_route(Route::GitPull, git_api::pull_handler)
        .ship_route(Route::GitCheckout, git_api::checkout_handler)
        .ship_route(Route::GitSse, git_api::sse_handler)
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
            "/api/v1/artifacts/{id}",
            routing::head(artifacts_api::head_artifact),
        )
        .ship_route(Route::ArtifactMetadata_template(), artifacts_api::artifact_metadata)
        .ship_route(Route::ArtifactPatch_template(), artifacts_api::patch_artifact)
        .ship_route(Route::ArtifactDelete_template(), artifacts_api::delete_artifact)
        .ship_route(Route::ArtifactPromote_template(), artifacts_api::promote_artifact)
        // Collections
        // Collections — GET + POST on /api/v1/collections merged on one route.
        .route(
            "/api/v1/collections",
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
        // Command bar registry
        .ship_route(Route::ListCommands, commands_api::list_commands)
        // Unified search — cmdk find mode consumer.
        .ship_route(Route::SearchAll, search_api::search)
        // SDK manifest (read-side)
        .ship_route(Route::ViewMeta_template(), sdk_manifest::get_view_meta)
        .ship_route(Route::SdkRoot, sdk_manifest::get_sdk_root)
        .ship_route(Route::SdkQuery, sdk_manifest::query_sdk)
        // Client-side view error capture — GET/POST/DELETE on one path,
        // merged into a single MethodRouter (axum panics on duplicate paths
        // across `.route()` calls). Canonical path comes from the Route enum
        // so URL builders (TS, Jinja) stay consistent.
        .route(
            "/api/v1/view-errors",
            axum::routing::post(view_errors_api::post_error)
                .get(view_errors_api::list_errors)
                .delete(view_errors_api::clear_errors),
        )
        // Glob-param file handlers (not in enum — glob syntax is project_api
        // internal; keep flat).
        .route(
            "/api/v1/projects/{id}/session-files/{*path}",
            axum::routing::get(project_api::read_session_file)
                .put(project_api::write_session_file)
                .delete(project_api::delete_session_file),
        )
        // SkillService files subresource — glob param keeps this flat (the
        // codegen Route enum doesn't model `{*path}`).
        .route(
            "/api/v1/skills/{id}/files/{*path}",
            axum::routing::put(skills_api::put_file)
                .delete(skills_api::delete_file),
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
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
    ),
    security(),
    tag = "health"
)]
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
