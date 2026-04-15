//! Attack-vector coverage for the HTML sanitizer. Twelve-plus cases as
//! required by the Phase 1 acceptance criteria.

use super::*;

#[test]
fn strips_script_tags() {
    let out = sanitize("<div>hi</div><script>alert(1)</script>");
    assert!(out.contains("<div>hi</div>"));
    assert!(!out.contains("<script"), "script leaked: {out}");
    assert!(!out.contains("alert(1)"));
}

#[test]
fn strips_nested_script_inside_svg() {
    let out = sanitize("<svg><script>alert(1)</script></svg>");
    assert!(!out.contains("<script"), "svg-nested script leaked: {out}");
}

#[test]
fn strips_onclick_and_other_event_handlers() {
    let out = sanitize(r#"<button onclick="alert(1)" onmouseover="x()">go</button>"#);
    assert!(out.contains("go"));
    assert!(!out.contains("onclick"), "onclick leaked: {out}");
    assert!(!out.contains("onmouseover"));
    assert!(!out.contains("alert"));
}

#[test]
fn strips_onload_and_onerror() {
    let out = sanitize(r#"<img src="/x.png" onerror="alert(1)" onload="x()">"#);
    assert!(!out.contains("onerror"));
    assert!(!out.contains("onload"));
}

#[test]
fn rejects_javascript_href() {
    let out = sanitize(r#"<a href="javascript:alert(1)">click</a>"#);
    assert!(!out.contains("javascript:"), "javascript scheme leaked: {out}");
}

#[test]
fn rejects_data_url_on_img_src() {
    let out = sanitize(r#"<img src="data:text/html,<script>alert(1)</script>">"#);
    assert!(!out.contains("data:text/html"), "data: url leaked: {out}");
    assert!(!out.contains("<script"));
}

#[test]
fn rejects_vbscript_scheme() {
    let out = sanitize(r#"<a href="vbscript:msgbox(1)">x</a>"#);
    assert!(!out.contains("vbscript:"));
}

#[test]
fn preserves_ship_element_with_attributes() {
    let out = sanitize(r#"<ship-diff variant="unified" data-path="/x.rs">body</ship-diff>"#);
    assert!(out.contains("<ship-diff"), "ship-diff missing: {out}");
    assert!(out.contains("data-path=\"/x.rs\""));
}

#[test]
fn preserves_data_attributes_on_standard_tags() {
    let out = sanitize(r#"<div id="root" data-init="@get('/api/x')" data-on-click="$n++"></div>"#);
    assert!(out.contains("id=\"root\""));
    assert!(out.contains("data-init"));
    assert!(out.contains("data-on-click"));
}

#[test]
fn preserves_aria_attributes() {
    let out = sanitize(r#"<button aria-label="Close" aria-pressed="true">x</button>"#);
    assert!(out.contains("aria-label=\"Close\""));
    assert!(out.contains("aria-pressed=\"true\""));
}

#[test]
fn strips_iframe_object_embed() {
    let dangerous = r#"<iframe src="//evil"></iframe><object data="x"></object><embed src="y">"#;
    let out = sanitize(dangerous);
    assert!(!out.contains("<iframe"));
    assert!(!out.contains("<object"));
    assert!(!out.contains("<embed"));
}

#[test]
fn strips_form_action_javascript_scheme() {
    let out = sanitize(r#"<form action="javascript:alert(1)"><input type="submit"></form>"#);
    assert!(!out.contains("javascript:"), "form action javascript: leaked: {out}");
}

#[test]
fn strips_svg_and_foreign_xml_by_default() {
    // ammonia drops <svg> by default unless added explicitly — ensures we
    // aren't accidentally opening an SVG-script vector.
    let out = sanitize(r#"<svg><g><text>hi</text></g></svg>"#);
    assert!(!out.contains("<svg"));
}

#[test]
fn preserves_nested_ship_elements() {
    let src = r#"<ship-card><ship-button label="ok">+</ship-button></ship-card>"#;
    let out = sanitize(src);
    assert!(out.contains("<ship-card"));
    assert!(out.contains("<ship-button"));
    assert!(out.contains("label=\"ok\""));
}

#[test]
fn mixed_case_script_tag_is_normalized_and_stripped() {
    let out = sanitize("<ScRiPt>alert(1)</ScRiPt>");
    assert!(!out.to_ascii_lowercase().contains("<script"), "case-folded script leaked: {out}");
}

#[test]
fn obfuscated_javascript_url_with_whitespace_rejected() {
    let out = sanitize(r#"<a href="  javascript:alert(1)">go</a>"#);
    assert!(!out.to_ascii_lowercase().contains("javascript:"));
}

#[test]
fn standard_text_and_structure_tags_preserved() {
    let src = "<section><h1>Hi</h1><p><strong>bold</strong> and <em>em</em></p></section>";
    let out = sanitize(src);
    assert!(out.contains("<section>"));
    assert!(out.contains("<h1>Hi</h1>"));
    assert!(out.contains("<strong>"));
}
