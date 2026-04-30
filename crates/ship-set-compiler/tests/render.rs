//! Integration tests for the SSR render walker (Phase D-1).
//!
//! Asserts the marker shapes against the marker-contract ADR. These
//! tests intentionally pin the *exact* HTML output for each construct
//! so a regression in marker shape is loud.

use ship_set_compiler::parse::parse;
use ship_set_compiler::render::{RenderOpts, render};

fn render_source(src: &str) -> String {
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "test".into() };
    render(&program, &opts).expect("render").html
}

#[test]
fn static_html_renders_verbatim() {
    let html = render_source(r#"<div class="x">hello</div>"#);
    assert_eq!(html, r#"<div class="x">hello</div>"#);
}

#[test]
fn dynamic_text_emits_comment_marker() {
    let src = "<state name=\"count\" value=\"0\" />\n<p>{count}</p>\n";
    let html = render_source(src);
    assert!(
        html.contains("<p><!--set:text:b0--><!--/set--></p>"),
        "expected dynamic-text marker; got: {html:?}"
    );
}

#[test]
fn event_handler_emits_data_set_on() {
    let src = "<state name=\"x\" value=\"0\" />\n<button on:click={() => x()}></button>";
    let html = render_source(src);
    assert!(
        html.contains(r#"<button data-set-on:click="b0">"#),
        "expected on:click marker; got: {html:?}"
    );
    assert!(html.contains("</button>"));
}

#[test]
fn bind_value_emits_data_set_bind() {
    // <input> is a void element — no closing tag.
    let src = "<state name=\"q\" value=\"''\" />\n<input bind:value={q} />";
    let html = render_source(src);
    assert!(
        html.contains(r#"<input data-set-bind:value="b0">"#),
        "expected bind:value marker; got: {html:?}"
    );
    assert!(
        !html.contains("</input>"),
        "void element must not emit a closing tag; got: {html:?}"
    );
}

#[test]
fn bind_checked_emits_data_set_bind() {
    let src = "<state name=\"on\" value=\"false\" />\n<input type=\"checkbox\" bind:checked={on} />";
    let html = render_source(src);
    assert!(
        html.contains(r#"data-set-bind:checked="b0""#),
        "expected bind:checked marker; got: {html:?}"
    );
}

#[test]
fn class_directive_emits_data_set_class() {
    let src = "<state name=\"on\" value=\"false\" />\n<p class:active={on}>x</p>";
    let html = render_source(src);
    assert!(
        html.contains(r#"data-set-class:active="b0""#),
        "expected class marker; got: {html:?}"
    );
}

#[test]
fn expr_attribute_emits_data_set_attr() {
    let src = "<state name=\"u\" value=\"'/'\" />\n<a href={u}>x</a>";
    let html = render_source(src);
    assert!(
        html.contains(r#"data-set-attr:href="b0""#),
        "expected attr:href marker; got: {html:?}"
    );
}

#[test]
fn if_emits_template_placeholder() {
    let src = "<state name=\"on\" value=\"true\" />\n{#if on}<p>x</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-if="b0"></template>"#),
        "expected if placeholder; got: {html:?}"
    );
    // Body must NOT be rendered in D-1.
    assert!(
        !html.contains("<p>x</p>"),
        "body must not be recursed in D-1; got: {html:?}"
    );
}

#[test]
fn for_emits_template_placeholder() {
    let src = "<state name=\"xs\" value=\"[]\" />\n{#for x of xs}<li>x</li>{/for}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-for="b0"></template>"#),
        "expected for placeholder; got: {html:?}"
    );
}

#[test]
fn match_emits_template_placeholder() {
    let src = "<state name=\"v\" value=\"'a'\" />\n{#match v}{:case 'a'}A{:default}D{/match}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-match="b0"></template>"#),
        "expected match placeholder; got: {html:?}"
    );
}

#[test]
fn static_attrs_are_html_escaped() {
    // The `&` in a static attribute value must become `&amp;`.
    let html = render_source(r#"<a title="a & b">x</a>"#);
    assert!(
        html.contains(r#"<a title="a &amp; b">"#),
        "expected escaped attr; got: {html:?}"
    );
}

#[test]
fn state_and_data_decls_skip_in_html() {
    let src = "<state name=\"count\" value=\"0\" />\n<p>hi</p>";
    let html = render_source(src);
    // <state> declarations are lifted by the parser, never appear in
    // the template tree, so they cannot be rendered. Confirm.
    assert!(!html.contains("<state"), "state must not render: {html:?}");
    assert!(html.contains("<p>hi</p>"));
}

#[test]
fn keys_increment_in_source_order() {
    // Two dynamic interpolations should get b0 and b1 in order.
    let src = "<state name=\"a\" value=\"0\" />\n<state name=\"b\" value=\"0\" />\n<p>{a}-{b}</p>";
    let html = render_source(src);
    let i0 = html.find("set:text:b0").expect("first marker");
    let i1 = html.find("set:text:b1").expect("second marker");
    assert!(i0 < i1, "keys must increment in source order: {html:?}");
}

#[test]
fn agents_view_renders_without_panic() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("views/agents/view.html.set"),
    )
    .expect("read agents view");
    let program = parse(&src).expect("agents view parses");
    let opts = RenderOpts { view_name: "agents".into() };
    let out = render(&program, &opts).expect("agents view renders");
    assert!(!out.html.is_empty(), "agents view must produce non-empty HTML");
    // Sanity: the static container should be present.
    assert!(out.html.contains("ship-view"), "container class missing: {}", &out.html[..200.min(out.html.len())]);
}
