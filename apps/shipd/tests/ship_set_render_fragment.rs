//! E2E tests for the Phase E-2 control-flow swap endpoint
//! (`POST /_ship-set/render-fragment/{view}`).
//!
//! Same harness as `ship_set_render.rs` — both share `tests/common`.

mod common;

use common::TestDaemon;

const AGENTS_VIEW_SRC: &str = include_str!("../../../views/agents/view.html.set");
const COUNTER_VIEW_SRC: &str =
    include_str!("../../../views/counter/view.html.set");

/// Walk a view's binding registry to find its first control-flow key
/// of a given kind.
fn find_key_of_kind(view_src: &str, kind: ship_set_compiler::keys::BindingKind) -> String {
    use ship_set_compiler::{bindings, parse};
    let p = parse::parse(view_src).expect("parse");
    let r = bindings::build(&p);
    r.iter()
        .find(|b| b.kind == kind)
        .unwrap_or_else(|| panic!("no binding of kind {:?}", kind))
        .key
        .clone()
}

fn outer_if_key(src: &str) -> String {
    find_key_of_kind(src, ship_set_compiler::keys::BindingKind::If)
}

#[tokio::test]
async fn agents_render_fragment_for_loading_branch() {
    let d = TestDaemon::start().await;
    let key = outer_if_key(AGENTS_VIEW_SRC);
    let res = d
        .bare()
        .post(d.url("/_ship-set/render-fragment/agents"))
        .json(&serde_json::json!({
            "init": { "agents": { "kind": "loading" } },
            "key": key,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "loading branch fragment");
    let body: serde_json::Value = res.json().await.unwrap();
    let html = body.get("html").and_then(|v| v.as_str()).unwrap();
    // Loading-state fails the outer if's `kind === 'ok'` test, no else
    // body, so the body is empty.
    assert!(html.is_empty(), "expected empty body for loading, got: {html}");
    assert_eq!(
        body.get("init").unwrap().get("agents").unwrap(),
        &serde_json::json!({ "kind": "loading" })
    );
}

#[tokio::test]
async fn agents_render_fragment_for_ok_branch() {
    let d = TestDaemon::start().await;
    let key = outer_if_key(AGENTS_VIEW_SRC);
    let res = d
        .bare()
        .post(d.url("/_ship-set/render-fragment/agents"))
        .json(&serde_json::json!({
            "init": {
                "agents": { "kind": "ok", "value": { "items": [] } },
                "items": []
            },
            "key": key,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: serde_json::Value = res.json().await.unwrap();
    let html = body.get("html").and_then(|v| v.as_str()).unwrap();
    assert!(
        html.contains(r#"data-test="empty""#),
        "expected empty-state markers in fragment HTML: {html}"
    );
    assert!(
        !html.contains(&format!(r#"data-set-if="{key}""#)),
        "fragment must not include boundary wrapper: {html}"
    );
}

#[tokio::test]
async fn render_fragment_unknown_key_400() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .post(d.url("/_ship-set/render-fragment/agents"))
        .json(&serde_json::json!({ "init": {}, "key": "b9999" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let text = res.text().await.unwrap();
    assert!(
        text.contains("unknown binding key"),
        "expected helpful error, got: {text}"
    );
}

#[tokio::test]
async fn render_fragment_unknown_view_404() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .post(d.url("/_ship-set/render-fragment/does-not-exist"))
        .json(&serde_json::json!({ "init": {}, "key": "b0" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
    let text = res.text().await.unwrap();
    assert!(
        text.contains("views/does-not-exist/view.html.set"),
        "404 body must mention path: {text}"
    );
}

#[tokio::test]
async fn render_fragment_invalid_body_400() {
    let d = TestDaemon::start().await;
    let res = d
        .bare()
        .post(d.url("/_ship-set/render-fragment/agents"))
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn render_fragment_leaf_key_400() {
    let d = TestDaemon::start().await;
    // Counter view has only leaf bindings (text + on:click). Targeting
    // a leaf must fail the control-flow guard with a 400.
    let leaf = find_key_of_kind(COUNTER_VIEW_SRC, ship_set_compiler::keys::BindingKind::Text);
    let res = d
        .bare()
        .post(d.url("/_ship-set/render-fragment/counter"))
        .json(&serde_json::json!({ "init": {}, "key": leaf }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let text = res.text().await.unwrap();
    assert!(
        text.contains("control-flow"),
        "expected control-flow guard message, got: {text}"
    );
}
