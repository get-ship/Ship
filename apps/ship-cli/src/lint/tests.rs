//! Integration-style tests for the view linter.
//!
//! Each test writes a synthetic `.ship/views/<name>/` tree to a tempdir and
//! runs `lint_view` directly so failures point at rule logic, not CLI
//! plumbing.

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::report::Severity;
use super::rules::lint_view;
use super::{
    ALL_RULES, RULE_INLINE_STYLE_BUDGET, RULE_MISSING_PACK_DECLARATION, RULE_NO_INLINE_SCRIPT,
    RULE_STALE_PACK_DECLARATION, RULE_UNDEFINED_SIGNAL, ViewTarget, collect_view_files,
};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn view(root: &Path, name: &str) -> ViewTarget {
    let dir: PathBuf = root.join(name);
    let files = collect_view_files(&dir);
    ViewTarget {
        name: name.to_string(),
        dir,
        files,
    }
}

// ── no-inline-script ──────────────────────────────────────────────────────────

#[test]
fn no_inline_script_fires_on_script_tag() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<html><body>\n<script>console.log('nope');</script>\n</body></html>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), ALL_RULES);
    let script_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_NO_INLINE_SCRIPT)
        .collect();
    assert_eq!(script_findings.len(), 1, "{findings:?}");
    assert_eq!(script_findings[0].severity, Severity::Error);
    assert_eq!(script_findings[0].line, 2);
}

#[test]
fn no_inline_script_ignores_commented_tag() {
    // A `<script>` inside a Jinja comment must not fire — stripping happens
    // before the regex scan.
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<html>\n{#- <script>x</script> -#}\n<div></div>\n</html>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), ALL_RULES);
    assert!(
        !findings.iter().any(|f| f.rule == RULE_NO_INLINE_SCRIPT),
        "{findings:?}"
    );
}

#[test]
fn no_inline_script_passes_on_clean_view() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<html><body><ship-card></ship-card></body></html>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_NO_INLINE_SCRIPT]);
    assert!(findings.is_empty(), "{findings:?}");
}

// ── inline-style-budget ───────────────────────────────────────────────────────

fn style_block(lines: usize) -> String {
    let mut s = String::from("<style>\n");
    for i in 0..lines {
        s.push_str(&format!(".x-{} {{ color: red; }}\n", i));
    }
    s.push_str("</style>\n");
    s
}

#[test]
fn inline_style_budget_quiet_under_fifty() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/partials/styles.html.j2",
        &style_block(20),
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_INLINE_STYLE_BUDGET]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn inline_style_budget_warns_past_fifty() {
    let tmp = TempDir::new().unwrap();
    // Split across index + partial to prove the rule sums them.
    write(tmp.path(), "demo/index.html.j2", &style_block(26));
    write(
        tmp.path(),
        "demo/partials/extra.html.j2",
        &style_block(25),
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_INLINE_STYLE_BUDGET]);
    let style: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_INLINE_STYLE_BUDGET)
        .collect();
    assert_eq!(style.len(), 1);
    assert_eq!(style[0].severity, Severity::Warn);
    assert!(
        style[0].message.contains("51"),
        "expected line total in message: {}",
        style[0].message
    );
}

#[test]
fn inline_style_budget_errors_past_one_fifty() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "demo/index.html.j2", &style_block(151));
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_INLINE_STYLE_BUDGET]);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
}

// ── undefined-signal ──────────────────────────────────────────────────────────

#[test]
fn undefined_signal_fires_on_missing_declaration() {
    let tmp = TempDir::new().unwrap();
    // `$count` is never declared — should warn.
    write(
        tmp.path(),
        "demo/index.html.j2",
        r#"<html><body>
<div data-text="$count"></div>
</body></html>
"#,
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_UNDEFINED_SIGNAL]);
    let hits: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_UNDEFINED_SIGNAL)
        .collect();
    assert_eq!(hits.len(), 1, "{findings:?}");
    assert_eq!(hits[0].severity, Severity::Warn);
    assert!(hits[0].message.contains("count"));
}

#[test]
fn undefined_signal_passes_when_declared_via_macro() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        r#"<html><body>
<div data-signals-count="0" data-text="$count"></div>
</body></html>
"#,
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_UNDEFINED_SIGNAL]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn undefined_signal_respects_signal_macro_declarations() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        r#"<div {{ signal('form', {}) }}>
  <input data-bind="form.title" value="$form">
</div>
"#,
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_UNDEFINED_SIGNAL]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn undefined_signal_ignores_datastar_intrinsics() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        r#"<ship-button data-on:click="$_ = 1; console.log($evt.detail)">X</ship-button>
"#,
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_UNDEFINED_SIGNAL]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn undefined_signal_allowlist_suppresses_warning() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        r#"{#- @lint:signals-ok: foo, bar -#}
<div data-text="$foo"></div>
<div data-text="$bar"></div>
<div data-text="$baz"></div>
"#,
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_UNDEFINED_SIGNAL]);
    let names: Vec<&str> = findings
        .iter()
        .filter(|f| f.rule == RULE_UNDEFINED_SIGNAL)
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(names.len(), 1, "{names:?}");
    assert!(names[0].contains("$baz"));
}

#[test]
fn undefined_signal_ignores_jinja_expressions() {
    // `$loop` inside `{{ … }}` is a Jinja value reference, not a Datastar
    // signal. The linter should let it through.
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        r#"<div data-text="{{ item.label }}$count"></div>
<div {{ signal('count', 0) }}></div>
"#,
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_UNDEFINED_SIGNAL]);
    assert!(findings.is_empty(), "{findings:?}");
}

// ── missing-pack-declaration ──────────────────────────────────────────────────

#[test]
fn missing_pack_fires_when_code_editor_used_without_declaration() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "demo/manifest.yaml", "name: demo\ntype: html\n");
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<div><ship-code-editor language=\"yaml\"></ship-code-editor></div>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_MISSING_PACK_DECLARATION]);
    let hits: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_MISSING_PACK_DECLARATION)
        .collect();
    assert_eq!(hits.len(), 1, "{findings:?}");
    assert_eq!(hits[0].severity, Severity::Error);
    assert!(
        hits[0].message.contains("editor"),
        "message must name the missing pack: {}",
        hits[0].message
    );
    assert!(hits[0].message.contains("<ship-code-editor>"));
}

#[test]
fn missing_pack_fires_when_markdown_editor_used_without_declaration() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "demo/manifest.yaml", "name: demo\ntype: html\n");
    write(
        tmp.path(),
        "demo/partials/editor.html.j2",
        "<ship-markdown-editor value=\"hi\"></ship-markdown-editor>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_MISSING_PACK_DECLARATION]);
    let hits: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_MISSING_PACK_DECLARATION)
        .collect();
    assert_eq!(hits.len(), 1, "{findings:?}");
}

#[test]
fn missing_pack_fires_when_terminal_used_without_declaration() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "demo/manifest.yaml", "name: demo\ntype: html\n");
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-terminal readonly></ship-terminal>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_MISSING_PACK_DECLARATION]);
    let hits: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_MISSING_PACK_DECLARATION)
        .collect();
    assert_eq!(hits.len(), 1, "{findings:?}");
    assert!(hits[0].message.contains("terminal"));
}

#[test]
fn missing_pack_passes_when_declaration_present_yaml() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/manifest.yaml",
        "name: demo\ntype: html\npacks:\n  - editor\n",
    );
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-code-editor></ship-code-editor>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_MISSING_PACK_DECLARATION]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_pack_passes_when_declaration_present_jsonc() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/manifest.jsonc",
        r#"{
  // Demo view
  "name": "demo",
  "type": "html",
  "packs": ["editor"]
}
"#,
    );
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-markdown-editor></ship-markdown-editor>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_MISSING_PACK_DECLARATION]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn missing_pack_passes_when_no_heavy_islands_used() {
    let tmp = TempDir::new().unwrap();
    write(tmp.path(), "demo/manifest.yaml", "name: demo\ntype: html\n");
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-card><ship-list-tile>hi</ship-list-tile></ship-card>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_MISSING_PACK_DECLARATION]);
    assert!(findings.is_empty(), "{findings:?}");
}

// ── stale-pack-declaration ────────────────────────────────────────────────────

#[test]
fn stale_pack_warns_when_declaration_unused() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/manifest.yaml",
        "name: demo\ntype: html\npacks:\n  - editor\n",
    );
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-card>no editors here</ship-card>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_STALE_PACK_DECLARATION]);
    let hits: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_STALE_PACK_DECLARATION)
        .collect();
    assert_eq!(hits.len(), 1, "{findings:?}");
    assert_eq!(hits[0].severity, Severity::Warn);
    assert!(hits[0].message.contains("editor"));
}

#[test]
fn stale_pack_silent_when_declaration_used() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/manifest.yaml",
        "name: demo\ntype: html\npacks:\n  - terminal\n",
    );
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-terminal></ship-terminal>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_STALE_PACK_DECLARATION]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn stale_pack_silent_when_no_packs_declared() {
    // No manifest, no packs — nothing to warn about.
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-card></ship-card>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_STALE_PACK_DECLARATION]);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn stale_pack_ignores_unknown_pack_names() {
    // An unknown pack name (`viz`) is out of scope for our rule — the
    // daemon ignores it, and the linter should too rather than silently
    // nuking future-reserved pack names.
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/manifest.yaml",
        "name: demo\ntype: html\npacks:\n  - viz\n",
    );
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-card></ship-card>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_STALE_PACK_DECLARATION]);
    assert!(findings.is_empty(), "{findings:?}");
}
