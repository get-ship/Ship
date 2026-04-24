//! Tests for the pack injection helpers.

use super::*;

#[test]
fn pack_bundles_known_packs() {
    let editor = pack_bundles("editor");
    assert_eq!(editor.len(), 2);
    assert!(editor.contains(&"ship-code-editor.js"));
    assert!(editor.contains(&"ship-markdown-editor.js"));

    let terminal = pack_bundles("terminal");
    assert_eq!(terminal, &["ship-terminal.js"]);
}

#[test]
fn pack_bundles_unknown_returns_empty() {
    assert!(pack_bundles("nonsense").is_empty());
    assert!(pack_bundles("").is_empty());
}

#[test]
fn render_pack_scripts_emits_script_tags() {
    let html = render_pack_scripts(&["editor".to_string()]);
    assert!(html.contains("ship-code-editor.js"));
    assert!(html.contains("ship-markdown-editor.js"));
    assert!(html.contains("type=\"module\""));
    assert!(html.contains("/api/assets/"));
}

#[test]
fn render_pack_scripts_dedupes_across_packs() {
    // Two packs sharing a bundle (hypothetical) would only emit once —
    // today no pack overlaps, but the invariant protects future additions.
    let html = render_pack_scripts(&["editor".to_string(), "editor".to_string()]);
    let count = html.matches("ship-code-editor.js").count();
    assert_eq!(count, 1, "duplicate packs must dedupe: {html}");
}

#[test]
fn render_pack_scripts_ignores_unknown_packs() {
    let html = render_pack_scripts(&["bogus".to_string(), "terminal".to_string()]);
    assert!(html.contains("ship-terminal.js"));
    // "bogus" has no bundles and simply contributes nothing.
    assert_eq!(html.matches("<script").count(), 1);
}

#[test]
fn render_pack_scripts_empty_for_empty_input() {
    assert_eq!(render_pack_scripts(&[]), "");
}

#[test]
fn inject_packs_noop_when_empty() {
    let html = "<html><head></head><body></body></html>".to_string();
    let out = inject_packs(html.clone(), &[], false);
    assert_eq!(out, html);
    let out = inject_packs(html.clone(), &[], true);
    assert_eq!(out, html);
}

#[test]
fn inject_packs_full_page_injects_before_close_head() {
    let html = "<html><head><title>x</title></head><body>content</body></html>".to_string();
    let out = inject_packs(html, &["terminal".to_string()], false);
    // Injection must appear inside <head>, before </head>.
    let script_pos = out.find("ship-terminal.js").unwrap();
    let close_head_pos = out.find("</head>").unwrap();
    assert!(
        script_pos < close_head_pos,
        "pack script should precede </head>: {out}"
    );
    assert!(out.contains("<title>x</title>"), "existing head preserved");
}

#[test]
fn inject_packs_full_page_handles_uppercase_head() {
    let html = "<HTML><HEAD><TITLE>x</TITLE></HEAD><BODY></BODY></HTML>".to_string();
    let out = inject_packs(html, &["terminal".to_string()], false);
    let script_pos = out.find("ship-terminal.js").unwrap();
    let close_head_pos = out.to_lowercase().find("</head>").unwrap();
    assert!(script_pos < close_head_pos);
}

#[test]
fn inject_packs_fragment_prepends() {
    let html = "<div class=\"ship-view\">content</div>".to_string();
    let out = inject_packs(html.clone(), &["editor".to_string()], true);
    assert!(out.starts_with("<script"));
    assert!(out.contains("ship-code-editor.js"));
    assert!(out.contains("ship-markdown-editor.js"));
    assert!(out.ends_with("<div class=\"ship-view\">content</div>"));
}

#[test]
fn inject_packs_full_page_no_head_prepends() {
    // Malformed or fragment-shaped HTML misclassified as full page still
    // gets the scripts prepended rather than silently dropped.
    let html = "<div>orphan</div>".to_string();
    let out = inject_packs(html.clone(), &["terminal".to_string()], false);
    assert!(out.starts_with("<script"));
    assert!(out.contains("ship-terminal.js"));
    assert!(out.ends_with(&html));
}
