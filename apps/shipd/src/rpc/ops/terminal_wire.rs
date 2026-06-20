//! terminal.* mount — L2 Connect adapter over the PTY session domain.
//!
//! Translates the `ship.ops.v1.TerminalService` Connect surface onto the
//! two session registries that shipd holds:
//!
//! - `ship_terminal::SessionRegistry` (the L1 PTY session registry carved
//!   out of `ship-agents` in Phase 2) — chat / pipelines / per-turn agent
//!   sessions whose child is owned by the substrate supervisor.
//! - `ship_agents::services::sessions::ActorRegistry` — long-lived agent
//!   sessions that own a live `ActorHandle` (the destination of the
//!   supervisor-primitive migration; persistent-session harnesses).
//!
//! Either registry can hold the session id a request names; the wire
//! looks up the actor registry first (it owns the persistent-session
//! path the dock terminal uses today), then falls back to the PTY
//! registry. Phase 3 retired the legacy REST surface; this wire is the
//! permanent terminal Connect surface.
//!
//! Capability / blast-radius policy lives in the proto (read by the gate
//! from the descriptor at startup) — the wire does not re-enforce it.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_ops_rpc::{
    BytesChunk, JsonResponse, ListTerminalsResponse, OwnedGetTerminalRequestView,
    OwnedListTerminalsRequestView, OwnedResizeTerminalRequestView, OwnedSendInputRequestView,
    OwnedSessionEventsRequestView, OwnedSessionStreamRequestView,
    OwnedSpawnTerminalRequestView, OwnedStopTerminalRequestView, ResizeTerminalResponse,
    SendInputResponse, StopTerminalResponse, TerminalSession, TerminalService, TerminalServiceExt,
};
use ship_terminal::types::{SessionKind as PtyKind, SessionMeta, SessionStatus as PtyStatus};

use crate::ship_runtime::ApiState;

/// Daemon-side `TerminalService`. Holds `ApiState` so all eight methods
/// can reach both session registries + the runtime config / paths the
/// spawn path needs.
pub struct TerminalWire {
    state: ApiState,
}

impl TerminalWire {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

pub(super) fn mount(router: connectrpc::Router, state: &ApiState) -> connectrpc::Router {
    let wire = std::sync::Arc::new(TerminalWire::new(state.clone()));
    TerminalServiceExt::register(wire, router)
}

// ── Dual-registry helpers ────────────────────────────────────────────────────
//
// A `session_id` can live in either registry. Look up the actor
// registry first (the destination of the supervisor-primitive migration;
// long-lived agents + shells) and fall back to the PTY registry (chat
// sessions, per-turn agent turns, pipeline step children).

enum Located {
    Actor(ship_agents::services::sessions::ActorEntry),
    Pty(ship_terminal::types::PtySession),
}

impl TerminalWire {
    fn locate(&self, id: &str) -> Result<Located, ConnectError> {
        if let Some(entry) = self.state.actor_sessions().get(id) {
            return Ok(Located::Actor(entry));
        }
        if let Some(session) = self.state.pty_sessions().get(id) {
            return Ok(Located::Pty(session));
        }
        Err(ConnectError::not_found(format!(
            "session '{id}' not found"
        )))
    }
}

// ── Trait impl ───────────────────────────────────────────────────────────────

impl TerminalService for TerminalWire {
    // ── Spawn ────────────────────────────────────────────────────────────
    // Phase 3: shell kind only. The new `SpawnTerminalRequest` carries
    // `workspace_id` + `kind` + `work_dir` — no `agent_id` / `harness`
    // fields. The persistent-harness path (Claude, with --resume) needs
    // those to call `agent_actor::spawn_or_attach`; the wire refuses
    // `agent` here rather than fabricate. Phase 4 extends the proto
    // (or a separate `SpawnAgent` op) to carry them.
    #[allow(refining_impl_trait)]
    async fn spawn(
        &self,
        _ctx: RequestContext,
        request: OwnedSpawnTerminalRequestView,
    ) -> ServiceResult<TerminalSession> {
        let workspace_id = request.workspace_id.to_string();
        let kind = request.kind.to_string();
        let work_dir = resolve_work_dir(&self.state, &request.work_dir);

        match kind.as_str() {
            "shell" => match ship_terminal::shell_actor::spawn_shell(
                self.state.pty_sessions(),
                &workspace_id,
                work_dir,
                self.state.ship_dir().join("logs").join("sessions"),
            ) {
                Ok(session) => Ok(Response::new(meta_to_wire(&session.meta()))),
                Err(e) => Err(ConnectError::internal(format!("spawn_shell: {e}"))),
            },
            "agent" => Err(ConnectError::invalid_argument(
                "agent spawn requires agent_id + harness — not yet exposed on the wire \
                 (see Phase 4 / SpawnAgent op); only 'shell' is supported today",
            )),
            other => Err(ConnectError::invalid_argument(format!(
                "unknown kind '{other}' (only 'shell' is supported)"
            ))),
        }
    }

    // ── Stop ─────────────────────────────────────────────────────────────
    #[allow(refining_impl_trait)]
    async fn stop(
        &self,
        _ctx: RequestContext,
        request: OwnedStopTerminalRequestView,
    ) -> ServiceResult<StopTerminalResponse> {
        let id = request.session_id.to_string();
        // Actor registry first — the destination of the supervisor-primitive
        // migration. `entry.handle.stop()` returns `Result<(), anyhow::Error>`.
        if let Some(entry) = self.state.actor_sessions().get(&id) {
            entry
                .handle
                .stop()
                .map_err(|e| ConnectError::internal(format!("stop: {e}")))?;
            return Ok(Response::new(StopTerminalResponse::default()));
        }
        // PTY registry — shells + per-turn agent turns. `shell_actor::stop`
        // is a registry-remove (the supervisor's child has already exited
        // for the shell case; for the per-turn path the parent session is
        // removed and the next-turn handler spawns fresh).
        if ship_terminal::shell_actor::stop(self.state.pty_sessions(), &id) {
            return Ok(Response::new(StopTerminalResponse::default()));
        }
        Err(ConnectError::not_found(format!("session '{id}' not found")))
    }

    // ── List ─────────────────────────────────────────────────────────────
    // Sweep thinking → idle before serving, then union the two registries
    // and convert to the wire message. `include_ended` is accepted but
    // not yet acted on (the current registries' `list()` returns every
    // entry, alive or ended; a `status = Ended` filter lands when the
    // supervisor primitive flushes Ended rows from the actor registry
    // after a TTL).
    #[allow(refining_impl_trait)]
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: OwnedListTerminalsRequestView,
    ) -> ServiceResult<ListTerminalsResponse> {
        ship_agents::services::sessions::lifecycle::sweep_idle(self.state.pty_sessions());
        let mut sessions: Vec<TerminalSession> = self
            .state
            .pty_sessions()
            .list()
            .into_iter()
            .map(|m| meta_to_wire(&m))
            .collect();
        sessions.extend(
            self.state
                .actor_sessions()
                .list()
                .into_iter()
                .map(|m| meta_to_wire(&m)),
        );
        // Most-recent-activity first, matching the legacy `SessionListResponse`.
        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(Response::new(ListTerminalsResponse {
            sessions,
            ..Default::default()
        }))
    }

    // ── Get ──────────────────────────────────────────────────────────────
    #[allow(refining_impl_trait)]
    async fn get(
        &self,
        _ctx: RequestContext,
        request: OwnedGetTerminalRequestView,
    ) -> ServiceResult<TerminalSession> {
        let id = request.session_id.to_string();
        match self.locate(&id)? {
            Located::Actor(entry) => Ok(Response::new(meta_to_wire(&entry.meta_snapshot()))),
            Located::Pty(session) => Ok(Response::new(meta_to_wire(&session.meta()))),
        }
    }

    // ── SendInput ────────────────────────────────────────────────────────
    // Only sessions that own a live transport accept raw stdin — actor
    // entries (shells + persistent-harness agents). The PTY-registry
    // per-turn agent path has no live writer; clients drive those
    // through `send_agent_message`, not this RPC.
    #[allow(refining_impl_trait)]
    async fn send_input(
        &self,
        _ctx: RequestContext,
        request: OwnedSendInputRequestView,
    ) -> ServiceResult<SendInputResponse> {
        let id = request.session_id.to_string();
        let data = request.data.to_vec();
        let entry = self
            .state
            .actor_sessions()
            .get(&id)
            .ok_or_else(|| ConnectError::not_found(format!("session '{id}' not found")))?;
        entry
            .write_input(&data)
            .map_err(|e| ConnectError::internal(format!("write_input: {e}")))?;
        Ok(Response::new(SendInputResponse::default()))
    }

    // ── Resize ───────────────────────────────────────────────────────────
    // Persistent-harness agents on the actor registry resize via
    // `ActorHandle::resize`. The PTY-registry per-turn path exposes no
    // master handle (the supervisor's portable-pty lives inside `Inner`),
    // so per-turn sessions answer NOT_IMPLEMENTED — one-shot turn
    // processes don't need it.
    #[allow(refining_impl_trait)]
    async fn resize(
        &self,
        _ctx: RequestContext,
        request: OwnedResizeTerminalRequestView,
    ) -> ServiceResult<ResizeTerminalResponse> {
        let id = request.session_id.to_string();
        let cols = request.cols;
        let rows = request.rows;
        if let Some(entry) = self.state.actor_sessions().get(&id) {
            entry.handle
                .resize(cols as u16, rows as u16)
                .map_err(|e| ConnectError::internal(format!("resize: {e}")))?;
            return Ok(Response::new(ResizeTerminalResponse::default()));
        }
        if self.state.pty_sessions().get(&id).is_some() {
            return Err(ConnectError::unimplemented(
                "per-turn session: resize not supported",
            ));
        }
        Err(ConnectError::not_found(format!("session '{id}' not found")))
    }

    // ── SessionBytes (server-streaming) ─────────────────────────────────
    // Replay the tee'd log file, then tail the live broadcast. Byte-exact
    // — the SSE base64 dance the legacy route used is gone with the route.
    #[allow(refining_impl_trait)]
    async fn session_bytes(
        &self,
        _ctx: RequestContext,
        request: OwnedSessionStreamRequestView,
    ) -> ServiceResult<ServiceStream<BytesChunk>> {
        let id = request.session_id.to_string();
        let (log_path, mut rx) = match self.locate(&id)? {
            Located::Actor(entry) => (entry.log_path(), entry.subscribe()),
            Located::Pty(session) => (session.0.log_path.clone(), session.subscribe()),
        };
        let replay = tokio::fs::read(&log_path).await.unwrap_or_default();
        let stream = async_stream::stream! {
            if !replay.is_empty() {
                yield Ok::<BytesChunk, ConnectError>(BytesChunk {
                    data: replay,
                    ..Default::default()
                });
            }
            loop {
                match rx.recv().await {
                    Ok(bytes) => yield Ok(BytesChunk { data: bytes, ..Default::default() }),
                    // Slow consumer dropped frames — keep tailing rather than abort.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    // Sender gone — session ended.
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    // ── SessionEvents (server-streaming) ────────────────────────────────
    // The legacy `/api/v1/sessions/{id}/events` handler returned typed
    // `HarnessEvent`s as JSON. The actor registry wires the
    // harness-adapter broadcast; the PTY-registry per-turn path also
    // publishes here (see `agent_actor::run_headless_turn`). Shells
    // (no harness) return a closed stream.
    #[allow(refining_impl_trait)]
    async fn session_events(
        &self,
        _ctx: RequestContext,
        request: OwnedSessionEventsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let id = request.session_id.to_string();
        let mut rx = match self.locate(&id)? {
            Located::Actor(entry) => entry.subscribe_events(),
            Located::Pty(session) => session.subscribe_events(),
        };
        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            yield Ok::<JsonResponse, ConnectError>(JsonResponse {
                                json,
                                ..Default::default()
                            });
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }
}

// ── Wire conversion ─────────────────────────────────────────────────────────
//
// Convert `SessionMeta` (the L1 source of truth) into the
// `TerminalSession` Connect message. The 18-field union message has
// fields the L1 layer doesn't track yet (model, outcome, reason, input,
// parent, turn_count, ended_at) — they stay empty until the canonical
// session projection (#20) is wired through. That migration is Phase 4
// territory; the wire shape is set here so callers can start depending
// on the field set without re-shaping the proto.

fn meta_to_wire(m: &SessionMeta) -> TerminalSession {
    TerminalSession {
        id: m.id.clone(),
        kind: kind_str(m.kind),
        workspace_id: m.workspace_id.clone(),
        agent_id: m.agent_id.clone().unwrap_or_default(),
        harness: m.harness.clone().unwrap_or_default(),
        status: status_str(m.status),
        pid: m.pid.map(|p| p.to_string()).unwrap_or_default(),
        log_path: m.log_path.clone(),
        harness_session_id: m.harness_session_id.clone().unwrap_or_default(),
        started_at: m.spawned_at.to_rfc3339(),
        ..Default::default()
    }
}

fn kind_str(k: PtyKind) -> String {
    match k {
        PtyKind::Agent => "agent",
        PtyKind::Shell => "shell",
    }
    .to_string()
}

fn status_str(s: PtyStatus) -> String {
    // Format matches the legacy `SessionMetaWire`:
    // `format!("{:?}", m.status).to_lowercase()` — "spawning", "idle",
    // "thinking", "ended".
    format!("{s:?}").to_lowercase()
}

// ── Work-dir resolution ─────────────────────────────────────────────────────
//
// Mirrors the legacy `work_dir_for`: prefer the caller-supplied
// `work_dir`, then the daemon's attached workspace, then `current_dir()`.

fn resolve_work_dir(state: &ApiState, requested: &str) -> std::path::PathBuf {
    let trimmed = requested.trim();
    if !trimmed.is_empty() {
        return std::path::PathBuf::from(trimmed);
    }
    if let Some(ws) = state.workspace_dir() {
        return ws.to_path_buf();
    }
    std::env::current_dir().unwrap_or_else(|_| state.ship_dir().to_path_buf())
}
