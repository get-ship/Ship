//! Integration tests for `<form mutation="ship.X.Y">` — parse, render,
//! binding manifest, and validation error paths.

use sui_compiler::bindings;
use sui_compiler::parse::parse;
use sui_compiler::render::{RenderOpts, render_default as render};
use sui_compiler::{CompileMode, CompileOpts, compile};

fn render_source(src: &str) -> String {
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "myview".into() };
    render(&program, &opts).expect("render").html
}

fn compile_view(source: &str) -> sui_compiler::CompileError {
    compile(
        source,
        CompileOpts {
            setup_url: None,
            mode: CompileMode::View,
            source_path: None,
            styles: None,
            component_meta: None,
            emit_dts: false,
        },
    )
    .expect_err("expected error")
}

/// Parser produces `AttrInstr::Mutation` and render emits the marker + PE attrs.
#[test]
fn form_mutation_renders_marker_and_pe_attrs() {
    let src = r#"<form mutation="ship.agents.create"><button type="submit">Go</button></form>"#;
    let html = render_source(src);
    assert!(
        html.contains(r#"mutation="b0""#),
        "expected mutation marker; got: {html:?}"
    );
    assert!(
        html.contains(r#"action="/_sui/mutation/myview?form=b0""#),
        "expected PE action attr; got: {html:?}"
    );
    assert!(
        html.contains(r#"method="post""#),
        "expected method=post; got: {html:?}"
    );
}

/// A static `action=` on the author's form is preserved — PE inject is skipped.
#[test]
fn form_mutation_respects_existing_action() {
    let src = r#"<form action="/custom" mutation="ship.agents.create"></form>"#;
    let html = render_source(src);
    assert!(
        html.contains(r#"action="/custom""#),
        "author action must be preserved; got: {html:?}"
    );
    assert!(
        !html.contains("/_sui/mutation"),
        "PE inject must not appear when action is explicit; got: {html:?}"
    );
}

/// The binding manifest contains a `mutation` entry for the form.
#[test]
fn form_mutation_appears_in_binding_manifest() {
    let src = r#"<form mutation="ship.agents.create"></form>"#;
    let program = parse(src).expect("parse");
    let reg = bindings::build(&program);
    let binding = reg.get(0).expect("expected at least one binding");
    assert_eq!(binding.key, "b0");
    assert_eq!(binding.kind.as_marker_str(), "mutation");
    assert_eq!(binding.expr, "ship.agents.create");
}

/// Compiler rejects a `mutation=` value that is not a `ship.X.Y` chain.
#[test]
fn invalid_mutation_expr_is_rejected() {
    let src = r#"<form mutation="agents.create"></form>"#;
    let err = compile_view(src);
    assert!(
        err.message.contains("mutation="),
        "error should mention mutation=; got: {}",
        err.message
    );
}
