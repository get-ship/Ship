//! SSR acceptance: a SYNTHESIZED view renders correct first-paint HTML through
//! ship-js (deno_core/V8), with the route param resolved over host-dispatch —
//! NOT the Rust render-eval. This is the production TS-SSR path, beside it.

use std::sync::Arc;

use deck_ssr::build::{repo_root, synthesize_and_bundle, synthesize_and_bundle_with, ClientAlias};
use deck_ssr::host::SsrDispatch;
use deck_ssr::run::render;
use deck_core::DataPort;
use serde_json::{json, Value};
use sui_compiler::CompileMode;

const GREET: &str = "crates/deck/deck-conformance/app/views/greet/[id]/page.sui";
const GLYPH: &str = "crates/deck/deck-conformance/app/views/glyph/page.sui";
const BOARD: &str = "crates/deck/deck-conformance/app/views/board/page.sui";
const BOARD_CTRL: &str = "crates/deck/deck-conformance/app/views/board/controller.ts";
const FEED: &str = "crates/deck/deck-conformance/app/views/feed/page.sui";
const DEMO_SERVER: &str = "crates/deck/deck-conformance/app/clients/demo.server.ts";
const MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";

/// A test data port: answers the demo ops the host-dispatch wire forwards.
struct DemoData;
impl DataPort for DemoData {
    fn resolve(&self, domain: &str, op: &str, _args: &Value) -> Option<Value> {
        match (domain, op) {
            ("api", "demo.clock") => Some(json!({ "now": "12:34:56" })),
            _ => None,
        }
    }
}

#[test]
fn greet_first_paint_via_ship_js_with_route_through_host_dispatch() {
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/greet");

    // Synthesize + Rolldown-bundle into one self-contained ES module.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for bundling");
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, GREET, MANIFEST, CompileMode::View, &[], &build_dir))
        .expect("synthesize + bundle");
    assert!(bundle.len() > 1000, "bundle should be a real self-contained module");
    // `ship` is the only external import (the host wire); nothing else leaks.
    assert!(
        !bundle.contains("@ship/sui-runtime") && !bundle.contains("@ship/deck/route"),
        "framework specifiers must be inlined to their server entries; only `ship` stays external"
    );

    // Render in the isolate with the route the deck router matched for the request.
    let dispatch = Arc::new(SsrDispatch::for_request("/greet/zelda", json!({ "id": "zelda" }), json!({})));
    let out = render(&bundle, dispatch).expect("render in ship-js");

    // First paint produced by ship-js run_module (NOT render-eval).
    assert!(
        out.html.contains("<h1>Hello, zelda</h1>"),
        "first paint must contain the route-resolved heading; got:\n{}",
        out.html
    );
    // The value came through the host-dispatch wire.
    assert_eq!(out.route_params["id"], "zelda", "route param resolved via host-dispatch");
    assert!(
        out.call_path.iter().any(|op| op == "route.current"),
        "the route value must have crossed the wire (route.current); call path: {:?}",
        out.call_path
    );
}

#[test]
fn board_control_flow_first_paint_via_ship_js() {
    // The control-flow-heavy surface: {#if}/{#for}/class:/nested <demo-chip>,
    // backed by a typed authored controller. Proves the full TemplateEmitter
    // (bindIf/bindFor) runs server-side on the server-DOM and serializes.
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/board");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let bundle = rt
        .block_on(synthesize_and_bundle_with(
            &repo,
            BOARD,
            MANIFEST,
            CompileMode::View,
            Some(&repo.join(BOARD_CTRL)),
            &[],
            &build_dir,
        ))
        .expect("synthesize + bundle board");

    let dispatch = Arc::new(SsrDispatch::for_request("/board", json!({}), json!({})));
    let out = render(&bundle, dispatch).expect("render board");

    // {#if items().length > 0} → true → the list renders both rows.
    assert!(out.html.contains("<ul>"), "if-branch (the list) must render; got:\n{}", out.html);
    assert!(
        out.html.contains("Alpha") && out.html.contains("Bravo"),
        "{{#for}} must render every row; got:\n{}",
        out.html
    );
    // Nested component renders as its custom-element tag with slotted content.
    assert!(out.html.contains("<demo-chip>"), "nested component tag must render; got:\n{}", out.html);
    // class:flagged={item.flagged} → only Bravo (flagged) carries the class.
    assert!(
        out.html.matches("class=\"flagged\"").count() == 1,
        "class: directive must reflect per-row state (one flagged row); got:\n{}",
        out.html
    );
    // The remove buttons (on:click handlers) render too.
    assert!(out.html.contains("remove"), "on: handlers' elements must render; got:\n{}", out.html);
}

#[test]
fn glyph_svg_first_paint_serializes_namespaced() {
    // The broader server-DOM case (acceptance #3): an inline `<svg>` subtree.
    // The synthetic render builds it via `createElementNS(SVG_NS, …)`, and the
    // shared server-DOM serializes namespaced elements — empty SVG elements
    // self-close (`<circle .../>`), which HTML void elements never do.
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/glyph");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, GLYPH, MANIFEST, CompileMode::View, &[], &build_dir))
        .expect("synthesize + bundle glyph");

    let dispatch = Arc::new(SsrDispatch::for_request("/glyph", json!({}), json!({})));
    let out = render(&bundle, dispatch).expect("render glyph");

    assert!(out.html.contains("<svg"), "svg root must render; got:\n{}", out.html);
    assert!(
        out.html.contains("<circle") && out.html.contains("r=\"10\""),
        "svg child + its attribute must render; got:\n{}",
        out.html
    );
    assert!(out.html.contains("<path"), "second svg child must render; got:\n{}", out.html);
    // Namespace-aware serialization: empty SVG elements self-close. (HTML void
    // elements like <br> never emit the slash — this is the SVG branch.)
    assert!(
        out.html.contains("/>"),
        "empty SVG elements must self-close (namespaced serialization); got:\n{}",
        out.html
    );
}

#[test]
fn feed_data_client_value_resolves_via_host_dispatch() {
    // The <data live source="api.demo.clock()"> + <client> surface. The client
    // server entry resolves the source over the host-dispatch wire; after the
    // async flush, first paint carries the resolved state (not `loading`).
    let repo = repo_root().expect("repo root");
    let build_dir = repo.join("target/deck-ssr-test/feed");
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let client = ClientAlias {
        specifier: "@app/clients/demo".to_string(),
        server_entry: repo.join(DEMO_SERVER),
    };
    let bundle = rt
        .block_on(synthesize_and_bundle(&repo, FEED, MANIFEST, CompileMode::View, &[client], &build_dir))
        .expect("synthesize + bundle feed");

    let dispatch = Arc::new(
        SsrDispatch::for_request("/feed", json!({}), json!({})).with_data(Arc::new(DemoData)),
    );
    let out = render(&bundle, dispatch).expect("render feed");

    // The live data op crossed the host-dispatch wire.
    assert!(
        out.call_path.iter().any(|op| op == "api.demo.clock"),
        "the <data> source op must cross the wire; call path: {:?}",
        out.call_path
    );
    // After the flush, the live signal is resolved (ok), not loading.
    assert!(
        out.html.contains("state: ok"),
        "first paint must reflect the resolved live state via host-dispatch; got:\n{}",
        out.html
    );
}
