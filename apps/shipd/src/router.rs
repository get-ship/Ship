//! Axum router assembly: public + authenticated route groups, enumerated ship routes,
//! panic handler, and health check.

use axum::{
    Json, Router,
    body::Body,
    response::Response,
};
use std::sync::Arc;

use deck::config::DeckConfig;
use crate::{
    auth, services, ship_runtime,
    sessions,
};
use crate::routes::{
    artifacts_api,
    hooks_api, human_webhook,
    rest_api, runtime_api,
    sandbox_api,
    webhook_ext,
};
use crate::auth::{attest, pairing, session};
use crate::transport::{cors, pty as pty_handler};
use axum::routing;

/// Build the full Axum router with all route groups, auth, and CORS.
pub fn build_router(
    rt: Arc<ship_runtime::ShipRuntime>,
    token: Arc<String>,
    session_registry: auth::SessionRegistry,
    pairing_registry: pairing::SharedPairingRegistry,
    attestation_verifier: attest::AttestationVerifier,
    daemon_mode: deck::DaemonMode,
) -> Router {
    use crate::routes::{Route, ShipRouterExt};
    let st = rt.clone();
    // Non-enumerated third-party routes (telegram webhook + hooks) stay on
    // `.route()` — they predate the canonical `Route` enum. Leave flat.
    let webhook_routes = Router::new()
        .route("/webhook/telegram", axum::routing::post(human_webhook::telegram_webhook))
        .with_state(rt.kernel().clone());
    let hook_routes = Router::new()
        .route("/hooks/session-start", axum::routing::post(hooks_api::hook_session_start))
        .route("/hooks/before-tool", axum::routing::post(hooks_api::hook_before_tool))
        .route("/hooks/after-tool", axum::routing::post(hooks_api::hook_after_tool))
        .route("/hooks/session-end", axum::routing::post(hooks_api::hook_session_end))
        .route("/hooks/permission-denied", axum::routing::post(hooks_api::hook_permission_denied))
        .with_state(st.clone());
    // Enumerated routes — one `ApiState`-typed router, registered via
    // `ship_route(Route::X, handler)` against the canonical full paths.
    let api_ship_routes = build_ship_routes(st);

    // ONE ShipDataPort for the whole router: deck's serve_index / serve_setup
    // read provisioning via `Extension<Arc<dyn DataPort>>` (injected at the
    // root layer below), and the multiplex transport reuses the SAME Arc.
    let data_port: Arc<dyn deck_core::DataPort> =
        Arc::new(crate::data_port::ShipDataPort::new(rt.clone()));

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
        // Drift-detection endpoint (/_deck/meta), the shared CSS/theme assets,
        // first-run `/` + `/setup`, the per-tag source viewer, and the root
        // public-asset catch-all are all owned by `deck::router()` now —
        // shipd serves none of the frontend surface. shipd keeps only RPC +
        // daemon routes here.
        // Bind the runtime state here so the resulting `Router<()>` merges
        // cleanly with the root assembly below.
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
    // Space-window registration retired: the launched window now calls
    // `DeviceService.RegisterWindow` (device.proto, Connect) over the host
    // socket — same pid-bound C2 mint, kernel-attested pid from `PeerPid`,
    // operator-only at the transport. The legacy `/api/space-window/register`
    // REST route + `auth::space_window` were removed in the space cutover.

    let session_issue_registry = auth_registry.clone();
    let session_issue_handler = move |auth: axum::Extension<auth::AuthContext>| {
        let reg = session_issue_registry.clone();
        async move { session::issue_session_handler(reg, auth.0).await }
    };

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
        // The authority-projection layer runs INSIDE auth_middleware (it's
        // applied here, so auth wraps it): it reads the populated AuthContext and
        // stamps the resolved `ship_types::Authority` into the request, so
        // crate-owned Connect wires (ship-account) gate on the shared authority
        // without coupling to shipd's AuthContext (S6 operator-cred gate).
        .merge(
            crate::ops_rpc::router(rt.clone())
                .layer(axum::middleware::from_fn(auth::project_authority_middleware)),
        )
        // Multiplex transport (deck-axum) — same auth-gated group as the
        // Connect ops above: the live-data reads are gated behind
        // `auth_middleware` (capability ship.events.read was enforced by the
        // deleted proto's policy options; the Connect ops' own gate still
        // applies for the typed wire paths, and the multiplex inherits the
        // same bearer requirement here). The MultiplexState is folded into
        // a single Extension: data_port + registry + change_source.
        .merge(
            deck::multiplex_router().layer(axum::Extension(deck::MultiplexState::new(
                data_port.clone(),
                Arc::new(deck_core::WindowSubscriptionRegistry::default()),
                Arc::new(crate::data_port::ShipChangeSource::new(rt.clone())),
            ))),
        )
        // Dry-run view interceptor runs INSIDE auth (applied before the
        // auth layer here, which means outer->inner order wraps it with
        // auth) so requests must carry a valid bearer before short-circuit.
        .layer(axum::middleware::from_fn(move |req, next| {
            auth::auth_middleware(auth_token.clone(), auth_registry.clone(), req, next)
        }));

    let root = Router::new()
        .merge(public_routes)
        // (framework bundles, catalogs, pack assets, per-view
        // template/setup/css/payload + sub-path, `/_deck/view/{name}`,
        // and `/space/{id}` + `/space/_assets`). Mounted
        // OUTSIDE the auth layer — these are static, no user-specific data,
        // same posture as the rest of `/_deck/*`. `Router<()>` (its handlers
        // read `Option<Extension<DaemonMode>>`, not `State`), so it merges
        // straight into the stateless root. deck-axum owns these paths.
        .merge(deck::router())
        .merge(webhook_source_route)
        .merge(authenticated_routes);
    // No `.fallback(spa)` here — that would CLOBBER the data plane. The
    // Connect-ops router (`ops_rpc::router().into_axum_service()`) dispatches the
    // entire `/ship.ops.v1.*` surface from a `fallback_service`; merging it into
    // `root` makes it root's fallback (carrying its auth layer). A
    // `.fallback(spa_fallback)` would silently replace it and 404 every RPC. SPA
    // deep-route navigation is instead handled by `spa_nav_middleware` (layered
    // below), which short-circuits GET client-route paths to the shell and lets
    // everything else — all RPC POSTs, unknown backend paths — flow through to
    // the Connect fallback. The dev-only HMR firehose and the `/{file}` public-
    // asset catch-all are owned by `deck::router()`; shipd serves no frontend.
    // Load deck.jsonc from the workspace root (same root the bundler uses).
    // Falls back to defaults when absent so a zero-config host works unchanged.
    let repo_root = rt.workspace_dir()
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| rt.ship_dir().to_path_buf());
    let deck_cfg = Arc::new(DeckConfig::load(&repo_root));

    // Build the DaemonBuildInfo once here so deck's serve_meta handler
    // (/_deck/meta) can read it via Option<Extension<DaemonBuildInfo>>.
    let dbi = {
        let bi = build_info::build_info!();
        deck::DaemonBuildInfo {
            cargo: env!("CARGO_PKG_VERSION"),
            git: bi.git_sha,
            dirty: bi.git_dirty,
            built_at: bi.timestamp,
        }
    };

    root
        // SPA-navigation shim. Layered INNER to the DaemonMode/DeckConfig
        // extensions below (they run first on the way in, so this can read them)
        // but it short-circuits before routing. Composes shell-navigation with
        // the Connect fallback: GET client-routes serve the shell, everything
        // else reaches the Connect dispatch. See the `root` comment above.
        .layer(axum::middleware::from_fn(spa_nav_middleware))
        // Daemon-mode Extension — Sui handlers read it via
        // `Option<Extension<DaemonMode>>` without router-state plumbing.
        .layer(axum::Extension(daemon_mode))
        // DataPort Extension — deck's serve_index / serve_setup read host
        // provisioning via `Option<Extension<Arc<dyn DataPort>>>`. Same Arc the
        // multiplex transport uses — ONE ShipDataPort for the whole router.
        .layer(axum::Extension(data_port))
        // DaemonBuildInfo Extension — deck's /_deck/meta handler reads this
        // to report the daemon's own build identity alongside the compiler's.
        .layer(axum::Extension(dbi))
        // DeckConfig Extension — scaffold handlers (serve_shell, serve_space)
        // read scaffold declarations + inject_globals from this.
        .layer(axum::Extension(deck_cfg))
        // Bundler Extension (#167) — deck's view.js handler builds an unbuilt
        // view in-process via Rolldown on a cache miss (controller inlined, no
        // bun). The same instance the watcher drives.
        .layer(axum::Extension(rt.bundler().clone()))
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

/// Pre-routing SPA-navigation shim. A GET to a client-route path — a bare in-app
/// route like `/git` or `/studio/themes` with no real server handler — is served
/// the shell scaffold so a deep-link refresh hydrates client-side. Everything
/// else flows through to routing, crucially the Connect-ops `fallback_service`
/// (`/ship.ops.v1.*`, the root fallback) which a `.fallback(spa)` handler would
/// otherwise shadow into a dead data plane. Runs INNER to the DaemonMode/
/// DeckConfig extension layers so it can read them for `serve_shell`.
async fn spa_nav_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    if req.method() == axum::http::Method::GET && is_client_nav(req.uri().path()) {
        let mode = req
            .extensions()
            .get::<deck::DaemonMode>()
            .cloned()
            .map(axum::Extension);
        let cfg = req
            .extensions()
            .get::<Arc<DeckConfig>>()
            .cloned()
            .map(axum::Extension);
        return deck::serve_shell(mode, cfg).await;
    }
    next.run(req).await
}

/// Whether `path` is an in-app client route that should hydrate as the shell,
/// as opposed to a real server route or asset. Excludes every backend prefix,
/// the first-run handlers (`/`, `/setup`), and dotted asset/RPC paths
/// (`/favicon.ico`, `/ship.ops.v1.*`). This runs BEFORE routing, so the
/// exclusion set must cover every real GET route — a miss shadows it.
fn is_client_nav(path: &str) -> bool {
    if path == "/" || path == "/setup" {
        return false;
    }
    const BACKEND: [&str; 8] = [
        "/api", "/_deck", "/hooks", "/auth", "/webhook", "/daemon", "/space", "/shell",
    ];
    if BACKEND.iter().any(|p| path.starts_with(p)) {
        return false;
    }
    // Dotted paths are assets (favicon.ico) or Connect RPC (`/ship.*`).
    !path.contains('.')
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
fn build_ship_routes(st: rest_api::ApiState) -> Router {
    use crate::routes::{Route, ShipRouterExt};
    Router::new()
        // Capabilities (no state required; ok against a stateful router)
        .ship_route(Route::Capabilities, rest_api::capabilities)
        // Isolation reads are served by the proto IsolationService
        // (ship.ops.v1) — the one op-surface path. The former
        // /api/v1/isolation/* REST routes were removed to avoid a parallel
        // path; init's gated-op migration lands at the connect cutover.
        .ship_route(Route::SandboxList, sandbox_api::sandbox_list)
        .ship_route(Route::SandboxGet_template(), sandbox_api::sandbox_get)
        // Runtime — read-side only (daemon status, projections, event stream).
        // Agent stop/steer, pipeline dispatch, job lifecycle, log stream, and
        // pipeline run log stream all moved to typed Connect wires
        // (AgentsService / PipelinesService / agent.Dispatch).
        .ship_route(Route::DaemonStatus, runtime_api::daemon_status)
        // View schema (ship.*) — read-side. Manifest / bundle / file-list /
        // write paths moved to DiscoveryService / Connect view-edit wire
        // (view-sessions via ViewService; edit-mode grants via
        // ship_views::view_authoring).
        .ship_route(Route::ShipSchema, services::schema::serve_schema)
        // Sessions (PTY).
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
        // `POST /_deck/mutation/{name}` — tombstoned PE form endpoint.
        // The server-side string dispatch was removed; JS-attached mutations
        // call the SDK directly. The SUI compiler still writes this URL as
        // the form `action` attribute for PE fallback, so the route stays as
        // a 410 Gone rather than being deleted outright.
        .route(
            "/_deck/mutation/{name}",
            axum::routing::post(|| async { axum::http::StatusCode::GONE }),
        )
        // AgentsService — generic CRUD + lifecycle. First-party and
        // third-party views use this surface equally.
        // PipelineService — CRUD + run ops. Dispatch / cancel / log-stream
        // wrap the existing kernel-backed handlers so the write paths stay
        // single-sourced.
        // RuleService — markdown rules.
        // PermissionService — preset entries + composed tool-patterns.
        // SkillService — service-shaped CRUD + file tree + vars. Replaces
        // the project-scoped skill endpoints that used to hang off
        // /api/v1/projects/{id}/skills/*.
        // Specs REST
        // Events / projections
        .ship_route(Route::EventStream, runtime_api::event_stream)
        .ship_route(Route::RebuildProjections, runtime_api::rebuild_projections)
        .ship_route(Route::ProjectPty_template(), pty_handler::workspace_pty)
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

#[cfg(test)]
mod merge_tests {
    //! Guards the deck-axum router mount (`root.merge(deck::router())`).
    //!
    //! axum performs route-conflict detection eagerly during construction
    //! and at `into_make_service()`. The risk in mounting deck's whole
    //! surface is its catch-all `/_deck/views/{name}/{*sub_path}` coexisting
    //! with the state-aware static `/_deck/views/{name}/sse` that shipd keeps
    //! in `public_routes`. This reproduces that union (plus the new
    //! `/space/*` paths) and forces the conflict check — a panic here is the
    //! regression we'd otherwise only catch at daemon boot.

    use axum::{Router, routing::get};

    async fn noop() -> &'static str {
        "ok"
    }

    #[test]
    fn deck_router_merges_with_kept_static_sui_routes_without_conflict() {
        // The state-aware paths shipd keeps in `public_routes` that share the
        // `/_deck/views/{name}/` prefix with deck's catch-all.
        let kept: Router = Router::new()
            .route("/_deck/views/{name}/sse", get(noop))
            .route("/_deck/mutation/{name}", axum::routing::post(noop));

        // Merging deck's surface (catch-all + /space + /_deck/view/{name} + bundles)
        // with the kept routes must not panic, and the union must build into
        // a service.
        let merged = Router::new().merge(deck::router()).merge(kept);
        let _service = merged.into_make_service();
    }

    /// Guards `is_client_nav` — the pre-routing classifier that decides whether
    /// a GET serves the shell (in-app route) or flows through to real routing
    /// (incl. the Connect `fallback_service`). A wrong answer here either
    /// shadows a real route or 404s the data plane — the exact regression the
    /// `spa_nav_middleware` exists to prevent.
    #[test]
    fn is_client_nav_routes_navigation_but_never_backend() {
        use super::is_client_nav;
        // In-app client routes → serve the shell.
        for p in ["/git", "/agents", "/studio/themes", "/settings/general"] {
            assert!(is_client_nav(p), "{p} should hydrate as the shell");
        }
        // Real server routes/prefixes → must reach their handlers, never shadowed.
        for p in [
            "/",                            // serve_index (first-run branch)
            "/setup",                       // serve_setup
            "/api/health",
            "/api/assets/view-tokens.css",
            "/_deck/views/git",
            "/_deck/runtime.js",
            "/shell/_assets/x.js",
            "/space/abc",
            "/auth/sessions",
            "/hooks/session-start",
            "/webhook/telegram",
            "/favicon.ico",                 // dotted asset
            "/ship.ops.v1.GitService/Overview", // Connect RPC (dotted)
        ] {
            assert!(!is_client_nav(p), "{p} must NOT be treated as client nav");
        }
    }
}
