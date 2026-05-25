//! `/api/_meta` contract test: the JSON payload must carry
//! `compilerVersion`, `daemonVersion`, and a top-level `builtAt`
//! so the shell's drift-detection banner can compare against the
//! `_compilerVersion` marker baked into compiled bundles.

use axum::Extension;
use serde_json::Value;

use crate::meta::{serve_meta, DaemonBuildInfo};

#[tokio::test]
async fn meta_payload_carries_compiler_and_daemon_versions() {
    let dbi = DaemonBuildInfo {
        cargo: "0.2.0",
        git: "deadbeef",
        dirty: false,
        built_at: "2026-05-14T00:00:00Z",
    };
    let axum::Json(payload): axum::Json<Value> = serve_meta(Some(Extension(dbi))).await;

    let compiler = payload
        .get("compilerVersion")
        .expect("compilerVersion field present");
    assert!(compiler.get("cargo").and_then(Value::as_str).is_some());
    assert!(compiler.get("git").and_then(Value::as_str).is_some());
    assert!(compiler.get("builtAt").and_then(Value::as_str).is_some());

    let daemon = payload
        .get("daemonVersion")
        .expect("daemonVersion field present");
    assert_eq!(daemon.get("cargo").and_then(Value::as_str), Some("0.2.0"));
    assert_eq!(daemon.get("git").and_then(Value::as_str), Some("deadbeef"));
    assert_eq!(daemon.get("dirty").and_then(Value::as_bool), Some(false));
    assert_eq!(
        daemon.get("builtAt").and_then(Value::as_str),
        Some("2026-05-14T00:00:00Z"),
    );

    assert_eq!(
        payload.get("builtAt").and_then(Value::as_str),
        Some("2026-05-14T00:00:00Z"),
        "top-level builtAt mirrors daemon timestamp for at-a-glance freshness",
    );
}

#[tokio::test]
async fn meta_payload_handles_missing_extension() {
    // When the daemon hasn't wired the build-info extension (e.g. an
    // older test harness mounting the bare route), the handler should
    // still return a well-shaped payload with `"unknown"` placeholders
    // rather than panicking.
    let axum::Json(payload): axum::Json<Value> = serve_meta(None).await;
    let daemon = payload.get("daemonVersion").expect("daemonVersion present");
    assert_eq!(daemon.get("git").and_then(Value::as_str), Some("unknown"));
    assert_eq!(daemon.get("cargo").and_then(Value::as_str), Some("unknown"));
}
