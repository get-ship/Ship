//! Acceptance #1: the gated ship-js SSR path is WIRED into deck-core's serve
//! path — `serve_view_payload` returns ship-js first-paint when the
//! `DECK_SSR=ship-js` gate is on (and the renderer is registered), and the
//! unchanged Rust render-eval output when the gate is off. ADDITIVE: flag-off
//! is the default render-eval; the flag only swaps the first-paint source.
//!
//! This drives the SAME public entrypoint the daemon's HTTP handler calls
//! (`deck_core::serve_view_payload[_subpath]`), through the registered
//! `deck-ssr` renderer (pooled isolate, synthesize→bundle→run) — proving the
//! seam end-to-end, not just the capability in isolation.

use std::sync::Arc;

use deck_core::{DaemonMode, DataPort};
use deck_ssr::build::repo_root;
use serde_json::Value;

/// Host data surface for the `<data live source="api.demo.clock()">` view.
struct DemoData;
impl DataPort for DemoData {
    fn resolve(&self, domain: &str, op: &str, _args: &Value) -> Option<Value> {
        match (domain, op) {
            ("api", "demo.clock") => Some(serde_json::json!({ "now": "12:34:56" })),
            _ => None,
        }
    }
}

/// Pin the resolver at the conformance app + register the ship-js renderer.
/// Idempotent (OnceLock + a process-global root), safe to call once per binary.
fn setup() {
    let repo = repo_root().expect("repo root");
    deck_core::resolver::set_project_root(repo.join("crates/deck/deck-conformance/app"));
    deck_ssr::serve::register(Some(Arc::new(DemoData))).expect("register ship-js renderer");
}

fn payload_html(resp: &deck_core::DeckResponse) -> String {
    assert_eq!(resp.status, 200, "serve must be 200; body: {}", String::from_utf8_lossy(&resp.body));
    let v: Value = serde_json::from_slice(&resp.body).expect("payload is JSON");
    v["html"].as_str().expect("payload carries html").to_string()
}

#[test]
fn wired_serve_gate_selects_ship_js_or_render_eval() {
    setup();

    // ── board (a REAL controller({...}) DSL view) ──────────────────────────
    // Flag ON: ship-js runs the controller, so first paint carries the
    // controller-signal rows the Rust render-eval can't produce.
    unsafe { std::env::set_var("DECK_SSR", "ship-js") };
    let on = payload_html(&deck_core::serve_view_payload(DaemonMode::Development, "board"));
    assert!(on.contains("<ul>"), "ship-js {{#if}} branch renders; got:\n{on}");
    assert!(on.contains("Alpha") && on.contains("Bravo"), "ship-js {{#for}} rows; got:\n{on}");
    assert!(on.contains("<demo-chip>"), "nested component tag; got:\n{on}");
    assert!(
        on.matches("class=\"flagged\"").count() == 1,
        "per-row class: directive; got:\n{on}"
    );

    // Flag OFF: the default render-eval path — it cannot run the controller
    // signal, so the controller-sourced rows are absent. Proves the gate
    // selects the path and that flag-off is the unchanged render-eval output.
    unsafe { std::env::set_var("DECK_SSR", "off") };
    let off = payload_html(&deck_core::serve_view_payload(DaemonMode::Development, "board"));
    assert!(
        !off.contains("Alpha") && !off.contains("Bravo"),
        "render-eval must NOT carry controller-signal rows; got:\n{off}"
    );
    assert_ne!(on, off, "the two paths must differ; the gate is doing nothing otherwise");

    // ── feed (<data live> + host-dispatch) ─────────────────────────────────
    // Flag ON: the live op resolves over the registered DataPort, so first
    // paint reflects the resolved state, not `loading`.
    unsafe { std::env::set_var("DECK_SSR", "ship-js") };
    let feed_on = payload_html(&deck_core::serve_view_payload(DaemonMode::Development, "feed"));
    assert!(
        feed_on.contains("state: ok"),
        "ship-js resolves the live <data> over host-dispatch; got:\n{feed_on}"
    );

    // ── greet/[id] (route param via host-dispatch) ─────────────────────────
    // The router captures `id` from the sub-path; ship-js resolves
    // `route.params.id()` over host-dispatch into first paint.
    let greet_on =
        payload_html(&deck_core::serve_view_payload_subpath(DaemonMode::Development, "greet", "zelda"));
    assert!(
        greet_on.contains("Hello, zelda"),
        "ship-js resolves the route param into first paint; got:\n{greet_on}"
    );

    // Restore the default so a later test in this binary sees flag-off.
    unsafe { std::env::set_var("DECK_SSR", "off") };
    let greet_off =
        payload_html(&deck_core::serve_view_payload_subpath(DaemonMode::Development, "greet", "zelda"));
    assert!(
        !greet_off.contains("Hello, zelda"),
        "render-eval first paint must not carry the host-resolved param; got:\n{greet_off}"
    );
}
