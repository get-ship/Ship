//! Axum router assembly: public + authenticated route groups, enumerated ship routes,
//! panic handler, and health check.

use axum::{
    Json, Router,
    body::Body,
    response::Response,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::{
    auth, project_api, services, ship_runtime,
    sessions, views,
};
use crate::routes::{
    account_api, actor_api, actors_api, annotations_api,
    artifacts_api, commands_api, create_api, feedback_api, git_api,
    hooks_api, human_webhook, logs_api,
    registry_api, rest_api, runtime_api,
    sandbox_api, sdk_manifest, search_api, settings_api, themes_api,
    view_edit_mode_api, view_edit_tool_api, view_errors_api, view_sessions_api,
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
    _mcp_ct: CancellationToken,
    daemon_mode: deck::DaemonMode,
) -> Router {
    use crate::routes::{Route, ShipRouterExt};
    // Internal MCP server retired (#75): agents use the ConnectRPC op surface
    // via `ship ops` + capability-skills; external MCP clients use the
    // standalone `apps/mcp` adapter. The registry browser (`mcp_registry`) and
    // the `mcp.*` catalog reads stay.

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
        .route("/hooks/permission-denied", axum::routing::post(hooks_api::hook_permission_denied))
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
            axum::routing::post(move |
                peer: axum::extract::ConnectInfo<std::net::SocketAddr>,
                body: Option<axum::Json<auth::PairInitRequest>>,
            | {
                auth::pair_init(pair_init_reg.clone(), pair_init_sessions.clone(), Some(peer), body)
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
            let dbi = deck::DaemonBuildInfo {
                cargo: env!("CARGO_PKG_VERSION"),
                git: bi.git_sha,
                dirty: bi.git_dirty,
                built_at: bi.timestamp,
            };
            move || deck::serve_meta(Some(axum::Extension(dbi)))
        }))
        // Shared static assets — no user-specific data, safe to serve unauth.
        .ship_public_route(Route::AssetViewTokensCss, views::serve_view_tokens)
        .ship_public_route(Route::AssetThemeCss, views::serve_theme_css)
        .ship_public_route(Route::ListThemes, views::serve_themes_list)
        // SUI framework hosting — runtime/dom/sdk/components bundles +
        // per-view template/payload/css + data/mutation/patch endpoints.
        // These sit under `/_sui/*` (plus `/view-host` and `/sui/{name}`);
        // the handlers live in `deck` (asset serving) and
        // `apps/shipd/src/views/sui_*.rs` (project-scoped, state-aware).
        // State plumbs through `with_state(rt.clone())` at the bottom of
        // this builder. HMR routes stay dev-only — see further below.
        //
        // Restored after the phase2-substrate-split merge dropped the
        // wiring originally added in 30313c8f; without it every `/_sui/*`
        // request 404s and shell cannot boot.
        // Workspace catalog endpoints — replace the retired
        // `/api/assets/sui-custom-elements.json` umbrella. Two
        // projections (workspace = every component, published = only
        // non-private packaged components) emitted by
        // `bun packages/deck/catalog/src/cli.ts` to
        // `.deck/catalogs/{workspace,published}.json`.
        .ship_public_route(Route::SuiCatalogWorkspace, deck::serve_catalog_workspace)
        .ship_public_route(Route::SuiCatalogPublished, deck::serve_catalog_published)
        // Generalized pack-bundle handler — serves
        // `/_sui/packs/<pack-id>/<bundle>` for any workspace package
        // with a `customElements` field in package.json. Index built
        // lazily by walking the root workspaces glob.
        .ship_public_route(Route::SuiPackAsset_template(), deck::serve_pack_asset)
        .ship_public_route(Route::SuiRuntimeJs, deck::serve_sui_runtime)
        .ship_public_route(Route::SuiDomJs, deck::serve_sui_dom)
        .ship_public_route(Route::SuiSdkClientJs, deck::serve_sui_sdk_client)
        .ship_public_route(Route::SuiComponentsJs, deck::serve_sui_components)
        .ship_public_route(Route::SuiLucideIconsJson, deck::serve_sui_lucide_icons)
        .ship_public_route(Route::SuiLiveReloadJs, deck::serve_sui_live_reload)
        .ship_public_route(Route::SuiComponentBundle_template(), deck::serve_sui_island)
        .ship_public_route(
            Route::SuiViewTemplate_template(),
            deck::serve_sui_view_template,
        )
        .ship_public_route(
            Route::SuiViewSetup_template(),
            deck::serve_sui_view_setup,
        )
        .ship_public_route(
            Route::SuiViewPayload_template(),
            deck::serve_view_payload,
        )
        .ship_public_route(
            Route::SuiViewCss_template(),
            deck::serve_sui_view_css,
        )
        .ship_public_route(Route::SuiViewSse_template(), views::sui_sse::serve_sui_sse)
        .ship_public_route(Route::SuiViewHost, deck::serve_view_host)
        .ship_public_route(
            Route::SuiViewHostForName_template(),
            deck::serve_view_host,
        )
        .ship_public_route(Route::SuiData, views::sui_data::serve_data)
        .ship_public_route(
            Route::SuiMutation_template(),
            views::sui_mutation::serve_sui_mutation,
        )
        // NOTE: `SuiPatchElements` is intentionally NOT registered here —
        // its handler requires `Scope::Full` and extracts
        // `Extension<AuthContext>`, so it must sit behind the auth
        // middleware. A public registration left that extension
        // unfulfilled and every POST 500'd. It is registered in
        // `build_ship_routes` (the authenticated group) instead.
        .ship_public_route(Route::SuiSource_template(), views::sui_source::serve_sui_source)
        // The HMR route (SuiDevHmr) is NOT wired here — it stays
        // development-only and is merged into `root` inside the
        // `daemon_mode == Development` block further below.
        // Glob-param SUI fallbacks — the Route enum doesn't model `{*sub_path}`,
        // so these stay as direct `.route()` calls to preserve the donor
        // surface (view-host + payload sub-path + studio /sui/{name}/{*sub_path}).
        .route(
            "/_sui/views/{name}/{*sub_path}",
            axum::routing::get(deck::serve_view_payload_subpath),
        )
        .route(
            "/view-host/{*rest}",
            axum::routing::get(deck::serve_view_host),
        )
        .route(
            "/sui/{name}/{*sub_path}",
            axum::routing::get(deck::serve_view_host),
        )
        // First-run flow — both routes are unauthenticated so a fresh
        // install can reach the setup wizard before any browser is paired.
        .ship_public_route(Route::Index, views::setup::index_handler)
        .ship_public_route(Route::Setup, views::setup::setup_handler)
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

    // Account service is owned by ShipRuntime; expose it as an Extension
    // so REST handlers (`account_api::*`) can extract it. The read-op
    // registry reaches the same Arc via `rt.account()`.
    let account_service = rt.account().clone();

    // MCP registry proxy is a public, read-only endpoint — match the
    // original Cloudflare Worker handler's "no auth required" contract
    // so the shell's agent builder can query it before pairing.
    let mcp_registry_routes = Router::new()
        .ship_public_route(Route::McpServers, mcp_registry::list_mcp_servers)
        .layer(axum::Extension(registry_state));

    let authenticated_routes = Router::new()
        .nest("/api", pair_confirm_routes)
        .ship_route(Route::IssueSession, session_issue_handler)
        .merge(api_ship_routes)
        // Hook ingress lives at flat `/hooks/*` (authed, but OFF `/api/v1`):
        // consumed by generated git-hook shell scripts that POST via curl and
        // can't speak Connect, so it's a permanent ingress holdout. Kept out of
        // `/api/v1` so that namespace can shrink to just the auth-bootstrap.
        .merge(hook_routes)
        .nest("/api", webhook_routes)
        // ConnectRPC op surface (ship.ops.v1) — additive, wrapped by
        // auth_middleware below so the gate sees AuthContext. See ops_rpc.
        .merge(crate::ops_rpc::router(rt.clone()))
        .layer(axum::Extension(feedback_guard))
        .layer(axum::Extension(account_service))
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
        .merge(authenticated_routes);
    // The HMR firehose is development-only; not registered in production.
    if daemon_mode == deck::DaemonMode::Development {
        let hmr_routes: Router<()> = Router::new()
            .ship_public_route(Route::SuiDevHmr, views::sui_hmr::serve_bundle_hmr);
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
        // Isolation reads are served by the proto IsolationService
        // (ship.ops.v1) — the one op-surface path. The former
        // /api/v1/isolation/* REST routes were removed to avoid a parallel
        // path; init's gated-op migration lands at the connect cutover.
        .ship_route(Route::SandboxList, sandbox_api::sandbox_list)
        .ship_route(Route::SandboxGet_template(), sandbox_api::sandbox_get)
        // Runtime
        .ship_route(Route::StopAgent_template(), runtime_api::stop_agent)
        .ship_route(Route::SteerAgent_template(), runtime_api::steer_agent)
        .ship_route(Route::ListJobs, runtime_api::list_jobs)
        .ship_route(Route::DispatchPipeline, runtime_api::dispatch_agent_session)
        .ship_route(Route::PipelinesRun, runtime_api::run_pipeline)
        .ship_route(Route::GetJob_template(), runtime_api::get_job_detail)
        .ship_route(Route::EndJob, runtime_api::end_job)
        .ship_route(Route::CancelJob_template(), runtime_api::cancel_job)
        .ship_route(Route::RunStatusUpdate_template(), project_api::update_job_status)
        .ship_route(Route::StepDoneSignal_template(), project_api::step_done)
        .ship_route(Route::StreamJobLog_template(), logs_api::stream_job_log)
        .ship_route(
            Route::AttachAgentSession_template(),
            crate::routes::agent_sessions_attach::attach_agent_session,
        )
        .ship_route(Route::DaemonStatus, runtime_api::daemon_status)
        // Views — list + source now served by DiscoveryService (Connect, #139).
        .ship_route(Route::ShipSchema, services::schema::serve_schema)
        .ship_route(Route::ListViewFiles_template(), views::files_api::list_view_files)
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
        .ship_route(Route::InstalledProviders, sessions::api::installed_harnesses)
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
        // Account — gateway-mediated; CallerKind::Cli only (MCP denied)
        .ship_route(Route::StartAccountLogin, account_api::start_login)
        .ship_route(Route::CompleteAccountLogin, account_api::complete_login)
        .ship_route(Route::GetAccountState, account_api::get_state)
        .ship_route(Route::LogoutAccount, account_api::logout)
        // View authoring — chrome edit-mode handshake + agent tool dispatch.
        .ship_route(Route::EnterViewEditMode_template(), view_edit_mode_api::enter_edit_mode)
        .ship_route(Route::ExitViewEditMode_template(), view_edit_mode_api::exit_edit_mode)
        .ship_route(
            Route::DispatchViewEditTool_template(),
            view_edit_tool_api::dispatch_tool,
        )
        // Registry — gateway proxy. The daemon attaches the user's API key
        // and forwards to the gateway. CLI never talks to the gateway.
        .ship_route(Route::Publish, registry_api::publish)
        // AgentService — generic CRUD + lifecycle. First-party and
        // third-party views use this surface equally.
        // PipelineService — CRUD + run ops. Dispatch / cancel / log-stream
        // wrap the existing kernel-backed handlers so the write paths stay
        // single-sourced.
        .ship_route(Route::PipelineRunLogStream_template(), logs_api::stream_job_log)
        // RuleService — markdown rules.
        // PermissionService — preset entries + composed tool-patterns.
        // SkillService — service-shaped CRUD + file tree + vars. Replaces
        // the project-scoped skill endpoints that used to hang off
        // /api/v1/projects/{id}/skills/*.
        .ship_route(Route::ChatSend, services::chat::send_message)
        // Specs REST
        // Events / projections
        .ship_route(Route::EventStream, runtime_api::event_stream)
        .ship_route(Route::RebuildProjections, runtime_api::rebuild_projections)
        .ship_route(Route::ProjectPty_template(), pty_handler::workspace_pty)
        // Settings
        .ship_route(Route::ListProviders, settings_api::list_harnesses)
        // Project
        .ship_route(Route::ListSessionFiles_template(), project_api::list_session_files)
        .ship_route(Route::ListProjectAgents_template(), project_api::list_agents)
        // GitService — canonical `/api/v1/git/*` surface. Consolidates the
        // old workspace- and project-scoped git endpoints into one.
        .ship_route(Route::GitStage, git_api::stage_handler)
        .ship_route(Route::GitUnstage, git_api::unstage_handler)
        .ship_route(Route::GitCommit, git_api::commit_handler)
        .ship_route(Route::GitPush, git_api::push_handler)
        .ship_route(Route::GitFetch, git_api::fetch_handler)
        .ship_route(Route::GitPull, git_api::pull_handler)
        .ship_route(Route::GitCheckout, git_api::checkout_handler)
        .ship_route(Route::GitBranchCreate, git_api::branch_create_handler)
        .ship_route(Route::GitBranchDelete, git_api::branch_delete_handler)
        .ship_route(Route::GitInit, git_api::init_handler)
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
        .ship_route(Route::ArtifactDownload_template(), artifacts_api::download_artifact)
        .route(
            "/api/v1/artifacts/{id}",
            routing::head(artifacts_api::head_artifact),
        )
        .ship_route(Route::ArtifactMetadata_template(), artifacts_api::artifact_metadata)
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
