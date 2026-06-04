//! Slice 8 — mutation endpoint (no browser, ship-free).
//!
//! 1. Parses the real form binding key from `views/todos/page.sui` via
//!    `sui_compiler::{parse, bindings}` — the same path the handler uses.
//! 2. POSTs `application/x-www-form-urlencoded text=hi` to
//!    `/_sui/mutation/todos?form=<key>` with a `Referer` header.
//! 3. Asserts HTTP 303 and `Location` == `Referer`.
//! 4. Resolves `ship.demo.todos()` directly against the store and asserts
//!    the new item appears.
//!
//! Run: `cargo test -p deck-demo --test mutation`

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use deck_demo::data_port::{DemoStore, resolve_demo};
use deck_demo::{deck_app, default_app_dir};
use serde_json::json;
use sui_compiler::{bindings, keys::BindingKind, parse};
use tower::ServiceExt;

/// Read the mutation binding key from the todos view source.
fn todos_form_key() -> String {
    let app_dir = default_app_dir();
    let source =
        std::fs::read_to_string(app_dir.join("views/todos/page.sui")).expect("todos page.sui");
    let program = parse::parse(&source).expect("parse todos");
    let reg = bindings::build(&program);
    reg.iter()
        .find(|b| matches!(b.kind, BindingKind::Mutation))
        .expect("mutation binding in todos view")
        .key
        .clone()
}

/// POST text=hi → 303 redirect back to Referer.
#[tokio::test(flavor = "multi_thread")]
async fn mutation_post_returns_303_to_referer() {
    let key = todos_form_key();
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/_sui/mutation/todos?form={key}"))
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::REFERER, "http://localhost:7878/_sui/view/todos")
                .body(Body::from("text=hi"))
                .unwrap(),
        )
        .await
        .expect("response");

    assert_eq!(
        resp.status(),
        StatusCode::SEE_OTHER,
        "mutation must 303 redirect; got {}",
        resp.status()
    );
    let location = resp
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .expect("Location header");
    assert_eq!(
        location,
        "http://localhost:7878/_sui/view/todos",
        "Location must equal Referer; got: {location}"
    );
}

/// POST with no Referer → 303 redirect to `/`.
#[tokio::test(flavor = "multi_thread")]
async fn mutation_post_no_referer_redirects_to_root() {
    let key = todos_form_key();
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/_sui/mutation/todos?form={key}"))
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("text=no-referer"))
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let location = resp
        .headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(location, "/", "no Referer → redirect to /; got: {location}");
}

/// POST with unknown form key → 404.
#[tokio::test(flavor = "multi_thread")]
async fn mutation_unknown_form_key_is_404() {
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sui/mutation/todos?form=b9999")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("text=x"))
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "unknown form key must 404"
    );
}

/// POST to unknown view → 404.
#[tokio::test(flavor = "multi_thread")]
async fn mutation_unknown_view_is_404() {
    let app = deck_app(default_app_dir());
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/_sui/mutation/no-such-view?form=b0")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("text=x"))
                .unwrap(),
        )
        .await
        .expect("response");
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "unknown view must 404"
    );
}

/// After a successful mutation, the store contains the new item.
/// This test drives the store directly (no HTTP round-trip) to verify
/// the resolver semantics independently of the HTTP layer.
#[tokio::test]
async fn resolve_demo_shows_added_todo_after_add() {
    let store = DemoStore::new();

    // Add via resolver.
    resolve_demo("demo.addTodo", &json!({"text": "hi"}), &store)
        .await
        .unwrap();

    // List should show the item.
    let v = resolve_demo("demo.todos", &json!({}), &store)
        .await
        .unwrap();
    let items = v["items"].as_array().unwrap();
    assert_eq!(items.len(), 1, "one item after addTodo");
    assert_eq!(items[0]["text"], "hi");
}

/// The todos view contains exactly one mutation binding pointing at
/// `ship.demo.addTodo`.  This proves the wire vocabulary contract is in place.
#[test]
fn todos_view_has_mutation_binding_for_add_todo() {
    let app_dir = default_app_dir();
    let source =
        std::fs::read_to_string(app_dir.join("views/todos/page.sui")).expect("todos page.sui");
    let program = parse::parse(&source).expect("parse");
    let reg = bindings::build(&program);
    let mutations: Vec<_> = reg
        .iter()
        .filter(|b| matches!(b.kind, BindingKind::Mutation))
        .collect();
    assert_eq!(mutations.len(), 1, "todos view must have exactly one mutation");
    assert_eq!(
        mutations[0].expr, "ship.demo.addTodo",
        "mutation must point at ship.demo.addTodo; got: {}",
        mutations[0].expr
    );
}

/// The clock view has exactly one live data source pointing at
/// `ship.demo.clock()`.
#[test]
fn clock_view_has_live_data_for_demo_clock() {
    let app_dir = default_app_dir();
    let source =
        std::fs::read_to_string(app_dir.join("views/clock/page.sui")).expect("clock page.sui");
    let program = parse::parse(&source).expect("parse");
    let live: Vec<_> = program
        .data
        .iter()
        .filter(|d| d.live)
        .collect();
    assert_eq!(live.len(), 1, "clock view must have exactly one live data source");
    assert_eq!(
        live[0].source.source, "ship.demo.clock()",
        "live source must be ship.demo.clock(); got: {}",
        live[0].source.source
    );
    assert_eq!(live[0].name, "t");
}
