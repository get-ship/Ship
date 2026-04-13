//! Unit tests for `ViewSessionRegistry`.

use super::*;
use std::time::Duration;

fn fast_cfg() -> RegistryConfig {
    RegistryConfig {
        max_sessions_per_agent: 2,
        rate_limit_capacity: 3.0,
        rate_limit_refill_per_sec: 0.0,
        gc_timeout: Duration::from_millis(50),
    }
}

#[tokio::test]
async fn open_returns_unique_ids_and_enforces_quota() {
    let reg = ViewSessionRegistry::with_config(fast_cfg());
    let a = reg
        .open("agent.a".into(), "canvas/blank".into(), None, serde_json::json!({}))
        .await
        .unwrap();
    let b = reg
        .open("agent.a".into(), "canvas/blank".into(), None, serde_json::json!({}))
        .await
        .unwrap();
    assert_ne!(a, b, "view_ids must be unique");
    let err = reg
        .open("agent.a".into(), "canvas/blank".into(), None, serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, SessionError::QuotaExceeded { limit: 2, .. }));
}

#[tokio::test]
async fn close_requires_ownership() {
    let reg = ViewSessionRegistry::with_config(fast_cfg());
    let id = reg
        .open("owner".into(), "canvas/t".into(), None, serde_json::json!({}))
        .await
        .unwrap();
    let err = reg.close(&id, "other").await.unwrap_err();
    assert!(matches!(err, SessionError::NotOwner { .. }));
    reg.close(&id, "owner").await.unwrap();
    // Second close fails with NotFound.
    assert!(matches!(
        reg.close(&id, "owner").await.unwrap_err(),
        SessionError::NotFound { .. }
    ));
}

#[tokio::test]
async fn patch_requires_ownership_and_forwards_to_subscribers() {
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    let reg = ViewSessionRegistry::with_config(fast_cfg());
    let id = reg
        .open("owner".into(), "canvas/t".into(), None, serde_json::json!({}))
        .await
        .unwrap();

    let sse = reg.subscribe(&id).await.unwrap();
    let drain = tokio::spawn(async move {
        let bytes = sse.into_response().into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    });

    // Non-owner patch rejected.
    assert!(matches!(
        reg.patch(&id, "other", "#root", "<p>x</p>".into()).await.unwrap_err(),
        SessionError::NotOwner { .. }
    ));

    reg.patch(&id, "owner", "#root", "<p>hi</p>".into()).await.unwrap();
    reg.close(&id, "owner").await.unwrap();

    let body = drain.await.unwrap();
    assert!(body.contains("datastar-patch-elements"), "body: {body}");
    assert!(body.contains("<p>hi</p>"));
}

#[tokio::test]
async fn rate_limit_rejects_after_capacity_exhausted() {
    let reg = ViewSessionRegistry::with_config(fast_cfg());
    let id = reg
        .open("owner".into(), "canvas/t".into(), None, serde_json::json!({}))
        .await
        .unwrap();
    let _sse = reg.subscribe(&id).await.unwrap();
    for _ in 0..3 {
        reg.patch(&id, "owner", "#root", "<p/>".into()).await.unwrap();
    }
    let err = reg.patch(&id, "owner", "#root", "<p/>".into()).await.unwrap_err();
    assert!(matches!(err, SessionError::RateLimited { .. }));
}

#[tokio::test]
async fn gc_closes_idle_sessions_after_timeout() {
    let reg = ViewSessionRegistry::with_config(fast_cfg());
    let id = reg
        .open("owner".into(), "canvas/t".into(), None, serde_json::json!({}))
        .await
        .unwrap();
    // No subscribers ever attached — last_disconnect_at is set on open.
    assert_eq!(reg.list().await.len(), 1);
    tokio::time::sleep(Duration::from_millis(80)).await;
    reg.gc_now().await;
    assert!(
        reg.list().await.is_empty(),
        "idle session should have been GC'd"
    );
    // Subsequent patch on the GC'd id returns NotFound.
    assert!(matches!(
        reg.patch(&id, "owner", "#root", "<p/>".into()).await.unwrap_err(),
        SessionError::NotFound { .. }
    ));
}

#[tokio::test]
async fn gc_keeps_sessions_with_live_subscribers() {
    let reg = ViewSessionRegistry::with_config(fast_cfg());
    let id = reg
        .open("owner".into(), "canvas/t".into(), None, serde_json::json!({}))
        .await
        .unwrap();
    let _sse = reg.subscribe(&id).await.unwrap();
    tokio::time::sleep(Duration::from_millis(80)).await;
    reg.gc_now().await;
    assert_eq!(reg.list().await.len(), 1, "subscribed session must survive GC");
}

#[tokio::test]
async fn signal_broadcasts_datastar_signals() {
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    let reg = ViewSessionRegistry::with_config(fast_cfg());
    let id = reg
        .open("owner".into(), "canvas/t".into(), None, serde_json::json!({}))
        .await
        .unwrap();
    let sse = reg.subscribe(&id).await.unwrap();
    let drain = tokio::spawn(async move {
        let bytes = sse.into_response().into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    });
    reg.signal(&id, "owner", "counter", serde_json::json!(7)).await.unwrap();
    reg.close(&id, "owner").await.unwrap();
    let body = drain.await.unwrap();
    assert!(body.contains("datastar-patch-signals"), "body: {body}");
    assert!(body.contains("counter"));
    assert!(body.contains("7"));
}

#[tokio::test]
async fn not_found_returned_for_unknown_id() {
    let reg = ViewSessionRegistry::default();
    assert!(matches!(
        reg.patch("missing", "owner", "#x", "".into()).await.unwrap_err(),
        SessionError::NotFound { .. }
    ));
    assert!(matches!(
        reg.subscribe("missing").await.unwrap_err(),
        SessionError::NotFound { .. }
    ));
    assert!(matches!(
        reg.shell_context("missing").await.unwrap_err(),
        SessionError::NotFound { .. }
    ));
}
