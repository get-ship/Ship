//! Protocol tests for `CodexSession`. Drives the session over an
//! in-memory duplex pipe and pins the JSON-RPC exchange that a real
//! `codex app-server` would see. Focus is on wire-level contract:
//! handshake order, request/response plumbing, notification mapping,
//! cancellation.

use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, Lines, ReadHalf, WriteHalf};
use tokio::sync::oneshot;

use super::codex_session::CodexSession;
use crate::services::server_session::{ServerSession, SessionEvent};

type FakeReader = Lines<BufReader<ReadHalf<DuplexStream>>>;
type FakeWriter = WriteHalf<DuplexStream>;

/// Build a duplex pipe and return the four split halves.
/// Naming mirrors the session's view: `session_rx` is read by the
/// session, `session_tx` is written by the session.
fn transport() -> (
    ReadHalf<DuplexStream>,
    WriteHalf<DuplexStream>,
    FakeReader,
    FakeWriter,
) {
    let (a, b) = tokio::io::duplex(8192);
    let (session_rx, session_tx) = tokio::io::split(a);
    let (fake_rx, fake_tx) = tokio::io::split(b);
    (
        session_rx,
        session_tx,
        BufReader::new(fake_rx).lines(),
        fake_tx,
    )
}

async fn read_json_line(reader: &mut FakeReader) -> Value {
    let line = reader
        .next_line()
        .await
        .expect("read line")
        .expect("line available");
    serde_json::from_str(&line).expect("valid JSON-RPC line")
}

async fn write_json_line(writer: &mut FakeWriter, value: &Value) {
    let line = format!("{}\n", value);
    writer.write_all(line.as_bytes()).await.expect("write line");
    writer.flush().await.expect("flush");
}

async fn respond_handshake(reader: &mut FakeReader, writer: &mut FakeWriter) {
    let init = read_json_line(reader).await;
    assert_eq!(init["jsonrpc"], "2.0");
    assert_eq!(init["method"], "initialize");
    let id = init["id"].as_u64().expect("initialize id");
    write_json_line(
        writer,
        &json!({"jsonrpc": "2.0", "id": id, "result": {"server": {"name": "codex"}}}),
    )
    .await;

    let initialized = read_json_line(reader).await;
    assert_eq!(initialized["method"], "initialized");
    assert!(initialized.get("id").is_none());
}

#[tokio::test]
async fn handshake_sends_initialize_then_initialized() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();

    let fake = tokio::spawn(async move {
        let init = read_json_line(&mut f_rx).await;
        assert_eq!(init["jsonrpc"], "2.0");
        assert_eq!(init["method"], "initialize");
        assert_eq!(init["params"]["client"]["name"], "shipd");
        let id = init["id"].as_u64().expect("initialize id");
        write_json_line(
            &mut f_tx,
            &json!({"jsonrpc": "2.0", "id": id, "result": {}}),
        )
        .await;

        let notif = read_json_line(&mut f_rx).await;
        assert_eq!(notif["method"], "initialized");
        assert!(notif.get("id").is_none());
    });

    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake succeeds");
    fake.await.expect("fake driver");
    session.close().await.expect("close");
}

#[tokio::test]
async fn start_turn_routes_thread_then_turn_request() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();

    let fake = tokio::spawn(async move {
        respond_handshake(&mut f_rx, &mut f_tx).await;

        let thread_req = read_json_line(&mut f_rx).await;
        assert_eq!(thread_req["method"], "thread/start");
        let id = thread_req["id"].as_u64().expect("thread id");
        write_json_line(
            &mut f_tx,
            &json!({"jsonrpc": "2.0", "id": id, "result": {"thread_id": "th-42"}}),
        )
        .await;

        let turn_req = read_json_line(&mut f_rx).await;
        assert_eq!(turn_req["method"], "turn/start");
        assert_eq!(turn_req["params"]["thread_id"], "th-42");
        assert_eq!(turn_req["params"]["input"], "hello world");
        let id = turn_req["id"].as_u64().expect("turn id");
        write_json_line(
            &mut f_tx,
            &json!({"jsonrpc": "2.0", "id": id, "result": {"turn_id": "tr-7"}}),
        )
        .await;
    });

    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake");
    let turn_id = session
        .start_turn("hello world")
        .await
        .expect("start_turn succeeds");
    assert_eq!(turn_id, "tr-7");

    fake.await.expect("fake driver");
}

#[tokio::test]
async fn start_turn_reuses_thread_across_calls() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();

    let fake = tokio::spawn(async move {
        respond_handshake(&mut f_rx, &mut f_tx).await;

        // Only one thread/start should cross the wire even for two turns.
        let thread_req = read_json_line(&mut f_rx).await;
        assert_eq!(thread_req["method"], "thread/start");
        let id = thread_req["id"].as_u64().expect("thread id");
        write_json_line(
            &mut f_tx,
            &json!({"jsonrpc": "2.0", "id": id, "result": {"thread_id": "th-1"}}),
        )
        .await;

        for turn in ["tr-a", "tr-b"] {
            let turn_req = read_json_line(&mut f_rx).await;
            assert_eq!(turn_req["method"], "turn/start");
            assert_eq!(turn_req["params"]["thread_id"], "th-1");
            let id = turn_req["id"].as_u64().expect("turn id");
            write_json_line(
                &mut f_tx,
                &json!({"jsonrpc": "2.0", "id": id, "result": {"turn_id": turn}}),
            )
            .await;
        }
    });

    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake");
    assert_eq!(session.start_turn("one").await.unwrap(), "tr-a");
    assert_eq!(session.start_turn("two").await.unwrap(), "tr-b");

    fake.await.expect("fake driver");
}

#[tokio::test]
async fn agent_message_delta_maps_to_session_event() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();
    let (ready_tx, ready_rx) = oneshot::channel::<()>();

    let fake = tokio::spawn(async move {
        respond_handshake(&mut f_rx, &mut f_tx).await;
        ready_rx.await.expect("subscribe ready");
        write_json_line(
            &mut f_tx,
            &json!({
                "jsonrpc": "2.0",
                "method": "item/agentMessage/delta",
                "params": { "turn_id": "tr-1", "delta": "hello from codex" }
            }),
        )
        .await;
    });

    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake");
    let mut rx = session.subscribe("tr-1").await.expect("subscribe");
    ready_tx.send(()).expect("signal");

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("event arrives")
        .expect("channel open");
    match event {
        SessionEvent::AgentMessage { text } => assert_eq!(text, "hello from codex"),
        other => panic!("unexpected event: {other:?}"),
    }

    fake.await.expect("fake driver");
}

#[tokio::test]
async fn turn_completed_propagates() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();
    let (ready_tx, ready_rx) = oneshot::channel::<()>();

    let fake = tokio::spawn(async move {
        respond_handshake(&mut f_rx, &mut f_tx).await;
        ready_rx.await.expect("subscribe ready");
        write_json_line(
            &mut f_tx,
            &json!({
                "jsonrpc": "2.0",
                "method": "turn/completed",
                "params": { "turn_id": "tr-1", "stop_reason": "end_turn" }
            }),
        )
        .await;
    });

    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake");
    let mut rx = session.subscribe("tr-1").await.expect("subscribe");
    ready_tx.send(()).expect("signal");

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("event arrives")
        .expect("channel open");
    match event {
        SessionEvent::TurnCompleted { stop_reason } => assert_eq!(stop_reason, "end_turn"),
        other => panic!("unexpected event: {other:?}"),
    }

    fake.await.expect("fake driver");
}

#[tokio::test]
async fn cancel_sends_turn_interrupt() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();

    let fake = tokio::spawn(async move {
        respond_handshake(&mut f_rx, &mut f_tx).await;

        let req = read_json_line(&mut f_rx).await;
        assert_eq!(req["method"], "turn/interrupt");
        assert_eq!(req["params"]["turn_id"], "tr-9");
        let id = req["id"].as_u64().expect("interrupt id");
        write_json_line(
            &mut f_tx,
            &json!({"jsonrpc": "2.0", "id": id, "result": {}}),
        )
        .await;
    });

    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake");
    let _rx = session.subscribe("tr-9").await.expect("subscribe");
    session.cancel("tr-9").await.expect("cancel");

    fake.await.expect("fake driver");
}

#[tokio::test]
async fn provider_id_is_codex() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();
    let fake = tokio::spawn(async move {
        respond_handshake(&mut f_rx, &mut f_tx).await;
    });
    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake");
    assert_eq!(session.provider(), "codex");
    fake.await.expect("fake driver");
}

#[tokio::test]
async fn unknown_notification_is_ignored() {
    let (s_rx, s_tx, mut f_rx, mut f_tx) = transport();
    let (ready_tx, ready_rx) = oneshot::channel::<()>();

    let fake = tokio::spawn(async move {
        respond_handshake(&mut f_rx, &mut f_tx).await;
        ready_rx.await.expect("subscribe ready");
        // Unknown method — must not populate a subscriber event.
        write_json_line(
            &mut f_tx,
            &json!({
                "jsonrpc": "2.0",
                "method": "item/unknownThing/foo",
                "params": { "turn_id": "tr-1", "data": "ignored" }
            }),
        )
        .await;
        // Follow with a mapped event to prove the subscriber is alive.
        write_json_line(
            &mut f_tx,
            &json!({
                "jsonrpc": "2.0",
                "method": "turn/completed",
                "params": { "turn_id": "tr-1", "stop_reason": "done" }
            }),
        )
        .await;
    });

    let session = CodexSession::with_transport(Box::new(s_rx), Box::new(s_tx))
        .await
        .expect("handshake");
    let mut rx = session.subscribe("tr-1").await.expect("subscribe");
    ready_tx.send(()).expect("signal");

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("event arrives")
        .expect("channel open");
    assert!(matches!(event, SessionEvent::TurnCompleted { .. }));
    fake.await.expect("fake driver");
}
