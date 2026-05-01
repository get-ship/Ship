//! Integration tests for the SSR render walker.
//!
//! D-1 nailed marker shapes; D-2 evaluates the renderable subset
//! against the page's initial scope, inlines the rendered values
//! alongside markers, and emits the `__set_init__` JSON island after
//! the body. These tests pin the rendered HTML so any regression in
//! marker shape, evaluator output, or island serialization is loud.

use sui_compiler::bindings;
use sui_compiler::parse::parse;
use sui_compiler::render::{RenderOpts, RenderOutput, render_default as render};

fn render_source(src: &str) -> String {
    render_full(src).html
}

fn render_full(src: &str) -> RenderOutput {
    let program = parse(src).expect("parse");
    let opts = RenderOpts { view_name: "test".into() };
    render(&program, &opts).expect("render")
}

const ISLAND_OPEN: &str = "<script type=\"application/json\" id=\"__set_init__\">";

#[test]
fn static_html_renders_verbatim() {
    let html = render_source(r#"<div class="x">hello</div>"#);
    assert!(
        html.starts_with(r#"<div class="x">hello</div>"#),
        "static body must come first; got: {html:?}"
    );
    // No state/data decls -> island is the empty object.
    assert!(html.ends_with("{}</script>"), "expected empty island; got: {html:?}");
}

#[test]
fn dynamic_text_emits_comment_marker() {
    // Unbound identifier -> evaluator fails -> marker-only fallback.
    let html = render_source("<p>{count}</p>");
    assert!(
        html.contains("<p><!--set:text:b0--><!--/set--></p>"),
        "expected marker-only dynamic-text shape; got: {html:?}"
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
        html.contains(r#"data-set-bind:value="b0""#),
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
    // Unbound condition → evaluator errors → D-1 placeholder fallback.
    let src = "{#if missing}<p>x</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-if="b0"></template>"#),
        "expected if placeholder; got: {html:?}"
    );
    assert!(
        !html.contains("<p>x</p>"),
        "body must not be recursed when condition is unevaluable; got: {html:?}"
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
    // Unbound discriminant → evaluator errors → D-1 placeholder fallback.
    let src = "{#match missing}{:case 'a'}A{:default}D{/match}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-match="b0"></template>"#),
        "expected match placeholder; got: {html:?}"
    );
    assert!(
        !html.contains(">A<") && !html.contains(">D<"),
        "no body must render when discriminant is unevaluable; got: {html:?}"
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
            .join("views/agents/view.sui"),
    )
    .expect("read agents view");
    let program = parse(&src).expect("agents view parses");
    let opts = RenderOpts { view_name: "agents".into() };
    let out = render(&program, &opts).expect("agents view renders");
    assert!(!out.html.is_empty(), "agents view must produce non-empty HTML");
    // Sanity: the static container should be present.
    assert!(out.html.contains("ship-view"), "container class missing: {}", &out.html[..200.min(out.html.len())]);
}

// ---------- Phase D-2: scope evaluation + JSON island ----------

#[test]
fn state_value_inlines_into_text() {
    let src = "<state name=\"n\" value=\"42\" />\n<p>{n}</p>";
    let html = render_source(src);
    assert!(
        html.contains("<p><!--set:text:b0-->42<!--/set--></p>"),
        "expected inlined value between markers; got: {html:?}"
    );
}

#[test]
fn state_string_value_html_escapes() {
    // Tests text-content escaping: `&`, `<`, `>` -> entities; quotes
    // pass through PCDATA. Single-quoted inner string keeps the HTML
    // attribute parser happy.
    let src = "<state name=\"s\" value=\"'<a>&x'\" />\n<p>{s}</p>";
    let html = render_source(src);
    assert!(
        html.contains("<p><!--set:text:b0-->&lt;a&gt;&amp;x<!--/set--></p>"),
        "expected escaped text content; got: {html:?}"
    );
}

#[test]
fn data_initial_envelope_is_loading() {
    let src = "<data name=\"agents\" source=\"ship.agents.summaries()\" live />";
    let html = render_source(src);
    assert!(
        html.contains(r#""agents":{"kind":"loading"}"#),
        "expected data envelope in island; got: {html:?}"
    );
}

#[test]
fn unevaluable_expr_emits_marker_only() {
    // `ship` not in scope -> Unbound -> marker-only fallback.
    let html = render_source("<p>{ship.foo()}</p>");
    assert!(
        html.contains("<p><!--set:text:b0--><!--/set--></p>"),
        "expected marker-only fallback; got: {html:?}"
    );
}

#[test]
fn attribute_interp_inlines_value() {
    let src = "<state name=\"id\" value=\"'42'\" />\n<a href={'/u/' + id}>x</a>";
    let html = render_source(src);
    assert!(
        html.contains(r#"<a href="/u/42" data-set-attr:href="b0">"#),
        "expected inlined href and marker; got: {html:?}"
    );
}

#[test]
fn class_directive_renders_initial() {
    let src = "<div class:active={true}>x</div>";
    let html = render_source(src);
    assert!(
        html.contains(r#"<div class="active" data-set-class:active="b0">"#),
        "expected merged class plus marker; got: {html:?}"
    );
}

#[test]
fn bind_value_renders_initial() {
    let src = "<state name=\"query\" value=\"'hi'\" />\n<input bind:value={query} />";
    let html = render_source(src);
    assert!(
        html.contains(r#"<input value="hi" data-set-bind:value="b0">"#),
        "expected initial value plus marker; got: {html:?}"
    );
}

#[test]
fn json_island_present_after_body() {
    let src = "<state name=\"n\" value=\"3\" />\n<p>{n}</p>";
    let html = render_source(src);
    let count = html.matches(ISLAND_OPEN).count();
    assert_eq!(count, 1, "expected exactly one island; got {count}: {html:?}");
    let island_pos = html.find(ISLAND_OPEN).unwrap();
    let body_marker = html.find("<!--/set-->").expect("body marker present");
    assert!(
        body_marker < island_pos,
        "island must follow body content; got: {html:?}"
    );
}

#[test]
fn agents_view_island_carries_loading_envelope() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("views/agents/view.sui"),
    )
    .expect("read agents view");
    let program = parse(&src).expect("parse");
    let opts = RenderOpts { view_name: "agents".into() };
    let out = render(&program, &opts).expect("render");
    assert!(out.html.contains(ISLAND_OPEN), "island missing");
    assert!(
        out.html.contains(r#""agents":{"kind":"loading"}"#),
        "expected agents:loading in island"
    );
    let agents = out.init.get("agents").expect("init has agents key");
    assert_eq!(
        agents,
        &serde_json::json!({"kind": "loading"}),
        "init.agents shape mismatch"
    );
}

// ---------- Phase D-3: control-flow rendering ----------

#[test]
fn if_truthy_branch_renders_body() {
    let src = "<state name=\"show\" value=\"true\" />\n{#if show}<p>yes</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<div data-set-if="b0" style="display:contents"><p>yes</p></div>"#),
        "expected truthy branch body in active wrapper; got: {html:?}"
    );
}

#[test]
fn if_falsy_branch_skipped() {
    let src = "<state name=\"show\" value=\"false\" />\n{#if show}<p>yes</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-if="b0"></template>"#),
        "expected empty template for falsy branch with no else; got: {html:?}"
    );
    assert!(!html.contains("<p>yes</p>"));
}

#[test]
fn if_else_branch_renders_when_falsy() {
    let src = "<state name=\"x\" value=\"false\" />\n{#if x}<p>a</p>{:else}<p>b</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<div data-set-if="b0" style="display:contents"><p>b</p></div>"#),
        "expected else body; got: {html:?}"
    );
    assert!(!html.contains("<p>a</p>"));
}

#[test]
fn if_unevaluable_falls_back_to_placeholder() {
    // `route.params.id` is a string lookup on the empty `route.params`
    // placeholder, so the strict-equality test against `'42'` evaluates
    // cleanly to false. To get a genuinely unevaluable shape we have
    // to reference an unbound name — same end-state semantics.
    let src = "{#if missing.id === '42'}<p>x</p>{/if}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-if="b0"></template>"#),
        "expected empty placeholder; got: {html:?}"
    );
    assert!(!html.contains("<p>x</p>"));
}

#[test]
fn for_renders_each_iteration() {
    let src = "<state name=\"items\" value=\"[1,2,3]\" />\n{#for n of items}<li>{n}</li>{/for}";
    let html = render_source(src);
    // Body has one dynamic-text key allocated once and shared across iterations.
    assert!(
        html.contains(r#"<div data-set-for="b0" style="display:contents">"#),
        "expected for active wrapper open; got: {html:?}"
    );
    assert!(html.contains("<li><!--set:text:b1-->1<!--/set--></li>"));
    assert!(html.contains("<li><!--set:text:b1-->2<!--/set--></li>"));
    assert!(html.contains("<li><!--set:text:b1-->3<!--/set--></li>"));
}

#[test]
fn for_emits_data_set_key() {
    let src = "<state name=\"items\" value=\"[1,2,3]\" />\n{#for n of items key={n}}<li>{n}</li>{/for}";
    let html = render_source(src);
    assert!(html.contains(r#"<li data-set-key="1">"#), "missing key=1: {html:?}");
    assert!(html.contains(r#"<li data-set-key="2">"#), "missing key=2: {html:?}");
    assert!(html.contains(r#"<li data-set-key="3">"#), "missing key=3: {html:?}");
}

#[test]
fn for_iterations_share_inner_keys() {
    let src = "<state name=\"items\" value=\"[1,2]\" />\n{#for n of items}<li>{n}</li>{/for}";
    let html = render_source(src);
    let occurrences = html.matches("set:text:b1").count();
    // One open marker per iteration; `b1` reused across all iterations.
    assert_eq!(
        occurrences, 2,
        "expected the same b1 key reused across iterations; got {occurrences}: {html:?}"
    );
}

#[test]
fn for_empty_renders_else_body() {
    let src =
        "<state name=\"items\" value=\"[]\" />\n{#for n of items}<li>{n}</li>{:else}<p>none</p>{/for}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<div data-set-for="b0" style="display:contents"><p>none</p></div>"#),
        "expected else body inside for active wrapper; got: {html:?}"
    );
}

#[test]
fn for_unevaluable_iterable_falls_back() {
    let src = "{#for n of missing}<li>{n}</li>{/for}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-for="b0"></template>"#),
        "expected empty placeholder for unevaluable iterable; got: {html:?}"
    );
    assert!(!html.contains("<li>"));
}

#[test]
fn match_evaluates_discriminant() {
    let src = "<state name=\"kind\" value=\"'a'\" />\n{#match kind}{:case 'a'}<p>A</p>{:case 'b'}<p>B</p>{/match}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<div data-set-match="b0" style="display:contents"><p>A</p></div>"#),
        "expected case A rendered; got: {html:?}"
    );
    assert!(!html.contains("<p>B</p>"));
}

#[test]
fn match_unevaluable_falls_back() {
    let src = "{#match missing}{:case 'a'}<p>A</p>{:default}<p>D</p>{/match}";
    let html = render_source(src);
    assert!(
        html.contains(r#"<template data-set-match="b0"></template>"#),
        "expected empty placeholder for unevaluable discriminant; got: {html:?}"
    );
    assert!(!html.contains("<p>A</p>"));
    assert!(!html.contains("<p>D</p>"));
}

#[test]
fn nested_control_flow_round_trip() {
    let src = "<state name=\"x\" value=\"true\" />\n<state name=\"xs\" value=\"[1,2]\" />\n{#if x}{#for n of xs}<li>{n}</li>{/for}{/if}";
    let html = render_source(src);
    // Outer if takes b0; inner for takes b1; inner dynamic text takes b2.
    assert!(html.contains(r#"<div data-set-if="b0" style="display:contents">"#));
    assert!(html.contains(r#"<div data-set-for="b1" style="display:contents">"#));
    assert!(html.contains("<li><!--set:text:b2-->1<!--/set--></li>"));
    assert!(html.contains("<li><!--set:text:b2-->2<!--/set--></li>"));
    // for body closes inside if body which closes outside.
    let if_open = html.find("data-set-if=\"b0\"").unwrap();
    let for_open = html.find("data-set-for=\"b1\"").unwrap();
    assert!(if_open < for_open, "nesting order wrong: {html:?}");
}

#[test]
fn agents_view_round_trip_with_d3() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("views/agents/view.sui"),
    )
    .expect("read agents view");
    let program = parse(&src).expect("parse agents view");
    let opts = RenderOpts { view_name: "agents".into() };
    let out = render(&program, &opts).expect("render agents view");
    // Outer `{#if agents().kind === 'ok'}` evaluates against the loading
    // envelope at SSR time, so it falls into the placeholder branch.
    assert!(
        out.html.contains("data-set-if="),
        "expected at least one data-set-if marker: {}",
        &out.html[..200.min(out.html.len())]
    );
    // The agents view contains a `{#for}` somewhere; its outer guard
    // (the `{#if agents().kind === 'ok'}`) is falsy, so the for marker
    // never reaches the output. We only require that the walker did
    // not panic and produced a reachable shape — assert the `__set_init__`
    // island is still emitted.
    assert!(out.html.contains("__set_init__"), "init island missing");
    assert_eq!(
        out.init.get("agents"),
        Some(&serde_json::json!({"kind": "loading"})),
    );
}

// ---------- Phase E-prep: walker / registry parity ----------

/// For every `data-set-<kind>="b<N>"` marker (and every
/// `<!--set:text:b<N>-->`) the walker emits, look up `bindings[N]` and
/// confirm the registry entry's kind matches the marker family.
#[test]
fn walker_keys_match_registry() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("views/agents/view.sui"),
    )
    .expect("read agents view");
    let program = parse(&src).expect("parse");
    let registry = bindings::build(&program);
    let opts = RenderOpts { view_name: "agents".into() };
    let out = render(&program, &opts).expect("render");

    // Pull `(kind_str, key)` pairs out of the rendered HTML in order.
    let pairs = extract_marker_pairs(&out.html);
    assert!(
        !pairs.is_empty(),
        "agents view should emit at least one marker; got: {}",
        &out.html[..400.min(out.html.len())]
    );

    for (marker_kind, key) in pairs {
        let n: usize = key
            .strip_prefix('b')
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("malformed key {key}"));
        let entry = registry
            .get(n)
            .unwrap_or_else(|| panic!("registry missing index {n} (key {key})"));
        assert_eq!(
            entry.key, key,
            "registry[{n}].key={} but marker key={key}",
            entry.key
        );
        assert_eq!(
            entry.kind.as_marker_str(),
            marker_kind,
            "registry[{n}].kind={} but marker kind={marker_kind}",
            entry.kind.as_marker_str()
        );
    }
}

/// Find every `data-set-<kind>="bN"` attribute and `<!--set:text:bN-->`
/// comment marker in the rendered HTML, returning `(kind_str, key)`.
fn extract_marker_pairs(html: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let bytes = html.as_bytes();
    let mut i = 0;

    // data-set-<kind>="bN"
    while i < bytes.len() {
        if let Some(rest) = html[i..].strip_prefix("data-set-") {
            // Read kind up to '='.
            let eq_pos = rest.find('=');
            let quote_pos = rest.find('"');
            if let (Some(eq), Some(qs)) = (eq_pos, quote_pos) {
                if qs == eq + 1 {
                    let kind = &rest[..eq];
                    if let Some(qe) = rest[qs + 1..].find('"') {
                        let key = &rest[qs + 1..qs + 1 + qe];
                        if key.starts_with('b') {
                            out.push((kind.to_string(), key.to_string()));
                        }
                        i += "data-set-".len() + qs + 1 + qe + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }

    // <!--set:text:bN-->
    let mut j = 0;
    while let Some(start) = html[j..].find("<!--set:text:") {
        let abs = j + start + "<!--set:text:".len();
        if let Some(end_off) = html[abs..].find("-->") {
            let key = &html[abs..abs + end_off];
            if key.starts_with('b') {
                out.push(("text".to_string(), key.to_string()));
            }
            j = abs + end_off + 3;
        } else {
            break;
        }
    }

    out
}

#[test]
fn walker_keys_match_registry_for_counter_fixture() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/counter.sui"),
    )
    .expect("read counter fixture");
    let program = parse(&src).expect("parse");
    let registry = bindings::build(&program);
    let opts = RenderOpts { view_name: "counter".into() };
    let out = render(&program, &opts).expect("render");

    for (marker_kind, key) in extract_marker_pairs(&out.html) {
        let n: usize = key.strip_prefix('b').unwrap().parse().unwrap();
        let entry = registry.get(n).expect("registry has entry");
        assert_eq!(entry.kind.as_marker_str(), marker_kind);
    }
}
