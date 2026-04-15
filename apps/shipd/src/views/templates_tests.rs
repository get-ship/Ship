use super::*;
use serde::Serialize;

#[derive(Serialize)]
struct EmptyCtx {
    icon: &'static str,
    title: &'static str,
    body: &'static str,
}

#[test]
fn env_initializes_and_registers_every_template() {
    let e = env();
    for (name, _) in TEMPLATES {
        assert!(
            e.get_template(name).is_ok(),
            "template '{name}' must be registered"
        );
    }
}

#[test]
fn render_empty_partial_produces_ship_empty_state() {
    let html = render(
        "_empty.html.j2",
        &EmptyCtx {
            icon: "inbox",
            title: "Nothing here",
            body: "Create one to get started.",
        },
    )
    .expect("render must succeed");
    assert!(html.contains("ship-empty-state"));
    assert!(html.contains("Nothing here"));
}

#[test]
fn missing_template_yields_error() {
    #[derive(Serialize)]
    struct Empty {}
    let err = render("does-not-exist.html.j2", &Empty {}).unwrap_err();
    assert!(err.contains("not found"));
}

#[test]
fn ship_sdk_template_is_registered() {
    let e = env();
    assert!(
        e.get_template("_ship.html.j2").is_ok(),
        "_ship.html.j2 must be registered"
    );
}

#[test]
fn render_view_source_with_ship_items_nonexistent() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% set items = ship_items('nonexistent_xyz') %}{{ items | length }}"#;
    let result = render_view_source("test-items", source, &Ctx {});
    assert!(result.is_ok(), "must not crash: {:?}", result.err());
    assert_eq!(result.unwrap().trim(), "0");
}

#[test]
fn render_view_source_with_where_filter() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% set items = ship_items('nonexistent') | where('status', 'active') %}{{ items | length }}"#;
    let result = render_view_source("test-where", source, &Ctx {});
    assert!(result.is_ok(), "must not crash: {:?}", result.err());
    assert_eq!(result.unwrap().trim(), "0");
}

#[test]
fn render_view_source_with_take_filter() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% set items = ship_items('nonexistent') | take(5) %}{{ items | length }}"#;
    let result = render_view_source("test-take", source, &Ctx {});
    assert!(result.is_ok(), "must not crash: {:?}", result.err());
    assert_eq!(result.unwrap().trim(), "0");
}

#[test]
fn render_view_source_imports_ship_macros() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% from '_ship.html.j2' import head %}<html><head>{{ head('Test') }}</head></html>"#;
    let result = render_view_source("test-macros", source, &Ctx {});
    assert!(result.is_ok(), "must not crash: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<title>Test</title>"), "html: {html}");
    assert!(html.contains("view-tokens.css"), "html: {html}");
    assert!(html.contains("datastar.js"), "html: {html}");
}
