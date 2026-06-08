//! Integration tests for the agent-session WebSocket attach route.
//!
//! The handler upgrades to WS for PTY-backed sessions and bridges bytes
//! both directions through the substrate `ActorHandle`. These tests spin
//! up a real `TcpListener` so axum's WS upgrade machinery runs end-to-end.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use axum::{Router, routing::get};
use futures_util::{SinkExt, StreamExt};
use ship_substrate::services::dispatch_ports::{
    ExecutorHandle, SessionContext, SessionExecutor,
};
use ship_substrate::services::process_registry::ProcessRegistry;
use ship_substrate::supervisor::{ActorKind, ActorProfile, TransportKind, spawn_actor};
use crate::dispatch::run_dispatch::DispatchHandle;
use ship_agents::runtime::pty_executor_handle_for_test;
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::auth::{AuthContext, Scope};
use crate::routes::agent_sessions_attach::attach_agent_session;
use crate::ship_runtime::ShipRuntime;

// ── Stub executor ────────────────────────────────────────────────────────────

struct NoopExecutor;

#[async_trait]
impl SessionExecutor for NoopExecutor {
    async fn spawn(&self, _ctx: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Ok(ExecutorHandle { pid: None, inner: Box::new(()) })
    }
    async fn is_alive(&self, _h: &ExecutorHandle) -> bool { true }
    async fn stop(&self, _h: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _h: &ExecutorHandle, _m: &str) -> anyhow::Result<()> { Ok(()) }
}

// ── Harness ──────────────────────────────────────────────────────────────────

/// Bind a real TCP listener, mount the attach route, and return the address
/// plus the underlying registry handle so tests can register sessions.
async fn spawn_server() -> (TempDir, std::net::SocketAddr, Arc<Mutex<ProcessRegistry>>) {
    let tmp = TempDir::new().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();

    let kernel = Arc::new(Mutex::new(
        ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
    ));
    let rt = Arc::new(ShipRuntime::new_test(ship_dir, kernel));

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
    let executor: Arc<dyn SessionExecutor> = Arc::new(NoopExecutor);
    let dh = DispatchHandle {
        registry: registry.clone(),
        executor,
        activity_txs: Arc::new(Mutex::new(Default::default())),
    };
    rt.set_dispatch_handle(dh);

    let auth = AuthContext {
        caller_kind: ship_substrate::events::CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
        grants: vec![],
        operator_cred: false,
    };

    let app = Router::new()
        .route(
            "/api/v1/agent_sessions/{slug}/attach",
            get(attach_agent_session),
        )
        .layer(axum::Extension(auth))
        .with_state(rt);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (tmp, addr, registry)
}

/// Spawn a PTY child that echoes everything written to its stdin back as
/// "got: <line>" — gives us deterministic bytes for the round-trip tests.
fn spawn_echo_pty(slug: &str, log_dir: std::path::PathBuf) -> ship_substrate::supervisor::ActorHandle {
    std::fs::create_dir_all(&log_dir).unwrap();
    let profile = ActorProfile {
        actor_id_hint: Some(slug.to_string()),
        kind: ActorKind::PipelineStep,
        transport: TransportKind::Pty,
        one_shot: true,
        bin: "sh".into(),
        // `cat` echoes the PTY input straight back, which the broadcast
        // captures. Simpler than building a custom reply loop.
        args: vec!["-c".into(), "cat".into()],
        env: Default::default(),
        work_dir: log_dir.clone(),
        workspace_id: None,
        agent_id: None,
        harness: None,
        parent_actor_id: None,
        log_dir: Some(log_dir),
        subscribe_namespaces: vec![],
    };
    spawn_actor(profile).expect("pty spawn")
}

async fn connect(
    addr: std::net::SocketAddr,
    slug: &str,
) -> Result<tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >, tokio_tungstenite::tungstenite::Error> {
    let url = format!("ws://{addr}/api/v1/agent_sessions/{slug}/attach");
    let (ws, _resp) = tokio_tungstenite::connect_async(url).await?;
    Ok(ws)
}

/// Drain WS frames for up to `timeout` until `needle` is observed in the
/// concatenated text. Returns the concatenated text on success.
async fn drain_until(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    needle: &str,
    timeout: Duration,
) -> Option<String> {
    let mut acc = String::new();
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return None;
        }
        let next = tokio::time::timeout(remaining, ws.next()).await.ok()??;
        if let Ok(WsMessage::Binary(b)) = next {
            acc.push_str(&String::from_utf8_lossy(&b));
            if acc.contains(needle) {
                return Some(acc);
            }
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// 404 for an unknown slug — the dispatch handle is wired but the
/// registry doesn't contain a session for the requested name.
#[tokio::test(flavor = "multi_thread")]
async fn attach_unknown_slug_returns_404() {
    let (_tmp, addr, _registry) = spawn_server().await;
    let url = format!("http://{addr}/api/v1/agent_sessions/does-not-exist/attach");
    let resp = reqwest::Client::new()
        .get(url)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

/// 409 when the registered session is not PTY-backed (e.g. tmux-window
/// or server-mode executors that don't carry a substrate `ActorHandle`).
#[tokio::test(flavor = "multi_thread")]
async fn attach_non_pty_session_returns_409() {
    let (_tmp, addr, registry) = spawn_server().await;
    registry.lock().await.register(
        "tmux-job",
        ExecutorHandle { pid: Some(99999), inner: Box::new(()) },
    );

    let url = format!("http://{addr}/api/v1/agent_sessions/tmux-job/attach");
    let resp = reqwest::Client::new()
        .get(url)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
}

/// Round-trip: bytes written from the client appear at the substrate
/// `ActorHandle`'s input side (PTY echo loops them back), and the broadcast
/// flows back out through the WebSocket.
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(unix), ignore)]
async fn attach_round_trip_pty_bytes() {
    let (tmp, addr, registry) = spawn_server().await;
    let log_dir = tmp.path().join("logs");
    let handle = spawn_echo_pty("echo-job", log_dir);
    registry.lock().await.register(
        "echo-job",
        pty_executor_handle_for_test("echo-job", handle.clone()),
    );

    let mut ws = connect(addr, "echo-job").await.expect("ws connect");
    ws.send(WsMessage::Binary("ping\n".as_bytes().to_vec().into()))
        .await
        .expect("send");

    let observed = drain_until(&mut ws, "ping", Duration::from_secs(5)).await;
    assert!(observed.is_some(), "client should observe echoed bytes within 5s");

    // Drop the client; the substrate handle's process should still be live.
    drop(ws);
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(handle.is_alive(), "ws close must not kill the underlying PTY");
    let _ = handle.stop();
}

/// Server-side bytes (echoed PTY output) reach both concurrent attachers.
#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(unix), ignore)]
async fn attach_concurrent_clients_both_receive_output() {
    let (tmp, addr, registry) = spawn_server().await;
    let log_dir = tmp.path().join("logs2");
    let handle = spawn_echo_pty("multi-job", log_dir);
    registry.lock().await.register(
        "multi-job",
        pty_executor_handle_for_test("multi-job", handle.clone()),
    );

    let mut a = connect(addr, "multi-job").await.unwrap();
    let mut b = connect(addr, "multi-job").await.unwrap();
    // Both subscribers must be attached before the producer fires its
    // bytes, otherwise the second client misses the broadcast.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Either client can drive input — last-writer-wins is fine.
    a.send(WsMessage::Binary("hello\n".as_bytes().to_vec().into()))
        .await
        .expect("send");

    let got_a = drain_until(&mut a, "hello", Duration::from_secs(5)).await;
    let got_b = drain_until(&mut b, "hello", Duration::from_secs(5)).await;
    assert!(got_a.is_some(), "first client must see echo");
    assert!(got_b.is_some(), "second client must see echo");
    let _ = handle.stop();
}
