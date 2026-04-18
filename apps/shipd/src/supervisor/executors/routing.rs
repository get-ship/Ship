//! RoutingExecutor — per-job selection between Server, TmuxWindow, and
//! Process executors.
//!
//! The orchestrator receives a single `Arc<dyn JobExecutor>`, but we
//! want the routing decision (server-mode session vs. interactive tmux
//! vs. headless child process) made **per job** based on (a) the
//! provider's declared capabilities and (b) the `headless` flag that
//! flows in from `ship run --headless` / the API.
//!
//! Routing precedence (highest first):
//!
//! 1. **Server mode** — when `adapter.caps().server_mode == true` AND
//!    the session registry has a bound session for that provider AND
//!    the job isn't `headless`-forcing a process. Currently the
//!    registry is empty for every provider (scaffolding slice); real
//!    sessions bind in `provider-codex-02` / `provider-opencode-02`.
//! 2. **Process** — when `ctx.headless == true` or TmuxWindow is
//!    unavailable (Windows, tmux missing).
//! 3. **TmuxWindow** — the default for interactive-attach pipelines.
//!
//! Lifecycle calls (`is_alive`, `stop`, `send`) dispatch based on the
//! opaque handle state: Server/TmuxWindow/Process each use a distinct
//! state struct, so `downcast_ref` already routes correctly.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use runtime::events::KernelRouter;
use runtime::services::dispatch_ports::{ExecutorHandle, JobContext, JobExecutor};
use runtime::services::provider_adapter::adapter_for_or_default;
use runtime::services::providers::codex_session::CodexSession;
use runtime::services::server_session::ServerSession;
use tokio::sync::Mutex;

use super::{ProcessExecutor, ServerExecutor, TmuxWindowExecutor};

/// In-memory registry mapping provider id -> bound `ServerSession`.
///
/// Scaffolding: empty in this slice. `provider-codex-02` /
/// `provider-opencode-02` will populate it at daemon startup once
/// codex/opencode server processes are spawned. The registry is an
/// `Arc`-wrapped `std::sync::Mutex` over a `HashMap` — lookups are
/// lock-light and rare (once per spawn).
#[derive(Default, Clone)]
pub(crate) struct ServerSessionRegistry {
    inner: Arc<std::sync::Mutex<HashMap<String, Arc<dyn ServerSession>>>>,
}

impl ServerSessionRegistry {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Look up the session bound for a provider id. Returns `None` when
    /// no session is bound — routing falls through to Process/TmuxWindow.
    pub(crate) fn get(&self, provider: &str) -> Option<Arc<dyn ServerSession>> {
        self.inner.lock().ok()?.get(provider).cloned()
    }

    /// Bind a session for a provider id. Replaces any prior binding.
    /// Used by daemon startup and by tests to install a stub session.
    #[allow(dead_code)] // Used by tests + future provider-codex-02/provider-opencode-02.
    pub(crate) fn bind(&self, provider: impl Into<String>, session: Arc<dyn ServerSession>) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(provider.into(), session);
        }
    }
}

/// Routes each `spawn` to Server, TmuxWindow, or Process per the rules
/// in the module docs.
pub(crate) struct RoutingExecutor {
    kernel: Option<Arc<Mutex<KernelRouter>>>,
    tmux_window: Option<TmuxWindowExecutor>,
    process: ProcessExecutor,
    sessions: ServerSessionRegistry,
}

impl RoutingExecutor {
    pub fn new(kernel: Option<Arc<Mutex<KernelRouter>>>) -> Self {
        Self::with_registry(kernel, ServerSessionRegistry::new())
    }

    /// Construct with a pre-built session registry — used by tests to
    /// inject stub sessions and by future daemon wiring once real
    /// sessions are bound during supervisor startup.
    pub(crate) fn with_registry(
        kernel: Option<Arc<Mutex<KernelRouter>>>,
        sessions: ServerSessionRegistry,
    ) -> Self {
        let tmux_window = match TmuxWindowExecutor::new() {
            Ok(e) => Some(e),
            Err(e) => {
                tracing::warn!(
                    "job-dispatch: TmuxWindowExecutor unavailable ({e}); falling \
                     back to ProcessExecutor for all non-server jobs"
                );
                None
            }
        };
        Self {
            kernel: kernel.clone(),
            tmux_window,
            process: ProcessExecutor::new(kernel),
            sessions,
        }
    }

    /// Returns a `ServerExecutor` if the provider has `server_mode` AND
    /// a session is bound. Otherwise `None` — caller falls through to
    /// the existing Process/TmuxWindow selection.
    fn try_server_executor(&self, ctx: &JobContext) -> Option<ServerExecutor> {
        let adapter = adapter_for_or_default(ctx.launch.provider.as_deref()).ok()?;
        if !adapter.caps().server_mode {
            return None;
        }
        let session = self.sessions.get(adapter.id())?;
        Some(ServerExecutor::new(self.kernel.clone(), session))
    }

    /// Lazy-spawn a `CodexSession` on first codex dispatch, caching the
    /// binding in the session registry. On spawn failure (binary missing,
    /// handshake stalls, etc.) we log a warning and leave the registry
    /// untouched — the caller then falls through to Process/TmuxWindow,
    /// preserving the one-shot `codex exec` path as graceful degradation.
    async fn ensure_codex_session(&self, ctx: &JobContext) {
        let Some(provider) = ctx.launch.provider.as_deref() else {
            return;
        };
        if provider != "codex" {
            return;
        }
        if self.sessions.get("codex").is_some() {
            return;
        }
        if !codex_binary_on_path() {
            tracing::debug!(
                "routing: codex binary not on PATH; skipping CodexSession lazy-spawn"
            );
            return;
        }
        match CodexSession::spawn().await {
            Ok(session) => {
                tracing::info!("routing: lazy-spawned CodexSession for provider=codex");
                self.sessions.bind("codex", Arc::new(session));
            }
            Err(err) => {
                tracing::warn!(%err,
                    "routing: CodexSession lazy-spawn failed; falling back to ProcessExecutor");
            }
        }
    }

    /// Test-only inspection: which executor would `spawn` dispatch to,
    /// given this context? Avoids actually spawning a provider process
    /// in routing tests.
    #[cfg(test)]
    pub(crate) fn route_kind(&self, ctx: &JobContext) -> RouteKind {
        if self.try_server_executor(ctx).is_some() {
            return RouteKind::Server;
        }
        if !ctx.headless && self.tmux_window.is_some() {
            return RouteKind::TmuxWindow;
        }
        RouteKind::Process
    }
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RouteKind {
    Server,
    TmuxWindow,
    Process,
}

#[async_trait]
impl JobExecutor for RoutingExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        // Codex routes to a persistent app-server session when available.
        // The first codex dispatch lazy-spawns the child; subsequent
        // dispatches reuse the cached session Arc via the registry.
        self.ensure_codex_session(ctx).await;

        if let Some(server) = self.try_server_executor(ctx) {
            tracing::info!(slug = ctx.slug,
                provider = ?ctx.launch.provider,
                "routing: dispatching to ServerExecutor");
            return server.spawn(ctx).await;
        }
        if !ctx.headless
            && let Some(ref tmux_window) = self.tmux_window
        {
            return tmux_window.spawn(ctx).await;
        }
        self.process.spawn(ctx).await
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        // Ask each in turn; whichever owns the handle state answers.
        // Server handles downcast to ServerState, TmuxWindow to
        // TmuxWindowState, Process to ProcessState — no overlap.
        if let Some(server) = server_for_handle(self, handle)
            && server.is_alive(handle).await
        {
            return true;
        }
        if let Some(ref tmux_window) = self.tmux_window
            && tmux_window.is_alive(handle).await
        {
            return true;
        }
        self.process.is_alive(handle).await
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        if let Some(server) = server_for_handle(self, handle) {
            return server.stop(handle).await;
        }
        if let Some(ref tmux_window) = self.tmux_window {
            let _ = tmux_window.stop(handle).await;
        }
        self.process.stop(handle).await
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        if let Some(server) = server_for_handle(self, handle) {
            return server.send(handle, message).await;
        }
        if let Some(ref tmux_window) = self.tmux_window
            && tmux_window.send(handle, message).await.is_ok()
        {
            return Ok(());
        }
        self.process.send(handle, message).await
    }

    async fn resume_handle(
        &self,
        slug: &str,
        provider: Option<&str>,
        current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        // Tmux-attached jobs rehydrate from tmux window state. Process
        // executor has no out-of-band handle (headless jobs can't be
        // re-attached — the child process is gone with the daemon).
        // Server-mode sessions also can't be re-attached in this slice;
        // session-state persistence is future work.
        if let Some(ref tmux_window) = self.tmux_window
            && let Some(h) = tmux_window.resume_handle(slug, provider, current_step).await
        {
            return Some(h);
        }
        None
    }
}

/// Walk `PATH` once looking for the `codex` binary. Avoids a spawn
/// attempt (and its error log) when the user hasn't installed codex.
/// Cheap enough to re-check per dispatch — the success case caches
/// the bound session so this only runs until the first codex job.
fn codex_binary_on_path() -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        let candidate = dir.join("codex");
        candidate.is_file()
    })
}

/// Build a transient `ServerExecutor` for handle dispatch when the
/// handle's inner state matches server state. Returns `None` if the
/// handle isn't a server handle OR no session is registered for the
/// handle's provider any more (hot-swapped out).
///
/// We reconstruct the executor per-call rather than keeping N
/// instances around because each `ServerExecutor` holds one session
/// `Arc`, and we may have multiple bound providers.
fn server_for_handle(
    routing: &RoutingExecutor,
    handle: &ExecutorHandle,
) -> Option<ServerExecutor> {
    // Use the session stored on the handle itself — it carries its
    // session Arc. Borrow via the same type the executor uses.
    let state = handle.inner.downcast_ref::<super::server::ServerState>()?;
    Some(ServerExecutor::new(
        routing.kernel.clone(),
        state.session_arc(),
    ))
}
