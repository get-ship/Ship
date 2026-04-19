//! Ship daemon (shipd) library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon and
//! `network_status` / `network_stop` for CLI subcommands.

pub mod actor_api;
pub mod artifacts_api;
pub mod collection_api;
pub mod auth;
mod connections;
mod cors;
mod view_morph_target;
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
pub mod feedback_api;
pub mod mcp_apps_bridge;
mod server;
pub mod themes;
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
use runtime::events::daemon::{
    PortBindFailedPayload, ShutdownCompletedPayload, StartupCompletedPayload,
    event_types as daemon_events,
};
use runtime::events::{EventEnvelope, EventStore, SqliteEventStore};
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
    let init_started_at = std::time::Instant::now();
    daemon::cleanup_stale_daemon_files(&global_dir).map_err(|e| {
        eprintln!("\nshipd startup aborted: {e}\n");
        e
    })?;
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
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            let already_bound_by_pid = pidfile_pid_for_port(&global_dir, port);
            emit_daemon_event(
                &global_dir,
                daemon_events::DAEMON_PORT_BIND_FAILED,
                "daemon",
                &PortBindFailedPayload {
                    port,
                    already_bound_by_pid,
                    error: e.to_string(),
                },
            );
            return Err(anyhow!("bind {addr}: {e}"));
        }
    };

    let summary = runtime::db::event_log_summary(&global_dir);
    let startup = StartupCompletedPayload {
        pid: std::process::id(),
        port,
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit_sha: option_env!("VERGEN_GIT_SHA").unwrap_or("unknown").to_string(),
        event_count_at_startup: summary.event_count,
        latest_event_id: summary.latest_event_id,
        init_duration_ms: init_started_at.elapsed().as_millis() as u64,
    };
    if let Ok(envelope) =
        EventEnvelope::new(daemon_events::DAEMON_STARTUP_COMPLETED, "daemon", &startup)
    {
        let envelope = envelope.with_actor_id("daemon");
        if let Err(e) = rt.persist_and_route(&envelope).await {
            tracing::warn!("shipd: failed to emit daemon.startup_completed: {e}");
        }
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await.map_err(|e| anyhow!("server error: {e}"))?;

    let agents_drained = if let Some(dh) = rt.dispatch_handle() {
        let mut registry = dh.registry.lock().await;
        let count = registry.len();
        supervisor::graceful_shutdown(rt.kernel(), &mut registry, Arc::clone(&dh.executor)).await;
        count
    } else {
        0
    };
    let uptime_secs = rt.uptime().as_secs();
    rt.shutdown().await;
    emit_daemon_event(
        &global_dir,
        daemon_events::DAEMON_SHUTDOWN_COMPLETED,
        "daemon",
        &ShutdownCompletedPayload {
            pid: std::process::id(),
            uptime_secs,
            agents_drained,
            trigger: "sigterm".to_string(),
        },
    );
    Ok(())
}

#[cfg(test)]
mod collection_tests;
#[cfg(test)]
#[path = "artifacts_api_tests.rs"]
mod artifacts_api_tests;

/// Append a daemon.* event directly to platform.db.
///
/// Used in paths where the kernel may not be live (port bind failure) or where
/// the runtime is already torn down (post-serve shutdown). Fire-and-forget:
/// telemetry must never block lifecycle decisions.
fn emit_daemon_event<P: serde::Serialize>(
    global_dir: &std::path::Path,
    event_type: &str,
    entity_id: &str,
    payload: &P,
) {
    let envelope = match EventEnvelope::new(event_type, entity_id, payload) {
        Ok(e) => e.with_actor_id("daemon"),
        Err(e) => {
            tracing::warn!("shipd: cannot build {event_type} envelope: {e}");
            return;
        }
    };
    let store = match SqliteEventStore::new_at(global_dir) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("shipd: cannot open event store for {event_type}: {e}");
            return;
        }
    };
    if let Err(e) = store.append(&envelope) {
        tracing::warn!("shipd: cannot append {event_type}: {e}");
    }
}

/// Best-effort lookup of the pid currently holding `port`, sourced from the
/// supervisor's lock-based pidfile. Returns None if the file is missing,
/// unparseable, or records a different port.
fn pidfile_pid_for_port(global_dir: &std::path::Path, port: u16) -> Option<u32> {
    let path = global_dir.join("shipd.pid");
    let data = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(data.trim()).ok()?;
    let recorded_port = v.get("port")?.as_u64()? as u16;
    if recorded_port != port {
        return None;
    }
    Some(v.get("pid")?.as_u64()? as u32)
}
