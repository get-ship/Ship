//! Diagnostic-coverage tests. The compiler must reject the same
//! constructs the interpreter rejects (statements, assignments,
//! arrows where not allowed) with a message and span pointing at
//! the user's source.

use sui_compiler::{compile, CompileMode, CompileOpts};

fn compile_view(source: &str) -> sui_compiler::CompileError {
    compile(
        source,
        CompileOpts { setup_url: None, mode: CompileMode::View, source_path: None, styles: None, component_meta: None },
    )
    .expect_err("expected error")
}

#[test]
fn assignment_in_state_value_is_rejected() {
    let src = r#"<state name="x" value={a = 1} />"#;
    let err = compile_view(src);
    assert!(err.message.contains("assignments"), "msg: {}", err.message);
    assert!(err.category.is_some());
}

#[test]
fn arrow_in_attr_expression_is_rejected() {
    let src = r#"<button disabled={() => true}></button>"#;
    let err = compile_view(src);
    assert!(err.message.contains("arrow"), "msg: {}", err.message);
}

#[test]
fn statement_in_text_expression_is_rejected() {
    let src = "<p>{a; b}</p>";
    let err = compile_view(src);
    assert!(err.message.contains("statements"), "msg: {}", err.message);
}

#[test]
fn arrow_in_on_handler_is_allowed() {
    // sanity: on:click={() => fn()} must not trigger the arrow check.
    let src = r#"<button on:click={() => x()}></button>"#;
    compile(
        src,
        CompileOpts { setup_url: None, mode: CompileMode::View, source_path: None, styles: None, component_meta: None },
    )
    .expect("on: handlers allow arrows");
}

#[test]
fn missing_required_data_attr_reports_directive() {
    let src = r#"<data name="x" />"#;
    let err = compile_view(src);
    assert!(err.message.contains("source"), "msg: {}", err.message);
    assert_eq!(err.category.as_deref(), Some("<data>"));
}

#[test]
fn assignment_diagnostic_has_a_span() {
    let src = r#"<state name="x" value={a = 1} />"#;
    let err = compile_view(src);
    assert!(err.span.is_some(), "should carry a span");
    let rendered = err.render(src);
    assert!(rendered.contains("line 1"), "rendered: {rendered}");
}

#[test]
fn unknown_modifier_is_rejected() {
    let src = r#"<button on:click|wat={() => x()}></button>"#;
    let err = compile_view(src);
    assert!(err.message.contains("modifier"), "msg: {}", err.message);
}

#[test]
fn missing_close_tag_is_rejected() {
    let src = r#"<div>"#;
    let err = compile_view(src);
    assert!(err.message.contains("</div>") || err.message.contains("unterminated"));
}

#[test]
fn unknown_prop_type_is_rejected() {
    let src = r#"<prop name="x" type="custom" default="" />"#;
    let err = compile_view(src);
    assert!(err.message.contains("unknown type"), "msg: {}", err.message);
}
