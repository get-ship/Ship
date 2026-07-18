//! terminal.* mount — L2 Connect adapter over the unified live-session
//! registry.
//!
//! Translates the `ship.ops.v1.TerminalService` Connect surface onto
//! `ship_agents::services::sessions::Registry` — the one registry
//! that owns where a live session lives.
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
use ship_agents::services::sessions::registry::SessionRecord;
use ship_agents::services::sessions::types::SessionStatus;

use crate::ship_runtime::ApiState;

/// Daemon-side `TerminalService`. Holds `ApiState` so all eight methods
/// can reach the session registry + the runtime config / paths the spawn
/// path needs.
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

// ── Trait impl ───────────────────────────────────────────────────────────────

impl TerminalService for TerminalWire {
    // ── Spawn ────────────────────────────────────────────────────────────
    // Phase 3: shell kind only. The new `SpawnTerminalRequest` carries
    // `kind` + `work_dir` — no `agent_id` / `harness`
    // fields. The persistent-harness path (Claude, with --resume) needs
    // those to call `agent_session_spawner::spawn_or_attach`; the wire refuses
    // `agent` here rather than fabricate. Phase 4 extends the proto
    // (or a separate `SpawnAgent` op) to carry them.
    #[allow(refining_impl_trait)]
    async fn spawn(
        &self,
        _ctx: RequestContext,
        request: OwnedSpawnTerminalRequestView,
    ) -> ServiceResult<TerminalSession> {
        let kind = request.kind.to_string();
        let work_dir = resolve_work_dir(&self.state, &request.work_dir);

        match kind.as_str() {
            "shell" => match ship_agents::services::sessions::terminal_session_spawner::spawn_shell(
                self.state.live_sessions(),
                work_dir,
                self.state.ship_dir().join("logs").join("sessions"),
            ) {
                Ok(entry) => Ok(Response::new(meta_to_wire(&entry.meta_snapshot()))),
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
        if ship_agents::services::sessions::terminal_session_spawner::stop(self.state.live_sessions(), &id) {
            return Ok(Response::new(StopTerminalResponse::default()));
        }
        Err(ConnectError::not_found(format!("session '{id}' not found")))
    }

    // ── List ─────────────────────────────────────────────────────────────
    #[allow(refining_impl_trait)]
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: OwnedListTerminalsRequestView,
    ) -> ServiceResult<ListTerminalsResponse> {
        ship_agents::services::sessions::lifecycle::sweep_idle(self.state.live_sessions());
        let mut sessions: Vec<TerminalSession> = self
            .state
            .live_sessions()
            .list_all()
            .into_iter()
            .map(|m| meta_to_wire(&m))
            .collect();
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
        let entry = self
            .state
            .live_sessions()
            .get(&id)
            .ok_or_else(|| ConnectError::not_found(format!("session '{id}' not found")))?;
        Ok(Response::new(meta_to_wire(&entry.meta_snapshot())))
    }

    // ── SendInput ────────────────────────────────────────────────────────
    // Only sessions that own a live transport accept raw stdin. Every
    // entry on the actor registry owns a live `ProcessHandle`.
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
            .live_sessions()
            .get(&id)
            .ok_or_else(|| ConnectError::not_found(format!("session '{id}' not found")))?;
        entry
            .write_input(&data)
            .map_err(|e| ConnectError::internal(format!("write_input: {e}")))?;
        Ok(Response::new(SendInputResponse::default()))
    }

    // ── Resize ───────────────────────────────────────────────────────────
    #[allow(refining_impl_trait)]
    async fn resize(
        &self,
        _ctx: RequestContext,
        request: OwnedResizeTerminalRequestView,
    ) -> ServiceResult<ResizeTerminalResponse> {
        let id = request.session_id.to_string();
        let cols = request.cols;
        let rows = request.rows;
        let entry = self
            .state
            .live_sessions()
            .get(&id)
            .ok_or_else(|| ConnectError::not_found(format!("session '{id}' not found")))?;
        entry
            .handle
            .resize(cols as u16, rows as u16)
            .map_err(|e| ConnectError::internal(format!("resize: {e}")))?;
        Ok(Response::new(ResizeTerminalResponse::default()))
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
        let entry = self
            .state
            .live_sessions()
            .get(&id)
            .ok_or_else(|| ConnectError::not_found(format!("session '{id}' not found")))?;
        let log_path = entry.log_path();
        let mut rx = entry.subscribe();
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
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    // ── SessionEvents (server-streaming) ────────────────────────────────
    // The actor registry wires the agent-driver broadcast. Shell entries
    // (no harness) return a closed stream.
    #[allow(refining_impl_trait)]
    async fn session_events(
        &self,
        _ctx: RequestContext,
        request: OwnedSessionEventsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let id = request.session_id.to_string();
        let entry = self
            .state
            .live_sessions()
            .get(&id)
            .ok_or_else(|| ConnectError::not_found(format!("session '{id}' not found")))?;
        let mut rx = entry.subscribe_events();
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
// Convert `SessionRecord` (the L1 process-lifecycle source of truth) into
// the `TerminalSession` Connect message. The proto's `kind` field is a
// wire-boundary concern only — the dock genuinely needs one heterogeneous
// list of everything running, tagged so the client can render agent vs.
// terminal tabs differently. Internally there is no kind enum; the tag is
// derived structurally (agent_id present or not) right here at the
// serialization edge.
//
// The 18-field union message has fields the L1 layer doesn't track yet
// (model, outcome, reason, input, parent, turn_count, ended_at) — they stay
// empty until the canonical session projection (#20) is wired through.
// That migration is Phase 4 territory; the wire shape is set here so
// callers can start depending on the field set without re-shaping the proto.

fn meta_to_wire(m: &SessionRecord) -> TerminalSession {
    TerminalSession {
        id: m.id.clone(),
        kind: kind_str(m),
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

fn kind_str(m: &SessionRecord) -> String {
    if m.agent_id.is_some() { "agent" } else { "shell" }.to_string()
}

fn status_str(s: SessionStatus) -> String {
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
