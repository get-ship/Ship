//! Protocol tests for `OpenCodeSession`. Drives the session against an
//! in-process stub HTTP server to pin session-creation order, turn
//! submission body, SSE event mapping, cancel shape, and session reuse.
//! Stub server lives in the sibling `tests_opencode_session_stub` module.

#[path = "tests_opencode_session_stub.rs"]
mod stub;

use std::sync::Arc;
use std::time::Duration;

use stub::{Response, StubServer, handler_with_events};

use super::opencode_session::OpenCodeSession;
use crate::services::server_session::{ServerSession, SessionEvent};

#[tokio::test]
async fn start_turn_posts_to_session_then_message_endpoint() {
    let server = StubServer::spawn(handler_with_events("sess-1", "msg-1", vec![])).await;
    let session = OpenCodeSession::with_server(server.base_url.clone());

    let turn_id = session
        .start_turn("hello opencode")
        .await
        .expect("start_turn succeeds");
    assert_eq!(turn_id, "msg-1");

    // Give the server a moment to flush capture entries, then verify the
    // session hit /session before /session/{id}/message.
    tokio::time::sleep(Duration::from_millis(50)).await;
    let caps = server.captures().await;
    let non_event: Vec<_> = caps.iter().filter(|c| c.path != "/event").collect();
    assert_eq!(non_event.len(), 2, "expected 2 requests, got: {caps:?}");
    assert_eq!(non_event[0].method, "POST");
    assert_eq!(non_event[0].path, "/session");
    assert_eq!(non_event[1].method, "POST");
    assert_eq!(non_event[1].path, "/session/sess-1/message");
    let body: serde_json::Value =
        serde_json::from_str(&non_event[1].body).expect("message body JSON");
    assert_eq!(body["role"], "user");
    assert_eq!(body["content"], "hello opencode");

    session.close().await.expect("close");
}

#[tokio::test]
async fn assistant_event_maps_to_agent_message() {
    let sse = vec![serde_json::json!({
        "type": "assistant",
        "properties": {
            "sessionID": "sess-1",
            "messageID": "msg-1",
            "text": "hi from opencode",
        }
    })];
    let server = StubServer::spawn(handler_with_events("sess-1", "msg-1", sse)).await;
    let session = OpenCodeSession::with_server(server.base_url.clone());

    let turn_id = session.start_turn("hi").await.expect("start_turn");
    let mut rx = session.subscribe(&turn_id).await.expect("subscribe");

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("event arrives")
        .expect("channel open");
    match event {
        SessionEvent::AgentMessage { text } => assert_eq!(text, "hi from opencode"),
        other => panic!("unexpected event: {other:?}"),
    }

    session.close().await.expect("close");
}

#[tokio::test]
async fn session_completed_maps_to_turn_completed() {
    let sse = vec![serde_json::json!({
        "type": "session_completed",
        "properties": { "sessionID": "sess-1", "messageID": "msg-1" }
    })];
    let server = StubServer::spawn(handler_with_events("sess-1", "msg-1", sse)).await;
    let session = OpenCodeSession::with_server(server.base_url.clone());

    let turn_id = session.start_turn("go").await.expect("start_turn");
    let mut rx = session.subscribe(&turn_id).await.expect("subscribe");

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("event arrives")
        .expect("channel open");
    match event {
        SessionEvent::TurnCompleted { stop_reason } => assert_eq!(stop_reason, "completed"),
        other => panic!("unexpected event: {other:?}"),
    }

    session.close().await.expect("close");
}

#[tokio::test]
async fn cancel_sends_delete_request() {
    let server = StubServer::spawn(handler_with_events("sess-1", "msg-1", vec![])).await;
    let session = OpenCodeSession::with_server(server.base_url.clone());

    let turn_id = session.start_turn("cancel me").await.expect("start_turn");
    let _rx = session.subscribe(&turn_id).await.expect("subscribe");
    session.cancel(&turn_id).await.expect("cancel");

    tokio::time::sleep(Duration::from_millis(50)).await;
    let caps = server.captures().await;
    let deletes: Vec<_> = caps.iter().filter(|c| c.method == "DELETE").collect();
    assert_eq!(deletes.len(), 1, "expected one DELETE, got: {caps:?}");
    assert_eq!(deletes[0].path, "/session/sess-1/message/msg-1");

    session.close().await.expect("close");
}

#[tokio::test]
async fn session_id_reused_across_turns() {
    // Handler scripts two distinct message IDs per session so we can
    // confirm /session was only hit once.
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = counter.clone();
    let handler = move |method: &str, path: &str| -> Option<Response> {
        if method == "POST" && path == "/session" {
            return Some(Response::Json(serde_json::json!({ "id": "sess-reuse" })));
        }
        if method == "POST" && path == "/session/sess-reuse/message" {
            let n = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mid = format!("msg-{n}");
            return Some(Response::Json(serde_json::json!({ "messageID": mid })));
        }
        if method == "GET" && path == "/event" {
            return Some(Response::Sse(vec![]));
        }
        None
    };
    let server = StubServer::spawn(handler).await;
    let session = OpenCodeSession::with_server(server.base_url.clone());

    let t1 = session.start_turn("one").await.expect("turn 1");
    let t2 = session.start_turn("two").await.expect("turn 2");
    assert_eq!(t1, "msg-0");
    assert_eq!(t2, "msg-1");

    tokio::time::sleep(Duration::from_millis(50)).await;
    let caps = server.captures().await;
    let sessions: Vec<_> = caps
        .iter()
        .filter(|c| c.method == "POST" && c.path == "/session")
        .collect();
    assert_eq!(
        sessions.len(),
        1,
        "expected exactly one POST /session, got {sessions:?}"
    );

    session.close().await.expect("close");
}
