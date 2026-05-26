//! Regression: the component-mount state proxy (`__sp`) must expose the
//! `$signals` accessor — the canonical raw-signal handle controllers read
//! through after #121 (`state.$signals.x`).
//!
//! Without the `$signals` get-trap branch, the component-mount path returned
//! `undefined` for `state.$signals`, so every controller migrated to the typed
//! surface threw `Cannot read properties of undefined (reading '<field>')` at
//! mount — invisible to tsc (the type declares `$signals`), caught only by
//! web-QA. The view-mount path was already correct via the runtime
//! `buildWritableProxy`; this locks the parity for the compiler-emitted
//! component proxy.

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

fn component_opts(setup: Option<&str>) -> CompileOpts {
    CompileOpts {
        setup_url: setup.map(|s| s.to_string()),
        mode: CompileMode::Component,
        source_path: None,
        styles: None,
        component_meta: Some(ComponentMeta {
            name: "sui-test-dock".into(),
            shadow: true,
        }),
        emit_dts: false,
        emit_ssr: false,
    }
}

#[test]
fn component_mount_proxy_exposes_signals_accessor() {
    let src = r#"<div>{x()}</div>"#;
    let out = compile(src, component_opts(Some("./sui-test-dock.controller.ts")))
        .expect("compile component + paired controller");

    // The __sp get-trap must short-circuit `$signals` to the scope target so
    // `state.$signals.x` resolves to the raw signal — matching the runtime
    // writableProxy used on the view-mount + on-handler paths.
    assert!(
        out.js.contains(r#"if (k === "$signals") return t;"#),
        "component-mount __sp must expose the $signals accessor, got:\n{}",
        out.js
    );

    // And mount must still be wired with __sp (the proxy carrying $signals).
    assert!(
        out.js.contains("__lc.mount(__sp, host)"),
        "expected mount(__sp, host) wiring, got:\n{}",
        out.js
    );
}
