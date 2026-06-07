//! Lifecycle CRUD handlers for the PTY-session HTTP surface:
//! spawn / list / get / stop / resize, plus the `installed_harnesses`
//! probe that the spawn UI uses to gate harness options.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::auth::{AuthContext};
use crate::routes::rest_api::ApiState;

use super::super::lifecycle;
use super::super::types::{SessionKind, SessionMeta};
use super::{err, work_dir_for};

#[derive(Deserialize)]
pub struct SpawnBody {
    pub workspace_id: String,
    /// `"agent" | "shell"`. Pipelines register internally.
    pub kind: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub harness: Option<String>,
}

#[derive(Serialize)]
pub struct SpawnResponse {
    pub session_id: String,
    pub attached: bool,
}

#[derive(Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionMetaWire>,
}

/// Wire-format session meta. Mirrors `SessionMeta` but decoupled from
/// the internal `types::private` representation: the wire shape is
/// declared here so the private type can change without breaking the
/// HTTP contract.
#[derive(Serialize)]
pub struct SessionMetaWire {
    pub id: String,
    pub kind: String,
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub status: String,
    pub log_path: String,
    pub spawned_at: String,
    pub last_activity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

impl From<SessionMeta> for SessionMetaWire {
    fn from(m: SessionMeta) -> Self {
        Self {
            id: m.id,
            kind: match m.kind {
                SessionKind::Agent => "agent",
                SessionKind::Pipeline => "pipeline",
                SessionKind::Shell => "shell",
            }
            .to_string(),
            workspace_id: m.workspace_id,
            agent_id: m.agent_id,
            harness: m.harness,
            pid: m.pid,
            status: format!("{:?}", m.status).to_lowercase(),
            log_path: m.log_path,
            spawned_at: m.spawned_at.to_rfc3339(),
            last_activity: m.last_activity.to_rfc3339(),
            exit_code: m.exit_code,
        }
    }
}

/// Harness detection: which of the known harnesses have their CLI
/// binaries on PATH. Powers the command-bar's spawn card.
#[derive(Serialize)]
pub struct InstalledProvidersResponse {
    pub harnesses: Vec<String>,
}

const KNOWN_HARNESSES: &[(&str, &str)] = &[
    ("claude", "claude"),
    ("codex", "codex"),
    ("cursor", "cursor-agent"),
    ("opencode", "opencode"),
];

fn binary_exists(bin: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|p| {
        let candidate = p.join(bin);
        if !candidate.is_file() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(&candidate)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            true
        }
    })
}

pub async fn installed_harnesses(
    Extension(_auth): Extension<AuthContext>,
) -> axum::response::Response {
    let harnesses = KNOWN_HARNESSES
        .iter()
        .filter(|(_, bin)| binary_exists(bin))
        .map(|(id, _)| id.to_string())
        .collect();
    (
        StatusCode::OK,
        Json(InstalledProvidersResponse { harnesses }),
    )
        .into_response()
}

pub async fn spawn_session(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Json(req): Json<SpawnBody>,
) -> axum::response::Response {
    let registry = state.pty_sessions();
    let log_dir = state.ship_dir().join("logs").join("sessions");
    let work_dir = work_dir_for(&state, &req.workspace_id);

    match req.kind.as_str() {
        "agent" => {
            let agent_id = match req.agent_id.as_deref() {
                Some(a) if !a.is_empty() => a,
                _ => return err(StatusCode::BAD_REQUEST, "agent_id required for agent kind"),
            };
            let harness = match req.harness.as_deref() {
                Some(p) if !p.is_empty() => p,
                _ => return err(StatusCode::BAD_REQUEST, "harness required for agent kind"),
            };

            // Persistent-session harnesses (all five CLI agents today)
            // go through the supervisor primitive + ActorRegistry. The
            // push_mechanism (channel / stdin / respawn) is decided at
            // send-time; the spawn shape is identical regardless.
            if let Ok(adapter) = ship_harnesses::harness_adapter::adapter_for(harness)
                && adapter.caps().persistent_session {
                    let append = lifecycle::load_agent_rules(&work_dir, agent_id);
                    return match super::super::agent_actor::spawn_or_attach(
                        state.actor_sessions(),
                        &req.workspace_id,
                        agent_id,
                        harness,
                        work_dir,
                        log_dir,
                        append,
                    ) {
                        Ok(outcome) => {
                            let resp = SpawnResponse {
                                session_id: outcome.entry.handle.actor_id.clone(),
                                attached: outcome.attached,
                            };
                            (StatusCode::OK, Json(resp)).into_response()
                        }
                        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                    };
                }

            match lifecycle::spawn_agent(
                registry,
                &req.workspace_id,
                agent_id,
                harness,
                work_dir,
                log_dir,
            ) {
                Ok(outcome) => {
                    if !outcome.attached {
                        lifecycle::emit_actor_created_for(
                            &outcome.session,
                            &lifecycle::ActorSpawnExtras {
                                transport: Some("pty".to_string()),
                                persistence: Some("long-lived".to_string()),
                                ..Default::default()
                            },
                        );
                    }
                    let resp = SpawnResponse {
                        session_id: outcome.session.meta().id,
                        attached: outcome.attached,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        }
        "shell" => {
            // Shell sessions go through the supervisor primitive +
            // ActorRegistry. Kernel registration, PTY hold, log file,
            // and `actor.created` emission all happen inside
            // `ship_substrate::supervisor::spawn_actor` — no second registry
            // write, no second event emit.
            match super::super::shell_actor::spawn_shell(
                state.actor_sessions(),
                &req.workspace_id,
                work_dir,
                log_dir,
            ) {
                Ok(entry) => {
                    let resp = SpawnResponse {
                        session_id: entry.handle.actor_id.clone(),
                        attached: false,
                    };
                    (StatusCode::OK, Json(resp)).into_response()
                }
                Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        }
        other => err(StatusCode::BAD_REQUEST, format!("unknown kind '{other}'")),
    }
}

/// List PTY-backed sessions (terminal-attached agent/shell sessions
/// from the runtime registry). Distinct from workspace sessions
/// (`runtime_api::list_workspace_sessions` on `/api/v1/sessions`) and
/// auth-token sessions (`auth::api::list_auth_sessions` on
/// `/api/v1/auth/sessions`).
pub async fn list_pty_sessions(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
) -> axum::response::Response {
    // Cheap housekeeping — flip thinking → idle as needed before
    // handing the snapshot to the client.
    lifecycle::sweep_idle(state.pty_sessions());
    let mut sessions: Vec<SessionMetaWire> = state
        .pty_sessions()
        .list()
        .into_iter()
        .map(SessionMetaWire::from)
        .collect();
    sessions.extend(
        state
            .actor_sessions()
            .list()
            .into_iter()
            .map(SessionMetaWire::from),
    );
    (StatusCode::OK, Json(SessionListResponse { sessions })).into_response()
}

pub async fn get_session(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> axum::response::Response {
    if let Some(entry) = state.actor_sessions().get(&id) {
        let wire: SessionMetaWire = entry.meta_snapshot().into();
        return (StatusCode::OK, Json(wire)).into_response();
    }
    match state.pty_sessions().get(&id) {
        Some(session) => {
            let wire: SessionMetaWire = session.meta().into();
            (StatusCode::OK, Json(wire)).into_response()
        }
        None => err(StatusCode::NOT_FOUND, format!("session '{id}' not found")),
    }
}

pub async fn stop_session(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
) -> axum::response::Response {
    if super::super::shell_actor::stop(state.actor_sessions(), &id)
        || lifecycle::stop(state.pty_sessions(), &id)
    {
        (StatusCode::NO_CONTENT, ()).into_response()
    } else {
        err(StatusCode::NOT_FOUND, format!("session '{id}' not found"))
    }
}

#[derive(Deserialize)]
pub struct ResizeBody {
    pub cols: u16,
    pub rows: u16,
}

pub async fn session_resize(
    State(state): State<ApiState>,
    Extension(_auth): Extension<AuthContext>,
    Path(id): Path<String>,
    Json(body): Json<ResizeBody>,
) -> axum::response::Response {
    if let Some(entry) = state.actor_sessions().get(&id) {
        if let Err(e) = entry.handle.resize(body.cols, body.rows) {
            return err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
        return (StatusCode::NO_CONTENT, ()).into_response();
    }
    // Per-turn PtySession path (codex/opencode etc. — harnesses without
    // channel-capability). The portable-pty master sits inside the Inner
    // struct and is not currently exposed for resize. These are one-shot
    // turn processes, so the gap is benign in practice.
    if state.pty_sessions().get(&id).is_some() {
        return err(
            StatusCode::NOT_IMPLEMENTED,
            "per-turn session: resize not supported",
        );
    }
    err(StatusCode::NOT_FOUND, format!("session '{id}' not found"))
}
