//! Corpus-level integration tests: compile each fixture .set file
//! end-to-end and assert structural invariants on the emitted JS.
//!
//! These act as drift detectors — when emit shape changes, these
//! tests fail loudly and explain why.

use sail_compiler::{CompileResult, compile_file};
use std::path::Path;

fn compile(name: &str) -> CompileResult {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    let opts = sail_compiler::CompileOpts::default();
    compile_file(&path, &opts).unwrap()
}

fn js(name: &str) -> String {
    compile(name).js
}

#[test]
fn counter_compiles() {
    let js = js("counter.set");

    // Custom-element registration with the kebab-cased tag.
    assert!(js.contains("defineComponent(\"sail-counter\""));

    // Reactive declarations.
    assert!(js.contains("const count = signal(0)"));
    assert!(js.contains("const doubled = derived(() => (count() * 2))"));

    // User functions preserved.
    assert!(js.contains("function inc()"));
    assert!(js.contains("function reset()"));

    // Two text interpolations → two data-st anchors.
    assert!(js.contains("data-st=\"0\""));
    assert!(js.contains("data-st=\"1\""));

    // Two buttons with on:click → two data-sa anchors.
    assert!(js.contains("data-sa=\"0\""));
    assert!(js.contains("data-sa=\"1\""));
    assert!(js.contains("on(_n, \"click\", inc)"));
    assert!(js.contains("on(_n, \"click\", reset)"));

    // Cleanup chain present.
    assert!(js.contains("for (const c of cleanups)"));
}

#[test]
fn counter_imports_only_what_is_used() {
    let js = js("counter.set");
    // The counter doesn't use bindAttr, bindValue, bindChecked.
    assert!(!js.contains("bindAttr"));
    assert!(!js.contains("bindValue"));
    assert!(!js.contains("bindChecked"));
    // But it does use these.
    assert!(js.contains("bindText"));
    assert!(js.contains("defineComponent"));
    assert!(js.contains("signal"));
    assert!(js.contains("derived"));
    assert!(js.contains(", on,") || js.contains(" on }") || js.contains(" on,"));
}

#[test]
fn typeahead_compiles() {
    let js = js("typeahead.set");

    // Tag.
    assert!(js.contains("defineComponent(\"sail-typeahead\""));

    // Two props with kebab-case observed-attrs.
    assert!(js.contains("\"placeholder\""));
    assert!(js.contains("\"min-chars\""));
    assert!(js.contains("camel: \"minChars\""));

    // Number decoder for minChars.
    assert!(js.contains("Number(r)"));

    // Prop signals.
    assert!(js.contains("propSignals.placeholder = signal("));
    assert!(js.contains("'Search…'"));
    assert!(js.contains("propSignals.minChars = signal("));

    // Two-way binding on the input.
    assert!(js.contains("bindValue(_n, query)"));

    // Interpolated attribute (placeholder={placeholder}).
    assert!(js.contains("bindAttr"));

    // Debounce modifier.
    assert!(js.contains("{ kind: \"debounce\", arg: \"200\" }"));
}

#[test]
fn styled_compiles_with_scoped_css_and_tokens() {
    let r = compile("styled.set");
    let js = r.js;

    // STYLE constant present, scoped, with token rewrite.
    assert!(js.contains("const STYLE ="));
    assert!(js.contains("[data-component=\"sail-styled\"] .card"));
    assert!(js.contains("[data-component=\"sail-styled\"] .body"));
    // `:host` rewrites to the bare prefix.
    assert!(js.contains("[data-component=\"sail-styled\"]"));
    assert!(!js.contains(":host"));
    // Token interpolation rewrite.
    assert!(js.contains("var(--ship-space-3)"));
    assert!(js.contains("var(--ship-color-surface)"));
    assert!(js.contains("var(--ship-color-fg)"));
    // Style is injected at module load.
    assert!(js.contains("document.head.querySelector('style[data-sail=\"sail-styled\"]')"));

    // Control flow is wired.
    assert!(js.contains("bindIf"));
}

#[test]
fn ssr_dts_schema_all_emit_for_typeahead() {
    let r = compile("typeahead.set");
    // SSR wraps in host element with anchors preserved.
    assert!(r.ssr.starts_with("<sail-typeahead data-component=\"sail-typeahead\">"));
    assert!(r.ssr.contains("data-sa=\"0\""));
    assert!(r.ssr.ends_with("</sail-typeahead>"));

    // .d.ts surfaces props and tag map.
    assert!(r.dts.contains("export interface SailTypeahead extends HTMLElement"));
    assert!(r.dts.contains("placeholder: string;"));
    assert!(r.dts.contains("minChars: number;"));
    assert!(r.dts.contains("\"sail-typeahead\": SailTypeahead"));

    // schema.json round-trips.
    let v: serde_json::Value = serde_json::from_str(&r.schema).unwrap();
    assert_eq!(v["tag"], "sail-typeahead");
    assert_eq!(v["props"]["placeholder"]["type"], "string");
    assert_eq!(v["props"]["minChars"]["type"], "number");
}

#[test]
fn emitted_js_is_valid_module_shape() {
    for name in ["counter.set", "typeahead.set"] {
        let js = js(name);
        assert!(js.starts_with("// Auto-generated"), "{name} missing header");
        assert!(js.contains("from '@sail/runtime'"), "{name} missing import");
        assert!(js.trim_end().ends_with("})"), "{name} should end with closing paren");
    }
}
