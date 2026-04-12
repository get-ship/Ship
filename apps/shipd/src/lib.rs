//! Ship daemon (shipd) library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon and
//! `network_status` / `network_stop` for CLI subcommands.

pub mod actor_api;
pub mod auth;
mod connections;
pub mod scopes;
mod handler;
pub mod hooks_api;
pub mod human_webhook;
pub mod managers;
pub mod openapi;
pub mod project_api;
pub mod pty_handler;
pub mod rest_api;
pub mod runtime_api;
mod server;
pub mod settings_api;
pub mod attest;
pub mod pairing;
pub mod ship_runtime;
pub mod supervised;
pub mod supervisor;

#[cfg(test)]
pub mod test_harness;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
#[path = "auth_tests.rs"]
mod auth_tests;
#[cfg(test)]
#[path = "scope_tests.rs"]
mod scope_tests;
#[cfg(test)]
#[path = "ship_runtime_tests.rs"]
mod ship_runtime_tests;
#[cfg(test)]
#[path = "pairing_integration_tests.rs"]
mod pairing_integration_tests;

/// Test-only: build a minimal router with the CORS middleware for testing.
#[cfg(test)]
pub fn test_cors_router() -> Router {
    Router::new()
        .route("/test", axum::routing::get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(cors_middleware))
}

pub use server::NetworkServer;

use anyhow::{Result, anyhow};
use axum::{
    Json, Router,
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};
use utoipa::OpenApi;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Start the network daemon and block until shutdown (SIGINT/SIGTERM).
///
/// `dev` enables relaxed auth for local development (auto-pair, localhost-only).
/// Never set `dev = true` in production.
pub async fn run_network(host: String, port: u16, dev: bool) -> Result<()> {
    if dev {
        tracing::warn!(
            "╔══════════════════════════════════════════════════════════════════╗"
        );
        tracing::warn!(
            "║  SHIPD DEV MODE — auth relaxed, browser clients auto-pair        ║"
        );
        tracing::warn!(
            "║  Listening on localhost only. NOT FOR PRODUCTION.                ║"
        );
        tracing::warn!(
            "╚══════════════════════════════════════════════════════════════════╝"
        );
    }
    // Dev mode forces localhost bind. Agents can set SHIP_HOST env vars,
    // but they can't flip the --dev flag on the daemon process they didn't start.
    let host = if dev { "127.0.0.1".to_string() } else { host };
    pairing::set_dev_mode(dev);
    let global_dir = runtime::project::get_global_dir()?;
    cleanup_stale_daemon_files(&global_dir);
    let meta = ship_runtime::AppMetadata { global_dir: global_dir.clone() };
    let config = ship_runtime::RuntimeConfig { host: host.clone(), port, ..Default::default() };
    let mut runtime = ship_runtime::ShipRuntime::new(meta, config).await?;
    let mesh_registry = connections::spawn_mesh_service(runtime.kernel(), runtime.health_registry()).await?;
    runtime.set_mesh_registry(mesh_registry);
    let rt = Arc::new(runtime);

    supervisor::subscribe_workspace_events(rt.kernel().clone(), global_dir.clone(), rt.health_registry()).await;
    let job_mgr = managers::JobManager::init(rt.kernel().clone(), global_dir.clone(), rt.health_registry()).await;
    if let Some(dh) = job_mgr.dispatch_handle {
        rt.set_dispatch_handle(dh);
    }
    // Start the abandoned session scanner.
    rt.session_scanner().await;
    spawn_human_gateway(rt.kernel(), rt.health_registry()).await;

    let token = match auth::read_token(&global_dir) {
        Ok(existing) if !existing.is_empty() => Arc::new(existing),
        _ => {
            let new_token = auth::generate_token()
                .map_err(|e| anyhow!("failed to generate daemon token: {e}"))?;
            auth::write_token_file(&global_dir, &new_token).map_err(|e| anyhow!("write daemon.token: {e}"))?;
            Arc::new(new_token)
        }
    };
    let hook_token = Arc::new(auth::ensure_hook_token(&global_dir)?);
    write_port_file(&global_dir, port)?;
    write_pid_file(&global_dir)?;

    let ct = CancellationToken::new();
    let ct2 = ct.clone();
    tokio::spawn(async move { shutdown_signal().await; ct2.cancel(); });

    let session_registry = rt.session_registry().clone();
    // Hydrate any persisted paired sessions from a previous daemon lifetime.
    // Dev mode starts fresh — don't carry prior sessions across dev restarts.
    if !dev {
        auth::hydrate_sessions(&session_registry).await;
    }
    let pairing_registry = pairing::new_shared();
    let attestation_verifier = attest::AttestationVerifier::new();
    let app = build_router(
        rt.clone(),
        token,
        hook_token,
        session_registry,
        pairing_registry,
        attestation_verifier,
        ct.child_token(),
    );
    let addr = format!("{host}:{port}");
    tracing::info!("shipd: listening on http://{addr}/mcp");
    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| anyhow!("bind {addr}: {e}"))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await.map_err(|e| anyhow!("server error: {e}"))?;

    if let Some(dh) = rt.dispatch_handle() {
        supervisor::graceful_shutdown(rt.kernel(), &mut *dh.registry.lock().await, Arc::clone(&dh.executor)).await;
    }
    rt.shutdown().await;
    Ok(())
}

/// Build the full Axum router with all route groups, auth, and CORS.
fn build_router(
    rt: Arc<ship_runtime::ShipRuntime>,
    token: Arc<String>,
    hook_token: Arc<String>,
    session_registry: auth::SessionRegistry,
    pairing_registry: pairing::SharedPairingRegistry,
    attestation_verifier: attest::AttestationVerifier,
    mcp_ct: CancellationToken,
) -> Router {
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
    let webhook_routes = Router::new()
        .route("/webhook/telegram", axum::routing::post(human_webhook::telegram_webhook))
        .with_state(rt.kernel().clone());
    let hook_routes = Router::new()
        .route("/hooks/checkout", axum::routing::post(rest_api::hooks_checkout))
        .route("/hooks/session-start", axum::routing::post(hooks_api::hook_session_start))
        .route("/hooks/before-tool", axum::routing::post(hooks_api::hook_before_tool))
        .route("/hooks/after-tool", axum::routing::post(hooks_api::hook_after_tool))
        .route("/hooks/session-end", axum::routing::post(hooks_api::hook_session_end));
    let api_routes = build_api_routes(st.clone());
    let runtime_routes = build_runtime_routes(st.clone());
    let supervisor_routes = build_supervisor_routes(st.clone());
    let settings_routes = build_settings_routes(st.clone());
    let project_routes = build_project_routes(st);
    let capabilities_routes = Router::new()
        .route("/capabilities", axum::routing::get(rest_api::capabilities));

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
        .route("/api/session/register", axum::routing::post(move |body: axum::Json<auth::SessionRegisterRequest>| {
            auth::session_register(register_registry, body)
        }))
        .route("/api/health", axum::routing::get(move || {
            health_check(health_rt)
        }));

    let auth_token = token;
    let auth_hook_token = hook_token;
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
        );

    let authenticated_routes = Router::new()
        .nest("/api", pair_confirm_routes)
        .nest("/api", api_routes)
        .nest("/api", hook_routes)
        .nest("/api", webhook_routes)
        .nest("/api", capabilities_routes)
        .nest("/api/runtime", runtime_routes)
        .nest("/api", supervisor_routes)
        .nest("/api", settings_routes)
        .nest("/api", project_routes)
        .nest_service("/mcp", service)
        .layer(axum::middleware::from_fn(move |req, next| {
            auth::auth_middleware(auth_token.clone(), auth_hook_token.clone(), auth_registry.clone(), req, next)
        }));

    Router::new()
        .merge(public_routes)
        .route("/api/openapi.json", axum::routing::get(|| async {
            Json(openapi::ApiDoc::openapi())
        }))
        .merge(authenticated_routes)
        .layer(axum::middleware::from_fn(cors_middleware))
        // Request body limit: 1 MB for all non-streaming endpoints.
        .layer(tower_http::limit::RequestBodyLimitLayer::new(1024 * 1024))
        // Catch handler panics → 500 instead of connection abort.
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(handle_panic))
}

fn build_api_routes(st: rest_api::ApiState) -> Router {
    Router::new()
        .route("/mesh/register", axum::routing::post(rest_api::mesh_register))
        .route("/mesh/send", axum::routing::post(rest_api::mesh_send))
        .route("/mesh/broadcast", axum::routing::post(rest_api::mesh_broadcast))
        .route("/mesh/discover", axum::routing::get(rest_api::mesh_discover))
        .route("/mesh/status", axum::routing::post(rest_api::mesh_status_update))
        .route("/mesh/events/{agent_id}", axum::routing::get(rest_api::mesh_events))
        .route("/actor/spawn", axum::routing::post(actor_api::actor_spawn))
        .route("/events/route", axum::routing::post(actor_api::event_route))
        .with_state(st)
}

fn build_runtime_routes(st: rest_api::ApiState) -> Router {
    Router::new()
        .route("/workspaces", axum::routing::get(runtime_api::list_workspaces))
        .route("/sessions", axum::routing::get(runtime_api::list_sessions))
        .route("/agents", axum::routing::get(runtime_api::list_agents))
        .route("/agents/{slug}/stop", axum::routing::post(runtime_api::stop_agent))
        .route("/agents/{slug}/steer", axum::routing::post(runtime_api::steer_agent))
        .route("/jobs", axum::routing::get(runtime_api::list_jobs).post(runtime_api::create_job))
        .route("/jobs/end", axum::routing::post(runtime_api::end_job))
        .route("/views", axum::routing::get(runtime_api::list_views))
        .route("/view/{name}", axum::routing::get(runtime_api::serve_view))
        .route("/events", axum::routing::get(runtime_api::event_stream))
        .route("/rebuild-projections", axum::routing::post(runtime_api::rebuild_projections))
        .route("/workspaces/{id}/pty", axum::routing::get(pty_handler::workspace_pty))
        .with_state(st)
}

fn build_settings_routes(st: rest_api::ApiState) -> Router {
    Router::new()
        .route("/settings", axum::routing::get(settings_api::list_settings).put(settings_api::update_setting))
        .route("/settings/{key}", axum::routing::get(settings_api::get_setting))
        .route("/providers", axum::routing::get(settings_api::list_providers))
        .with_state(st)
}

fn build_supervisor_routes(st: rest_api::ApiState) -> Router {
    Router::new()
        .route("/supervisor/workspaces/{id}/start", axum::routing::post(supervisor::start_workspace))
        .with_state(st)
}

fn build_project_routes(st: rest_api::ApiState) -> Router {
    Router::new()
        .route("/workspaces/{id}/session-files", axum::routing::get(project_api::list_session_files))
        .route("/workspaces/{id}/session-files/{*path}", axum::routing::get(project_api::read_session_file).put(project_api::write_session_file).delete(project_api::delete_session_file))
        .route("/workspaces/{id}/git/status", axum::routing::get(project_api::git_status))
        .route("/workspaces/{id}/git/diff", axum::routing::get(project_api::git_diff))
        .route("/workspaces/{id}/git/log", axum::routing::get(project_api::git_log))
        .route("/workspaces/{id}/agents", axum::routing::get(project_api::list_agents).post(project_api::write_agent))
        .route("/workspaces/{id}/skills", axum::routing::get(project_api::list_skills))
        .route("/workspaces/{id}/skills/{skillId}/files/{*path}", axum::routing::put(project_api::write_skill_file).delete(project_api::delete_skill_file))
        .route("/workspaces/{id}/skills/{skillId}/vars", axum::routing::get(project_api::get_skill_vars))
        .route("/workspaces/{id}/skills/{skillId}/vars/{key}", axum::routing::put(project_api::set_skill_var))
        .route("/workspaces/{id}", axum::routing::delete(project_api::delete_workspace))
        .route("/workspaces/{id}/activate", axum::routing::post(project_api::activate_workspace))
        .route("/workspaces/{id}/pipelines", axum::routing::get(project_api::list_pipelines))
        .route("/workspaces/{id}/pipelines/{name}", axum::routing::put(project_api::write_pipeline).delete(project_api::delete_pipeline))
        .route("/project/git/status", axum::routing::get(project_api::project_git_status))
        .route("/project/git/diff", axum::routing::get(project_api::project_git_diff))
        .route("/project/git/log", axum::routing::get(project_api::project_git_log))
        .route("/events/emit", axum::routing::post(project_api::emit_event))
        .route("/workspaces/{id}/artifacts/status", axum::routing::get(project_api::artifacts_status))
        .route("/workspaces/{id}/artifacts/promote", axum::routing::post(project_api::promote_artifact))
        .route("/artifacts", axum::routing::get(project_api::list_project_artifacts))
        .with_state(st)
}

/// Check whether the network daemon is running by inspecting ~/.ship/daemon.port.
pub fn network_status() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    match std::fs::read_to_string(global_dir.join("daemon.port")) {
        Ok(port) => println!("shipd: running on port {}", port.trim()),
        Err(_) => println!("shipd: not running"),
    }
    Ok(())
}

/// Send SIGTERM to the network daemon via the stored PID file.
pub fn network_stop() -> Result<()> {
    let global_dir = runtime::project::get_global_dir()?;
    let pid_file = global_dir.join("daemon.pid");
    let pid_str = std::fs::read_to_string(&pid_file)
        .map_err(|_| anyhow!("shipd is not running (no pid file)"))?;
    let pid = pid_str.trim().to_string();
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .map_err(|e| anyhow!("failed to send SIGTERM to pid {pid}: {e}"))?;
        println!("shipd: sent SIGTERM to pid {pid}");
    }
    #[cfg(not(unix))]
    anyhow::bail!("shipd stop is not supported on this platform");
    Ok(())
}

/// Spawn `HumanGatewayService` if Telegram env vars are present.
/// Silently skips if `SHIP_TELEGRAM_TOKEN` or `SHIP_TELEGRAM_CHAT_ID` is absent.
async fn spawn_human_gateway(
    kernel: &Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>,
    health_registry: &supervised::HealthRegistry,
) {
    use runtime::services::human_gateway::{HumanGatewayService, TelegramAdapter};
    use runtime::services::spawn_service;
    use runtime::events::ActorConfig;
    use std::sync::Arc as StdArc;

    let adapter = match TelegramAdapter::from_env() {
        Some(a) => a,
        None => {
            tracing::info!("shipd: SHIP_TELEGRAM_TOKEN/CHAT_ID not set — human gateway disabled");
            return;
        }
    };

    let (outbox_tx, mut outbox_rx) =
        tokio::sync::mpsc::unbounded_channel::<runtime::events::EventEnvelope>();

    let config = ActorConfig {
        namespace: "service.human-gateway".to_string(),
        write_namespaces: vec!["human.".to_string()],
        read_namespaces: vec!["job.".to_string()],
        subscribe_namespaces: vec!["job.".to_string()],
    };

    let handler: Box<dyn runtime::services::ServiceHandler> = Box::new(
        HumanGatewayService::new(StdArc::new(adapter), outbox_tx),
    );

    match spawn_service(&mut *kernel.lock().await, "service.human-gateway", config, handler) {
        Ok(_) => tracing::info!("shipd: human gateway service started (telegram)"),
        Err(e) => {
            tracing::warn!("shipd: failed to spawn human gateway: {e}");
            return;
        }
    }

    // Drain outbox → kernel (currently unused, reserved for future adapter replies)
    let kr = kernel.clone();
    supervised::spawn_monitored(health_registry, "human-gateway-outbox", async move {
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            workspace_id: None,
            session_id: None,
        };
        while let Some(event) = outbox_rx.recv().await {
            if let Err(e) = kr.lock().await.route(event, &ctx).await {
                tracing::warn!("human-gateway outbox routing error: {e}");
            }
        }
    });
}

/// Convert a handler panic into a structured 500 response instead of
/// aborting the connection. Logs the panic for diagnostics.
fn handle_panic(info: Box<dyn std::any::Any + Send + 'static>) -> Response {
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

/// Clean up stale daemon files from a previous crashed instance.
///
/// If `daemon.pid` exists but the process is dead, remove PID, port, and token
/// files so the new instance starts cleanly.
fn cleanup_stale_daemon_files(global_dir: &Path) {
    let pid_file = global_dir.join("daemon.pid");
    let pid_str = match std::fs::read_to_string(&pid_file) {
        Ok(s) => s,
        Err(_) => return, // No PID file — nothing to clean.
    };

    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Corrupt PID file — remove it.
            let _ = std::fs::remove_file(&pid_file);
            return;
        }
    };

    // Check if the old process is still alive.
    #[cfg(unix)]
    let alive = {
        // signal 0 checks existence without sending a real signal.
        unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
    };
    #[cfg(not(unix))]
    let alive = false;

    if alive && pid != std::process::id() {
        tracing::warn!(
            old_pid = pid,
            "shipd: another daemon is still running — aborting stale cleanup"
        );
        return;
    }

    // Old process is dead (or is us from a previous failed start). Clean up.
    tracing::info!(old_pid = pid, "shipd: cleaning up stale daemon files from crashed instance");
    let _ = std::fs::remove_file(global_dir.join("daemon.pid"));
    let _ = std::fs::remove_file(global_dir.join("daemon.port"));
    // Keep daemon.token — reuse across restarts for stability.
}

#[cfg(test)]
mod stale_cleanup_tests {
    use super::cleanup_stale_daemon_files;
    use tempfile::TempDir;

    #[test]
    fn no_pid_file_is_noop() {
        let dir = TempDir::new().unwrap();
        // Should not panic or error.
        cleanup_stale_daemon_files(dir.path());
    }

    #[test]
    fn corrupt_pid_file_gets_removed() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("daemon.pid"), "not-a-number").unwrap();
        cleanup_stale_daemon_files(dir.path());
        assert!(!dir.path().join("daemon.pid").exists());
    }

    #[test]
    fn stale_pid_cleans_up_pid_and_port() {
        let dir = TempDir::new().unwrap();
        // PID 99999999 is almost certainly not running.
        std::fs::write(dir.path().join("daemon.pid"), "99999999").unwrap();
        std::fs::write(dir.path().join("daemon.port"), "9315").unwrap();
        std::fs::write(dir.path().join("daemon.token"), "abc123").unwrap();

        cleanup_stale_daemon_files(dir.path());

        assert!(!dir.path().join("daemon.pid").exists(), "PID file should be removed");
        assert!(!dir.path().join("daemon.port").exists(), "port file should be removed");
        assert!(dir.path().join("daemon.token").exists(), "token should be preserved");
    }
}

/// GET /api/health — unauthenticated health check.
///
/// Returns daemon uptime, PID, and status of all supervised background tasks.
/// Any dead task sets the top-level `healthy` flag to false.
async fn health_check(rt: Arc<ship_runtime::ShipRuntime>) -> Json<serde_json::Value> {
    let tasks = rt.health_registry().snapshot();
    let all_healthy = tasks.iter().all(|t| t.alive);
    Json(serde_json::json!({
        "healthy": all_healthy,
        "pid": std::process::id(),
        "dev_mode": pairing::is_dev_mode(),
        "tasks": tasks,
    }))
}

fn write_port_file(global_dir: &Path, port: u16) -> Result<()> {
    std::fs::write(global_dir.join("daemon.port"), port.to_string())
        .map_err(|e| anyhow!("failed to write daemon.port: {e}"))
}

fn write_pid_file(global_dir: &Path) -> Result<()> {
    let pid = std::process::id();
    std::fs::write(global_dir.join("daemon.pid"), pid.to_string())
        .map_err(|e| anyhow!("failed to write daemon.pid: {e}"))
}

/// Wait for SIGINT or SIGTERM (UNIX) or just SIGINT (other platforms).
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut sigterm) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = sigterm.recv() => {}
                }
            }
            Err(e) => {
                // SIGTERM handler unavailable — fall back to SIGINT only
                tracing::warn!("could not register SIGTERM handler: {e}; SIGINT only");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

/// Allowed origins for CORS. Localhost ports (dev) and getship.dev (production).
/// Wildcard is intentionally excluded — only known clients may connect.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://getship.dev",
    "http://localhost:3000",
    "http://localhost:5173",
    "http://localhost:4321",
];

async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
    use axum::http::{HeaderValue, Method};

    let origin = req
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let allowed = ALLOWED_ORIGINS.contains(&origin)
        || origin.starts_with("http://localhost:")
        || origin.starts_with("http://127.0.0.1:");

    if !allowed && !origin.is_empty() {
        tracing::warn!("shipd: rejected CORS request from origin: {origin}");
    }

    let origin_header = if allowed && !origin.is_empty() {
        origin.to_string()
    } else {
        // Non-browser clients (curl, CLI) send no Origin — allow through without ACAO header.
        // Browsers with a disallowed origin are rejected by the browser itself.
        String::new()
    };

    if req.method() == Method::OPTIONS {
        let mut res = Response::new(Body::empty());
        let h = res.headers_mut();
        if !origin_header.is_empty() {
            if let Ok(val) = HeaderValue::from_str(&origin_header) {
                h.insert("access-control-allow-origin", val);
            }
        }
        h.insert(
            "access-control-allow-methods",
            HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
        );
        h.insert(
            "access-control-allow-headers",
            HeaderValue::from_static("content-type, authorization, accept, mcp-session-id, x-ship-caller, x-ship-agent-id, x-ship-workspace-id, x-ship-session-id"),
        );
        h.insert(
            "access-control-expose-headers",
            HeaderValue::from_static("mcp-session-id"),
        );
        h.insert(
            "access-control-allow-credentials",
            HeaderValue::from_static("true"),
        );
        return res;
    }

    let mut res = next.run(req).await;
    let h = res.headers_mut();
    if !origin_header.is_empty() {
        if let Ok(val) = HeaderValue::from_str(&origin_header) {
            h.insert("access-control-allow-origin", val);
        }
    }
    h.insert(
        "access-control-expose-headers",
        HeaderValue::from_static("mcp-session-id"),
    );
    res
}
