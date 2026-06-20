//! Axum router assembly: public + authenticated route groups, enumerated ship
//! routes, panic handler, and health check.
//!
//! Split (file-cap): `ship_routes` binds the enumerated `/api/v1/*` group,
//! `middleware` carries the SPA-nav shim + build-skew header, `health` the
//! health check + panic handler. This module owns the assembly.

pub(super) mod apps_config;
mod health;
mod middleware;
mod ship_routes;

use axum::Router;
use std::sync::Arc;

use deck::config::DeckConfig;
use crate::{auth, ship_runtime};
use crate::routes::{human_webhook, webhook_ext};
#[cfg(feature = "ship-code")]
use crate::routes::hooks_api;
use crate::auth::{attest, pairing, session};
use crate::transport::cors;

use health::{handle_panic, health_check};
use middleware::{build_header_middleware, spa_nav_middleware};
use ship_routes::build_ship_routes;

/// Build the full Axum router with all route groups, auth, and CORS.
///
/// `apps` is the per-app serve config (read from `apps.jsonc` at
/// startup) and injected as `Extension<Vec<AppServeConfig>>`. This is
/// the only app-serve path — the old scaffold-by-name API and the
/// `DeckConfig::scaffolds` field it used were removed at the per-app
/// cutover.
pub fn build_router(
    rt: Arc<ship_runtime::ShipRuntime>,
    token: Arc<String>,
    session_registry: auth::SessionRegistry,
    pairing_registry: pairing::SharedPairingRegistry,
    attestation_verifier: attest::AttestationVerifier,
    daemon_mode: deck::DaemonMode,
    apps: Vec<deck::AppServeConfig>,
) -> Router {
    use crate::routes::{Route, ShipRouterExt};
    let st = rt.clone();
    // Non-enumerated third-party routes (telegram webhook + hooks) stay on
    // `.route()` — they predate the canonical `Route` enum. Leave flat.
    let webhook_routes = Router::new()
        .route("/webhook/telegram", axum::routing::post(human_webhook::telegram_webhook))
        .with_state(rt.kernel().clone());
    #[cfg(feature = "ship-code")]
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
        Arc::new(crate::ports::data::ShipDataPort::new(rt.clone()));

    // Synthetic ship-js SSR (gated). Compiled in via the `ship-js-ssr` feature
    // (keeps V8 out of the default build) and activated at runtime by
    // DECK_SSR=ship-js. Register the renderer (isolate pool + DataPort) only when
    // the flag is set, so V8 is not spun up otherwise; deck-core's serve seam
    // falls back to render-eval whenever no renderer is registered.
    #[cfg(feature = "ship-js-ssr")]
    if deck_core::synthetic_ssr::enabled() {
        match deck_ssr::serve::register(Some(data_port.clone())) {
            Ok(()) => {
                tracing::info!("deck-ssr: synthetic ship-js SSR registered (DECK_SSR=ship-js)")
            }
            Err(e) => {
                tracing::warn!(error = %e, "deck-ssr: SSR registration failed; serving via render-eval")
            }
        }
    }

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
        // Gated: hooks_api uses ship-agents PTY session state (ship-code feature).
        ;
    #[cfg(feature = "ship-code")]
    let authenticated_routes = authenticated_routes.merge(hook_routes);
    let authenticated_routes = authenticated_routes
        .nest("/api", webhook_routes)
        // ConnectRPC op surface (ship.ops.v1) — additive, wrapped by
        // auth_middleware below so the gate sees AuthContext. See rpc::ops.
        // The authority-projection layer runs INSIDE auth_middleware (it's
        // applied here, so auth wraps it): it reads the populated AuthContext and
        // stamps the resolved `ship_types::Authority` into the request, so
        // crate-owned Connect wires (ship-account) gate on the shared authority
        // without coupling to shipd's AuthContext (S6 operator-cred gate).
        .merge(
            crate::rpc::ops::router(rt.clone())
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
                Arc::new(crate::ports::data::ShipChangeSource::new(rt.clone())),
            ))),
        )
        // Dry-run view interceptor runs INSIDE auth (applied before the
        // auth layer here, which means outer->inner order wraps it with
        // auth) so requests must carry a valid bearer before short-circuit.
        .layer(axum::middleware::from_fn(move |req, next| {
            auth::auth_middleware(auth_token.clone(), auth_registry.clone(), req, next)
        }))
        // Top-level DataPort Extension — deck's serve_index / serve_setup
        // read provisioning via `Option<Extension<Arc<dyn DataPort>>>`
        // and need a top-level extension (the per-multiplex MultiplexState
        // wrapper doesn't satisfy the type). Same Arc the multiplex
        // state holds; no extra cost.
        .layer(axum::Extension(data_port.clone()));
    // The per-app router is merged into the root BEFORE the auth layer
    // (apps are unauthenticated public surfaces; the auth middleware
    // above skips them via the path-prefix checks in the SPA-nav shim).
    // `serve_apps` reads the per-app list from `Extension<Arc<Vec<...>>>`
    // and mounts `<app.route>` + `/<app.name>/{file}` + the
    // primary app's `/{file}` catch-all. The L1 framework (`deck::router`)
    // still owns the framework surface (`/_deck/*` + `/api/*`); per-app
    // serving is host-driven from this point.
    let per_app = deck::serve_apps(&apps);
    let apps_ext = Arc::new(apps);
    let root = Router::new()
        .merge(public_routes)
        .merge(deck::pages::serve_index_route())
        .merge(deck::pages::serve_setup_route())
        .merge(per_app)
        .merge(deck::router())
        .merge(webhook_source_route)
        .merge(authenticated_routes)
        .layer(axum::Extension(apps_ext.clone()))
        // Top-level DataPort Extension — same Arc as the multiplex's
        // MultiplexState. deck's serve_index / serve_setup read this via
        // `Option<Extension<Arc<dyn DataPort>>>` to drive the first-run
        // flow (provisioned → serve the primary app, not provisioned →
        // redirect to /setup). Adding it here, on `root`, covers every
        // request — the per-app routes, the public catch-all, and the
        // multiplex.
        .layer(axum::Extension(data_port.clone()));
    // entire `/ship.ops.v1.*` surface from a `fallback_service`; merging it into
    // `root` makes it root's fallback (carrying its auth layer). A
    // `.fallback(spa_fallback)` would silently replace it and 404 every RPC. SPA
    // deep-route navigation is instead handled by `spa_nav_middleware` (layered
    // below), which short-circuits GET client-route paths to the shell and lets
    // everything else — all RPC POSTs, unknown backend paths — flow through to
    // the Connect fallback. The dev-only HMR firehose and the `/{file}` public-
    // asset catch-all are owned by `deck::router()`; shipd serves no frontend.
    // Load deck.jsonc from the workspace root (same root the bundler uses)
    // for the L1 bundle config — consumer targets, source roots, component
    // source roots. It carries NO app/serve list: the per-app serve config
    // comes from apps.jsonc (the `Vec<AppServeConfig>` the caller already
    // read and passes in). deck.jsonc and the app list are deliberately
    // separate surfaces — L1 is per-app; the host owns the list.
    let repo_root = rt.workspace_dir()
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| rt.ship_dir().to_path_buf());
    let deck_cfg = Arc::new(DeckConfig::load(&repo_root));
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
        .layer(axum::Extension(apps_ext.clone()))
        // Daemon-mode Extension — Sui handlers read it via
        // `Option<Extension<DaemonMode>>` without router-state plumbing.
        .layer(axum::Extension(daemon_mode))
        // DataPort Extension — deck's serve_index / serve_setup read host
        // provisioning via `Option<Extension<Arc<dyn DataPort>>>`. Same Arc the
        // DeckConfig Extension — scaffold handlers (serve_shell, serve_space)
        // read scaffold declarations + inject_globals from this.
        .layer(axum::Extension(deck_cfg))
        // DaemonBuildInfo Extension — deck's /_deck/meta handler reads this
        // to report the daemon's own build identity alongside the compiler's.
        .layer(axum::Extension(dbi))
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
        // The state-aware path shipd keeps in `public_routes` that shares the
        // `/_deck/views/{name}/` prefix with deck's catch-all. (The
        // `/_deck/mutation/{name}` tombstone now lives in `deck::router()`, so
        // it must NOT be re-declared here or the merge would conflict.)
        let kept: Router = Router::new()
            .route("/_deck/views/{name}/sse", get(noop));

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
        use super::middleware::is_client_nav;
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
            "/ship.ops.v1.AgentsService/Summaries", // Connect RPC (dotted)
        ] {
            assert!(!is_client_nav(p), "{p} must NOT be treated as client nav");
        }
    }
}
