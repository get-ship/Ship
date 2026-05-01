//! E2E tests for the F-1 SSR walker debug endpoint
//! (`GET /_sui/render/{view}`).
//!
//! Phase E-2 fragment-render tests live in `sui_render_fragment.rs`
//! to keep this file focused on the GET surface. Both share
//! `tests/common/mod.rs` for the daemon harness.

mod common;

use common::TestDaemon;

#[tokio::test]
async fn agents_view_renders() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_sui/render/agents"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let ct = res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("text/html"), "expected text/html, got {ct}");
    let body = res.text().await.unwrap();
    assert!(body.contains("<!-- BEGIN SSR walker output -->"));
    assert!(body.contains("<!-- END SSR walker output -->"));
    assert!(body.contains("<!--sui:text:") || body.contains("<sui-if key=") || body.contains("on:click="));
    assert!(body.contains("__set_init__"));
}

#[tokio::test]
async fn counter_view_renders() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_sui/render/counter"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let ct = res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("text/html"), "expected text/html, got {ct}");
    let body = res.text().await.unwrap();
    assert!(body.contains("<!-- BEGIN SSR walker output -->"));
    assert!(body.contains("<!-- END SSR walker output -->"));
    assert!(
        body.contains(r#"<div class="counter">"#),
        "counter wrapper missing in body: {body}"
    );
    assert!(
        body.contains(r#"<p>Count: <span data-test="value""#),
        "count display missing in body: {body}"
    );
    assert!(
        body.contains("<!--sui:text:") && body.contains("-->0<!--/sui-->"),
        "expected text-marker comments wrapping initial count `0`: {body}"
    );
    let clicks = body.matches(r#"on:click=""#).count();
    assert!(
        clicks >= 3,
        "expected at least 3 on:click markers (inc/dec/reset), got {clicks}: {body}"
    );
    assert!(
        body.contains(r#"<script type="application/json" id="__set_init__">"#),
        "init island missing: {body}"
    );
    assert!(
        body.contains(r#""count":0"#),
        "init island must seed count=0: {body}"
    );
}

#[tokio::test]
async fn counter_view_render_json() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_sui/render/counter?format=json"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let html = body.get("html").and_then(|v| v.as_str()).unwrap();
    assert!(html.contains(r#"<div class="counter">"#));
    let init = body.get("init").expect("init key");
    assert_eq!(init.get("count"), Some(&serde_json::json!(0)));
}

#[tokio::test]
async fn unknown_view_404() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_sui/render/does-not-exist"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
    let body = res.text().await.unwrap();
    assert!(
        body.contains("views/does-not-exist/view.sui"),
        "404 body must mention the path it tried — got: {body}"
    );
}

#[tokio::test]
async fn format_json() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .get(d.url("/_sui/render/agents?format=json"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let ct = res
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        ct.starts_with("application/json"),
        "expected application/json, got {ct}"
    );
    let body: serde_json::Value = res.json().await.unwrap();
    assert!(body.get("html").and_then(|v| v.as_str()).is_some());
    let init = body.get("init").expect("init key");
    let agents = init.get("agents").expect("agents key in init");
    assert_eq!(agents, &serde_json::json!({ "kind": "loading" }));
}
