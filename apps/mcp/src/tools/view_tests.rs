//! Unit tests for `view.*` tool handlers.
//!
//! Full daemon round-trip coverage lives in the view-session e2e pipeline
//! (slice 5). These tests cover request-shape parsing and error propagation
//! when the daemon is unreachable.

use super::*;
use crate::requests::{
    ViewAppendRequest, ViewCloseRequest, ViewOpenRequest, ViewPatchRequest, ViewSignalRequest,
};

fn ensure_no_daemon() {
    // Tests assume the daemon is not running under this temp $HOME.
    // SAFETY: single-threaded tokio test; no concurrent env access.
    let tmp = tempfile::tempdir().unwrap();
    unsafe {
        std::env::set_var("HOME", tmp.path());
        std::env::set_var("SHIP_HOME", tmp.path());
    }
    // Leak tmp so directory persists for the duration of the test.
    std::mem::forget(tmp);
}

#[tokio::test]
async fn open_without_daemon_returns_structured_error() {
    ensure_no_daemon();
    let req = ViewOpenRequest {
        template: "canvas/test-ping".into(),
        title: Some("hello".into()),
        initial_context: None,
    };
    let out = view_open("agent.test", req).await;
    assert!(out.starts_with("Error:"), "expected error surface, got: {out}");
}

#[tokio::test]
async fn patch_without_daemon_returns_structured_error() {
    ensure_no_daemon();
    let req = ViewPatchRequest {
        view_id: "01HZZ".into(),
        selector: "#root".into(),
        html: "<p/>".into(),
    };
    let out = view_patch("agent.test", req).await;
    assert!(out.starts_with("Error:"), "got: {out}");
}

#[tokio::test]
async fn append_signal_close_return_errors_when_no_daemon() {
    ensure_no_daemon();
    assert!(view_append(
        "agent.test",
        ViewAppendRequest { view_id: "x".into(), selector: "#y".into(), html: "".into() }
    )
    .await
    .starts_with("Error:"));
    assert!(view_signal(
        "agent.test",
        ViewSignalRequest { view_id: "x".into(), name: "n".into(), value: serde_json::json!(1) }
    )
    .await
    .starts_with("Error:"));
    assert!(view_close("agent.test", ViewCloseRequest { view_id: "x".into() })
        .await
        .starts_with("Error:"));
}
