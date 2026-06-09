//! RoutingExecutor — per-job selection between Server, TmuxWindow, and
//! Process executors.
//!
//! The orchestrator receives a single `Arc<dyn SessionExecutor>`, but we
//! want the routing decision (server-mode session vs. interactive tmux
//! vs. headless child process) made **per job** based on (a) the
//! harness's declared capabilities and (b) the `headless` flag that
//! flows in from `ship run --headless` / the API.
//!
//! Routing precedence (highest first):
//!
//! 1. **Server mode** — when `adapter.caps().server_mode == true` AND
//!    the session registry has a bound session for that harness AND
//!    the job isn't `headless`-forcing a process. Currently the
//!    registry is empty for every harness (scaffolding slice); real
//!    sessions bind in `harness-codex-02` / `harness-opencode-02`.
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
use ship_substrate::events::KernelRouter;
use crate::dispatch_ports::{ExecutorHandle, InvocationMode, SessionContext, SessionExecutor};
use ship_harnesses::harness_adapter::adapter_for_or_default;
use ship_substrate::services::server_session::ServerSession;
use tokio::sync::Mutex;

use super::{ProcessExecutor, ServerExecutor, TmuxWindowExecutor};

/// In-memory registry mapping harness id -> bound `ServerSession`.
///
/// Scaffolding: empty in this slice. `harness-codex-02` /
/// `harness-opencode-02` will populate it at daemon startup once
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

    /// Look up the session bound for a harness id. Returns `None` when
    /// no session is bound — routing falls through to Process/TmuxWindow.
    pub(crate) fn get(&self, harness: &str) -> Option<Arc<dyn ServerSession>> {
        self.inner.lock().ok()?.get(harness).cloned()
    }

    /// Bind a session for a harness id. Replaces any prior binding.
    /// Used by daemon startup and by tests to install a stub session.
    #[allow(dead_code)] // Used by tests + future harness-codex-02/harness-opencode-02.
    pub(crate) fn bind(&self, harness: impl Into<String>, session: Arc<dyn ServerSession>) {
        if let Ok(mut guard) = self.inner.lock() {
            guard.insert(harness.into(), session);
        }
    }
}

/// Routes each `spawn` to Server, TmuxWindow, or Process per the rules
/// in the module docs.
pub struct RoutingExecutor {
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

    /// Returns a `ServerExecutor` if the harness has `server_mode` AND
    /// a session is bound. Otherwise `None` — caller falls through to
    /// the existing Process/TmuxWindow selection.
    fn try_server_executor(&self, ctx: &SessionContext) -> Option<ServerExecutor> {
        let adapter = adapter_for_or_default(ctx.launch.harness.as_deref()).ok()?;
        if !adapter.caps().server_mode {
            return None;
        }
        let session = self.sessions.get(adapter.id())?;
        Some(ServerExecutor::new(self.kernel.clone(), session))
    }

    /// Lazy-spawn a server-mode session on first dispatch for any harness
    /// that declares one, caching the binding in the session registry. The
    /// adapter owns *how* to spawn (`spawn_server_session`); routing stays
    /// generic over the registry instead of branching per harness id. On
    /// missing binary or spawn failure we leave the registry untouched, so
    /// the caller falls through to Process/TmuxWindow — preserving the
    /// one-shot path as graceful degradation.
    async fn ensure_server_session(&self, ctx: &SessionContext) {
        let Some(harness) = ctx.launch.harness.as_deref() else {
            return;
        };
        let Ok(adapter) = adapter_for_or_default(Some(harness)) else {
            return;
        };
        if !adapter.caps().server_mode || self.sessions.get(adapter.id()).is_some() {
            return;
        }
        if !binary_on_path(adapter.binary()) {
            tracing::debug!(harness = adapter.id(), binary = adapter.binary(),
                "routing: binary not on PATH; skipping server-session lazy-spawn");
            return;
        }
        if let Some(session) = adapter.spawn_server_session().await {
            tracing::info!(harness = adapter.id(), "routing: lazy-spawned server session");
            self.sessions.bind(adapter.id(), session);
        }
    }

    /// Test-only inspection: which executor would `spawn` dispatch to,
    /// given this context? Avoids actually spawning a harness process
    /// in routing tests.
    #[cfg(test)]
    pub(crate) fn route_kind(&self, ctx: &SessionContext) -> RouteKind {
        if self.try_server_executor(ctx).is_some() {
            return RouteKind::Server;
        }
        if ctx.invocation == InvocationMode::Pty && self.tmux_window.is_some() {
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
impl SessionExecutor for RoutingExecutor {
    async fn spawn(&self, ctx: &SessionContext) -> Result<ExecutorHandle> {
        // Register the pipeline step as an actor so it surfaces in the
        // unified dashboard alongside chat agents and shell sessions.
        //
        // The persist runs on a bare std::thread (no ambient runtime — so the
        // durable write's block_on and the deliver's blocking_lock are both
        // safe), and we AWAIT it via a oneshot before spawning the child. The
        // old code detached the thread and discarded its result, so an
        // ultra-short Print session could spawn → run → exit before the row
        // committed (dropping it entirely), and a swallowed error left no
        // trace — the root of the transport-canary flake. Errors are logged.
        let actor_id = ctx.slug.clone();
        let workspace_id = ctx.branch.clone();
        let agent_id = ctx.agent.clone();
        let harness = ctx.launch.harness.clone();
        let transport = match ctx.invocation {
            InvocationMode::Print => "headless",
            InvocationMode::Pty => "pty",
            InvocationMode::Stream => "stream",
        }
        .to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let res = ship_substrate::db::actor_events::persist_actor_created_simple(
                &actor_id,
                "pipeline-step",
                Some(&workspace_id),
                Some(&agent_id),
                harness.as_deref(),
                Some(&transport),
                None,
            );
            let _ = tx.send(res);
        });
        match rx.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::warn!(slug = ctx.slug, %e,
                "routing: actor.created persist failed (dashboard row will be missing)"),
            Err(_) => tracing::warn!(slug = ctx.slug,
                "routing: actor.created persist thread panicked before reporting"),
        }

        // Server-mode harnesses route to a persistent session when the
        // registry has a binding. The first dispatch per harness lazy-
        // spawns the child; subsequent dispatches reuse the cached Arc.
        self.ensure_server_session(ctx).await;

        // Server-mode dispatch reuses an already-bound session — its
        // binary was checked at session-spawn time (ensure_*_session
        // above), so skip the pre-flight here and route directly.
        if let Some(server) = self.try_server_executor(ctx) {
            tracing::info!(slug = ctx.slug,
                harness = ?ctx.launch.harness,
                "routing: dispatching to ServerExecutor");
            return server.spawn(ctx).await;
        }

        // From here we're about to exec a fresh harness binary
        // (keepalive ProcessExecutor, TmuxWindowExecutor, or one-shot
        // ProcessExecutor). Pre-flight the binary so the user gets a
        // clear "install it first" message instead of an opaque
        // exit-127 / "No such file" from a few layers down.
        // The executable name comes from the adapter (`binary()`), not the
        // harness id — they diverge for cursor (id "cursor", bin
        // "cursor-agent").
        if let Some(harness) = ctx.launch.harness.as_deref()
            && let Ok(adapter) = adapter_for_or_default(Some(harness))
        {
            let bin = adapter.binary();
            if !binary_on_path(bin) {
                anyhow::bail!(
                    "dispatch: {bin} binary not on PATH. \
                     Install the {bin} CLI before dispatching agents that use it. \
                     (Check `which {bin}`; ensure the install dir is in PATH.)"
                );
            }
        }

        // Keepalive sessions need PTY for the harness to recognize a
        // TTY (claude/codex/etc downgrade output when there isn't one)
        // AND need the WS attach surface (only ProcessExecutor exposes
        // the substrate ActorHandle the attach endpoint upgrades to).
        // TmuxWindow can't serve attach today; Server mode is its own
        // beast. Force ProcessExecutor here regardless of `headless`.
        if ctx.keepalive {
            tracing::info!(slug = ctx.slug,
                harness = ?ctx.launch.harness,
                "routing: keepalive — forcing ProcessExecutor with build_interactive");
            return self.process.spawn(ctx).await;
        }
        // PTY invocation routes to TmuxWindow (interactive attach) when
        // available. Print routes to ProcessExecutor (one-shot `-p`).
        // Stream was handled above by the ServerExecutor branch.
        if ctx.invocation == InvocationMode::Pty
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
        harness: Option<&str>,
        current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        // Tmux-attached jobs rehydrate from tmux window state. Process
        // executor has no out-of-band handle (headless jobs can't be
        // re-attached — the child process is gone with the daemon).
        // Server-mode sessions also can't be re-attached in this slice;
        // session-state persistence is future work.
        if let Some(ref tmux_window) = self.tmux_window
            && let Some(h) = tmux_window.resume_handle(slug, harness, current_step).await
        {
            return Some(h);
        }
        None
    }
}

/// Walk `PATH` once looking for a named binary. Avoids a spawn attempt
/// (and its error log) when the user hasn't installed the harness.
/// Cheap enough to re-check per dispatch — the success case caches the
/// bound session so this only runs until the first matching job.
fn binary_on_path(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file()
    })
}

/// Build a transient `ServerExecutor` for handle dispatch when the
/// handle's inner state matches server state. Returns `None` if the
/// handle isn't a server handle OR no session is registered for the
/// handle's harness any more (hot-swapped out).
///
/// We reconstruct the executor per-call rather than keeping N
/// instances around because each `ServerExecutor` holds one session
/// `Arc`, and we may have multiple bound harnesses.
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
