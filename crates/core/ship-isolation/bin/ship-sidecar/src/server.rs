use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::{Json, Router, routing};
use base64::Engine;
use futures_util::stream::Stream;
use ship_isolation::sidecar::protocol::{
    ExecRequest, ExecResponse, HealthResponse, SessionStartRequest, SessionStartResponse,
    ShutdownRequest, SidecarEvent, SignalRequest, StdinRequest,
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::session::{self, SharedSession};

pub struct AppState {
    pub session: SharedSession,
    pub shutdown: Arc<Notify>,
    pub version: &'static str,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/health", routing::get(health))
        .route("/v1/session/start", routing::post(session_start))
        .route("/v1/exec", routing::post(exec))
        .route("/v1/stdin", routing::post(stdin))
        .route("/v1/signal", routing::post(signal))
        .route("/v1/events", routing::get(events_sse))
        .route("/v1/shutdown", routing::post(shutdown))
        .with_state(state)
}

async fn health(State(s): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: s.version.into(),
    })
}

async fn session_start(
    State(s): State<Arc<AppState>>,
    Json(req): Json<SessionStartRequest>,
) -> Result<Json<SessionStartResponse>, (StatusCode, String)> {
    let mut sess = s.session.lock().await;
    sess.label = req.label;
    Ok(Json(SessionStartResponse {
        session_id: "ses_default".into(),
    }))
}

async fn exec(
    State(s): State<Arc<AppState>>,
    Json(req): Json<ExecRequest>,
) -> Result<Json<ExecResponse>, (StatusCode, String)> {
    let (exec_id, pid) = session::spawn_exec(&s.session, &req.bin, req.args, req.env, &req.cwd)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(ExecResponse { exec_id, pid }))
}

async fn stdin(
    State(s): State<Arc<AppState>>,
    Json(req): Json<StdinRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.data_b64)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let exec = {
        let sess = s.session.lock().await;
        sess.execs.get(&req.exec_id).cloned()
    };
    let exec = exec.ok_or((StatusCode::NOT_FOUND, "no such exec".into()))?;
    exec.write_stdin(&bytes)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn signal(
    State(s): State<Arc<AppState>>,
    Json(req): Json<SignalRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let exec = {
        let sess = s.session.lock().await;
        sess.execs.get(&req.exec_id).cloned()
    };
    let exec = exec.ok_or((StatusCode::NOT_FOUND, "no such exec".into()))?;
    exec.kill_with(req.signal)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn events_sse(
    State(s): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = {
        let sess = s.session.lock().await;
        sess.events.subscribe()
    };
    let stream = BroadcastStream::new(rx)
        .filter_map(|r| r.ok())
        .map(|ev: SidecarEvent| {
            let payload = serde_json::to_string(&ev).unwrap_or_default();
            Ok(Event::default().data(payload))
        });
    Sse::new(stream)
}

async fn shutdown(
    State(s): State<Arc<AppState>>,
    Json(_): Json<ShutdownRequest>,
) -> StatusCode {
    let _ = s
        .session
        .lock()
        .await
        .events
        .send(SidecarEvent::SidecarShutdown {
            reason: "client requested".into(),
        });
    s.shutdown.notify_one();
    StatusCode::NO_CONTENT
}
