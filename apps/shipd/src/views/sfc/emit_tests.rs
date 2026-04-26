//! Tests for `emit.rs` — the JS-bundle emitter that wraps a `.ship` SFC
//! into a `customElements.define(...)` registration.

use super::emit::{emit, EmitInputs};
use super::script::{PropDecl, ScriptSchema};

fn schema(props: Vec<(&str, Option<&str>)>, signals: Vec<&str>) -> ScriptSchema {
    ScriptSchema {
        props: props
            .into_iter()
            .map(|(n, d)| PropDecl { name: n.to_string(), default: d.map(|s| s.to_string()) })
            .collect(),
        signals: signals.into_iter().map(|s| s.to_string()).collect(),
        on_connect_calls: 0,
        on_disconnect_calls: 0,
    }
}

#[test]
fn emit_contains_tag_and_define() {
    let s = schema(vec![], vec![]);
    let out = emit(&EmitInputs {
        tag: "my-comp",
        schema: &s,
        user_script: "// nothing",
        rendered_html: "<div></div>",
        scoped_css: "",
    });
    assert!(out.contains("const TAG = \"my-comp\""));
    assert!(out.contains("customElements.define(TAG, ShipComponent)"));
}

#[test]
fn emit_includes_user_script_verbatim() {
    let s = schema(vec![], vec!["x"]);
    let out = emit(&EmitInputs {
        tag: "t",
        schema: &s,
        user_script: "const $x = signal(42)",
        rendered_html: "<div></div>",
        scoped_css: "",
    });
    assert!(out.contains("const $x = signal(42)"));
    assert!(out.contains("SIGNAL_NAMES = [\"x\"]"));
}

#[test]
fn emit_escapes_backticks_and_dollars_in_template() {
    let s = schema(vec![], vec![]);
    let out = emit(&EmitInputs {
        tag: "t",
        schema: &s,
        user_script: "",
        rendered_html: "<div>`back$tick</div>",
        scoped_css: "",
    });
    assert!(out.contains("\\`back\\$tick"));
}

#[test]
fn emit_bridges_signals_to_datastar() {
    let s = schema(vec![], vec!["q"]);
    let out = emit(&EmitInputs {
        tag: "ta",
        schema: &s,
        user_script: "const $q = signal('')",
        rendered_html: "<div></div>",
        scoped_css: "",
    });
    assert!(
        out.contains("setAttribute(\"data-signals:\" + key"),
        "wrapper must write data-signals:<key> on host so Datastar ingests it"
    );
    assert!(
        out.contains("datastar-signal-patch"),
        "wrapper must subscribe to Datastar's signal-patch event for .get() / effect()"
    );
    assert!(
        out.contains("removeEventListener(\"datastar-signal-patch\""),
        "wrapper must clean up the listener on disconnect"
    );
}

#[test]
fn emit_dispatches_composed_events() {
    // emit() must set composed: true so events cross shadow-DOM boundaries
    // when the component is consumed inside a shadow root.
    let s = schema(vec![], vec![]);
    let out = emit(&EmitInputs {
        tag: "t",
        schema: &s,
        user_script: "",
        rendered_html: "<div></div>",
        scoped_css: "",
    });
    assert!(
        out.contains("composed: true"),
        "emit() must include composed: true on the CustomEvent"
    );
}

#[test]
fn prop_decoders_use_kebab_attr() {
    let s = schema(vec![("fetchUrl", None), ("minChars", Some("1"))], vec![]);
    let out = emit(&EmitInputs {
        tag: "t",
        schema: &s,
        user_script: "",
        rendered_html: "",
        scoped_css: "",
    });
    assert!(out.contains("this.getAttribute(\"fetch-url\")"));
    assert!(out.contains("this.getAttribute(\"min-chars\")"));
    assert!(out.contains("props[\"minChars\"]"));
}
