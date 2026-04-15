//! Ship daemon (shipd) library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon and
//! `network_status` / `network_stop` for CLI subcommands.

pub mod actor_api;
pub mod collection_api;
pub mod auth;
mod connections;
mod cors;
mod daemon;
pub mod scopes;
pub mod session;
mod handler;
pub mod hooks_api;
pub mod human_webhook;
mod human_gateway;
pub mod managers;
pub mod openapi;
pub mod project_api;
pub mod pty_handler;
pub mod rest_api;
pub mod routes;
pub mod router;
pub mod runtime_api;
pub mod view_sessions_api;
pub mod annotations_api;
pub mod mcp_apps_bridge;
mod server;
pub mod views;
pub mod views_raw;
pub mod views_git;
pub mod views_git_data;
pub mod views_git_diff;
pub mod views_pipelines;
pub mod views_pipelines_data;
pub mod views_settings;
pub mod views_agents;
pub mod views_skills;
pub mod views_chat;
pub mod views_events;
pub mod specs_api;
pub mod specs_view;
pub mod settings_api;
pub mod attest;
pub mod pairing;
pub mod shell_assets;
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

pub use daemon::{network_status, network_stop};
pub use server::NetworkServer;

/// Test-only: build a minimal router with the CORS middleware for testing.
#[cfg(test)]
pub fn test_cors_router() -> axum::Router {
    axum::Router::new()
        .route("/test", axum::routing::get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(cors::cors_middleware))
}

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Start the network daemon and block until shutdown (SIGINT/SIGTERM).
///
/// `dev` enables relaxed auth for local development (auto-pair, localhost-only).
/// Never set `dev = true` in production.
pub async fn run_network(
    host: String,
    port: u16,
    dev: bool,
    with_ui: bool,
    workspace_dir: Option<std::path::PathBuf>,
) -> Result<()> {
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
    daemon::cleanup_stale_daemon_files(&global_dir);
    if let Some(ref w) = workspace_dir {
        tracing::info!(workspace = %w.display(), "shipd: project-scoped mode");
    } else {
        tracing::info!("shipd: global-only mode (no --workspace given)");
    }
    let meta = ship_runtime::AppMetadata {
        global_dir: global_dir.clone(),
        workspace_dir: workspace_dir.clone(),
    };
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
    human_gateway::spawn_human_gateway(rt.kernel(), rt.health_registry()).await;

    let token = match auth::read_token(&global_dir) {
        Ok(existing) if !existing.is_empty() => Arc::new(existing),
        _ => {
            let new_token = auth::generate_token()
                .map_err(|e| anyhow!("failed to generate daemon token: {e}"))?;
            auth::write_token_file(&global_dir, &new_token).map_err(|e| anyhow!("write daemon.token: {e}"))?;
            Arc::new(new_token)
        }
    };
    daemon::write_port_file(&global_dir, port)?;
    daemon::write_pid_file(&global_dir)?;

    let ct = CancellationToken::new();
    let ct2 = ct.clone();
    tokio::spawn(async move { daemon::shutdown_signal().await; ct2.cancel(); });

    let session_registry = rt.session_registry().clone();
    // Hydrate any persisted paired sessions from a previous daemon lifetime.
    // Dev mode starts fresh — don't carry prior sessions across dev restarts.
    if !dev {
        auth::hydrate_sessions(&session_registry).await;
    }
    let pairing_registry = pairing::new_shared();
    let attestation_verifier = attest::AttestationVerifier::new();
    let app = router::build_router(
        rt.clone(),
        token,
        session_registry,
        pairing_registry,
        attestation_verifier,
        ct.child_token(),
        with_ui,
    );
    let addr = format!("{host}:{port}");
    tracing::info!("shipd: listening on http://{addr}/mcp");
    if with_ui {
        tracing::info!("shipd: shell: http://{addr}/shell/");
        eprintln!("shell: http://{addr}/shell/");
    }
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

#[cfg(test)]
mod collection_tests;
