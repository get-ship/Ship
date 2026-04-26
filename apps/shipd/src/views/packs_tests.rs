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
fn validate_packs_accepts_known_names() {
    let manifest = std::path::PathBuf::from("/tmp/view/manifest.jsonc");
    let packs = vec!["editor".to_string(), "git".to_string(), "terminal".to_string()];
    let out = validate_packs(&manifest, &packs).expect("known packs validate");
    assert_eq!(out, packs);
}

#[test]
fn validate_packs_rejects_unknown_with_path_and_offender() {
    let manifest = std::path::PathBuf::from("/tmp/view/manifest.jsonc");
    let packs = vec!["editor".to_string(), "viz".to_string()];
    let err = validate_packs(&manifest, &packs).expect_err("unknown pack must reject");
    assert_eq!(err.offending, "viz");
    assert_eq!(err.manifest, manifest);
    let msg = err.to_string();
    assert!(msg.contains("/tmp/view/manifest.jsonc"), "msg names path: {msg}");
    assert!(msg.contains("\"viz\""), "msg names offender: {msg}");
}

#[test]
fn validate_packs_empty_is_ok() {
    let manifest = std::path::PathBuf::from("/tmp/view/manifest.jsonc");
    assert_eq!(validate_packs(&manifest, &[]).unwrap(), Vec::<String>::new());
}

#[test]
fn read_view_packs_drops_list_when_unknown_pack_present() {
    // A manifest with one valid + one invalid pack must drop the entire
    // list — partial injection would mask the typo. The valid `editor`
    // bundles must NOT show up in the output.
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path().to_path_buf();
    std::fs::write(
        dir.join("manifest.jsonc"),
        r#"{"name":"x","type":"html","packs":["editor","viz"]}"#,
    )
    .unwrap();
    let out = read_view_packs(&dir);
    assert!(out.is_empty(), "unknown pack rejects whole list: {out:?}");
}

#[test]
fn read_view_packs_returns_known_packs_unchanged() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path().to_path_buf();
    std::fs::write(
        dir.join("manifest.jsonc"),
        r#"{"name":"x","type":"html","packs":["git"]}"#,
    )
    .unwrap();
    assert_eq!(read_view_packs(&dir), vec!["git".to_string()]);
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
