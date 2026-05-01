//! Fixture-based tests for the `.sui.d.ts` declaration emitter.
//!
//! Each test compiles a small `.sui` snippet with `emit_dts: true` and
//! asserts the resulting d.ts contains the expected type lines.

use sui_compiler::{compile, CompileMode, CompileOpts};

fn compile_dts(source: &str) -> String {
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::View,
        source_path: None,
        styles: None,
        component_meta: None,
        emit_dts: true,
    };
    compile(source, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

fn compile_component_dts(source: &str) -> String {
    use sui_compiler::ComponentMeta;
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        styles: None,
        component_meta: Some(ComponentMeta { name: "test-el".into(), shadow: false }),
        emit_dts: true,
    };
    compile(source, opts)
        .expect("compile")
        .dts
        .expect("dts emitted")
}

#[test]
fn state_integer_literal_emits_number() {
    let dts = compile_dts(r#"<state name="count" value="0" />"#);
    assert!(dts.contains("count: Signal<number>"), "got: {dts}");
    assert!(dts.contains("import type { Signal }"), "got: {dts}");
    assert!(!dts.contains("LiveState"), "got: {dts}");
}

#[test]
fn state_string_literal_emits_string() {
    let dts = compile_dts(r#"<state name="label" value="''" />"#);
    assert!(dts.contains("label: Signal<string>"), "got: {dts}");
}

#[test]
fn state_boolean_emits_boolean() {
    let dts = compile_dts(r#"<state name="open" value="false" />"#);
    assert!(dts.contains("open: Signal<boolean>"), "got: {dts}");
}

#[test]
fn data_live_emits_live_state() {
    let dts = compile_dts(r#"<data name="agents" source="ship.agents.list()" live />"#);
    assert!(dts.contains("agents: Signal<LiveState<unknown>>"), "got: {dts}");
    assert!(dts.contains("import type { Signal, LiveState }"), "got: {dts}");
}

#[test]
fn data_once_emits_signal_unknown() {
    let dts = compile_dts(r#"<data name="config" source="ship.config.get()" />"#);
    assert!(dts.contains("config: Signal<unknown>"), "got: {dts}");
    assert!(!dts.contains("LiveState"), "got: {dts}");
}

#[test]
fn prop_union_emits_union_type() {
    let dts = compile_component_dts(
        r#"<prop name="tone" values="primary|neutral" default="primary" /><span>x</span>"#,
    );
    assert!(
        dts.contains("tone: Signal<'primary' | 'neutral'>"),
        "got: {dts}"
    );
}

#[test]
fn prop_bool_emits_boolean() {
    let dts = compile_component_dts(
        r#"<prop name="loading" type="bool" default="false" /><span>x</span>"#,
    );
    assert!(dts.contains("loading: Signal<boolean>"), "got: {dts}");
}

#[test]
fn use_hook_with_as_emits_return_type() {
    let dts = compile_dts(
        r#"<data name="items" source="ship.list()" />
<use-filter-list as="filter"><span>x</span></use-filter-list>"#,
    );
    assert!(
        dts.contains("filter: ReturnType<typeof useFilterList>"),
        "got: {dts}"
    );
}

#[test]
fn emit_dts_false_returns_none() {
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::View,
        source_path: None,
        styles: None,
        component_meta: None,
        emit_dts: false,
    };
    let out = compile(r#"<state name="x" value="0" />"#, opts).expect("compile");
    assert!(out.dts.is_none(), "expected no dts when emit_dts=false");
}

#[test]
fn no_declarations_emits_empty_scope() {
    let dts = compile_dts("<p>Hello world</p>");
    assert!(dts.contains("export interface Scope {}"), "got: {dts}");
}

#[test]
fn multiple_declarations_all_appear() {
    let src = r#"
<data name="overview" source="ship.git.overview()" live />
<state name="tab" value="'changes'" />
"#;
    let dts = compile_dts(src);
    assert!(dts.contains("overview: Signal<LiveState<unknown>>"), "got: {dts}");
    assert!(dts.contains("tab: Signal<string>"), "got: {dts}");
    assert!(dts.contains("import type { Signal, LiveState }"), "got: {dts}");
}
