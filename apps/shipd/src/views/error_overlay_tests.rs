use super::*;

#[test]
fn extract_line_from_minijinja_error() {
    let msg = "invalid syntax at line 7, column 3";
    assert_eq!(extract_error_line(msg), Some(7));
}

#[test]
fn extract_line_uppercase() {
    let msg = "Error on Line 12";
    assert_eq!(extract_error_line(msg), Some(12));
}

#[test]
fn extract_line_none_when_absent() {
    let msg = "unexpected end of template";
    assert_eq!(extract_error_line(msg), None);
}

#[test]
fn overlay_contains_view_name_and_error() {
    let html = render_error_overlay("my-view/index", "<p>{{ x }</p>", "parse error at line 1");
    assert!(html.contains("my-view/index"), "must show view name");
    assert!(html.contains("parse error"), "must show error message");
}

#[test]
fn overlay_escapes_html_in_error() {
    let html = render_error_overlay("test", "<b>", "expected </div> tag");
    assert!(!html.contains("</div> tag"), "raw HTML must be escaped");
    assert!(html.contains("&lt;/div&gt;"), "HTML must be escaped");
}

#[test]
fn overlay_highlights_error_line() {
    let source = "line1\nline2\nline3\nline4\nline5";
    let html = render_error_overlay("test", source, "error at line 3");
    // Error line should have destructive background
    assert!(html.contains("line3"), "must show the error line content");
}

#[test]
fn overlay_shows_common_mistakes() {
    let html = render_error_overlay("test", "", "some error");
    assert!(html.contains("Common mistakes"), "must have tips section");
    assert!(html.contains("endfor"), "must mention endfor");
}

#[test]
fn overlay_truncates_long_source() {
    let source: String = (1..=50).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
    let html = render_error_overlay("test", &source, "unknown error");
    assert!(html.contains("more lines"), "must indicate truncation");
}

#[test]
fn empty_source_produces_overlay_without_code_block() {
    let html = render_error_overlay("test", "", "some error");
    // Should not panic, should still have the error banner
    assert!(html.contains("Template Error"));
}
