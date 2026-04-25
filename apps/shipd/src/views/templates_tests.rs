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

#[test]
fn section_macro_opens_ship_card_with_title() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% from '_ship.html.j2' import section, endsection %}{{ section('Recent jobs') }}body{{ endsection() }}"#;
    let html = render_view_source("test-section-basic", source, &Ctx {}).unwrap();
    assert!(html.contains(r#"<ship-card padding="none">"#), "html: {html}");
    assert!(html.contains(r#"slot="header""#), "html: {html}");
    assert!(html.contains("Recent jobs"), "html: {html}");
    assert!(html.contains("body"), "html: {html}");
    assert!(html.contains("</ship-card>"), "html: {html}");
}

#[test]
fn section_macro_renders_leading_icon_when_supplied() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% from '_ship.html.j2' import section, endsection %}{{ section('Agents', icon='users') }}{{ endsection() }}"#;
    let html = render_view_source("test-section-icon", source, &Ctx {}).unwrap();
    assert!(
        html.contains(r#"<ship-icon name="users""#),
        "leading icon must render: {html}"
    );
}

#[test]
fn section_macro_omits_icon_when_not_supplied() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% from '_ship.html.j2' import section, endsection %}{{ section('Plain') }}{{ endsection() }}"#;
    let html = render_view_source("test-section-no-icon", source, &Ctx {}).unwrap();
    assert!(!html.contains("<ship-icon"), "no icon should render: {html}");
}

#[test]
fn section_macro_emits_sse_link_for_view_href() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% from '_ship.html.j2' import section, endsection %}{{ section('Jobs', link_href='/api/v1/view/pipelines', link_label='View all') }}{{ endsection() }}"#;
    let html = render_view_source("test-section-sse-link", source, &Ctx {}).unwrap();
    assert!(html.contains("View all"), "label: {html}");
    assert!(
        html.contains("data-on:click.prevent=\"@get('/api/v1/view/pipelines')\""),
        "SSE handler must render for /api/v1/view/ hrefs: {html}"
    );
    assert!(
        html.contains(r#"href="/api/v1/view/pipelines""#),
        "href must render for fallback: {html}"
    );
    assert!(
        html.contains(r#"name="chevron-right""#),
        "chevron icon must render: {html}"
    );
}

#[test]
fn section_macro_plain_anchor_for_non_view_href() {
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% from '_ship.html.j2' import section, endsection %}{{ section('External', link_href='https://example.com/docs', link_label='Docs') }}{{ endsection() }}"#;
    let html = render_view_source("test-section-plain-link", source, &Ctx {}).unwrap();
    assert!(html.contains(r#"href="https://example.com/docs""#), "html: {html}");
    assert!(
        !html.contains("data-on:click.prevent"),
        "non-view hrefs should not wire @get: {html}"
    );
}

#[test]
fn section_macro_omits_link_region_when_label_missing() {
    // Both link_href and link_label must be set to render the link.
    #[derive(Serialize)]
    struct Ctx {}
    let source = r#"{% from '_ship.html.j2' import section, endsection %}{{ section('NoLink', link_href='/x') }}{{ endsection() }}"#;
    let html = render_view_source("test-section-no-link-label", source, &Ctx {}).unwrap();
    assert!(!html.contains(r#"href="/x""#), "no link should render without label: {html}");
}

/// Regression: editing a sibling partial must invalidate the user-env
/// cache so the next render sees the new partial. Before this fix the
/// cache key hashed only the top view's source, so partial edits
/// silently served stale content until the daemon restarted.
#[test]
fn sibling_partial_edit_busts_cache() {
    use tempfile::TempDir;

    #[derive(Serialize)]
    struct Ctx {}

    let dir = TempDir::new().unwrap();
    let view_dir = runtime::ShipHome::project_ship_dir(dir.path()).join("views/cache-bust");
    std::fs::create_dir_all(view_dir.join("partials")).unwrap();

    let index_path = view_dir.join("index.html.j2");
    let partial_path = view_dir.join("partials/body.html.j2");
    let index_src = r#"{% include 'cache-bust/partials/body.html.j2' %}"#;
    std::fs::write(&index_path, index_src).unwrap();
    std::fs::write(&partial_path, "FIRST").unwrap();

    // Point discovery at our temp project dir.
    // SAFETY: tests in this binary serially, and we restore prior state.
    let prior = runtime::env::read_raw(runtime::env::Var::Dir).ok();
    unsafe {
        runtime::env::set(runtime::env::Var::Dir, runtime::ShipHome::project_ship_dir(dir.path()));
    }

    let first = render_view_source("cache-bust/index.html.j2", index_src, &Ctx {})
        .expect("first render");
    assert!(first.contains("FIRST"), "first render must include original partial: {first}");

    // Edit the sibling; index is unchanged. If the cache key hashed only
    // the index, this second render would still return "FIRST".
    std::fs::write(&partial_path, "SECOND").unwrap();

    let second = render_view_source("cache-bust/index.html.j2", index_src, &Ctx {})
        .expect("second render");

    unsafe {
        match prior {
            Some(v) => runtime::env::set(runtime::env::Var::Dir, v),
            None => runtime::env::remove(runtime::env::Var::Dir),
        }
    }

    assert!(
        second.contains("SECOND"),
        "cache must invalidate on sibling edit, got: {second}"
    );
    assert!(
        !second.contains("FIRST"),
        "cache must not serve stale partial content, got: {second}"
    );
}
