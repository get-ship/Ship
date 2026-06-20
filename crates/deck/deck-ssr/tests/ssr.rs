//! SSR acceptance: a SYNTHESIZED view renders correct first-paint HTML through
//! ship-js (deno_core/V8), with the route param resolved over host-dispatch —
//! NOT the Rust render-eval. This is the production TS-SSR path, beside it.

use std::sync::Arc;

use deck_ssr::build::{repo_root, synthesize_and_bundle};
use deck_ssr::host::SsrDispatch;
use deck_ssr::run::render;
use serde_json::json;
use sui_compiler::CompileMode;

const GREET: &str = "crates/deck/deck-conformance/app/views/greet/[id]/page.sui";
const MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";

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
