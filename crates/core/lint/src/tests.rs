//! Integration-style tests for the view linter.

use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::report::Severity;
use crate::rules::lint_view;
use crate::{
    ALL_RULES, RULE_AST_UNKNOWN_FILTER, RULE_AST_UNKNOWN_MACRO, RULE_AST_UNKNOWN_ROUTE,
    RULE_AST_UNUSED_IMPORT, RULE_INLINE_STYLE_BUDGET, RULE_MISSING_PACK_DECLARATION,
    RULE_NO_INLINE_SCRIPT, RULE_STALE_PACK_DECLARATION, RULE_UNDEFINED_SIGNAL, ViewTarget,
    collect_view_files,
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
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<ship-card></ship-card>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_STALE_PACK_DECLARATION]);
    assert!(findings.is_empty(), "{findings:?}");
}

// ── AST rules ─────────────────────────────────────────────────────────────────

#[test]
fn ast_unknown_macro_import_fires() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "{% from '_ship.html.j2' import bogus_xyz %}<div>x</div>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_AST_UNKNOWN_MACRO]);
    let hits: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_AST_UNKNOWN_MACRO)
        .collect();
    assert_eq!(hits.len(), 1, "{findings:?}");
    assert!(hits[0].message.contains("bogus_xyz"));
}

#[test]
fn ast_unused_import_fires() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "{% from '_ship.html.j2' import signal %}<div>no signals</div>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_AST_UNUSED_IMPORT]);
    assert!(
        findings.iter().any(|f| f.rule == RULE_AST_UNUSED_IMPORT),
        "{findings:?}"
    );
}

#[test]
fn ast_unknown_filter_fires() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<div>{{ title | urlencod }}</div>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_AST_UNKNOWN_FILTER]);
    let hits: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_AST_UNKNOWN_FILTER)
        .collect();
    assert_eq!(hits.len(), 1, "{findings:?}");
    assert!(
        hits[0].message.contains("urlencod"),
        "{}",
        hits[0].message
    );
}

#[test]
fn ast_unknown_filter_suggests_close_match() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<div>{{ title | urlencod }}</div>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_AST_UNKNOWN_FILTER]);
    let hit = findings
        .iter()
        .find(|f| f.rule == RULE_AST_UNKNOWN_FILTER)
        .expect("filter finding");
    assert!(
        hit.message.contains("urlencode"),
        "should suggest close match: {}",
        hit.message
    );
}

#[test]
fn ast_unknown_route_fires() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "<button data-on:click=\"@get('/api/nope/nope')\">Go</button>\n",
    );
    let findings = lint_view(&view(tmp.path(), "demo"), &[RULE_AST_UNKNOWN_ROUTE]);
    assert!(
        findings.iter().any(|f| f.rule == RULE_AST_UNKNOWN_ROUTE),
        "{findings:?}"
    );
}

#[test]
fn ast_pass_silent_on_clean_template() {
    let tmp = TempDir::new().unwrap();
    write(
        tmp.path(),
        "demo/index.html.j2",
        "{% from '_ship.html.j2' import signal %}<div {{ signal('n', 1) }}>ok</div>\n",
    );
    let findings = lint_view(
        &view(tmp.path(), "demo"),
        &[
            RULE_AST_UNKNOWN_MACRO,
            RULE_AST_UNUSED_IMPORT,
            RULE_AST_UNKNOWN_FILTER,
            RULE_AST_UNKNOWN_ROUTE,
        ],
    );
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn ast_real_git_view_baseline() {
    // Real view — establishes a baseline findings count we can alert on.
    let git_view = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.ship/views/git");
    if !git_view.exists() {
        eprintln!("skipping: real git view not present at {:?}", git_view);
        return;
    }
    let files = crate::collect_view_files(&git_view);
    assert!(!files.is_empty(), "git view has no templates");
    let target = ViewTarget {
        name: "git".into(),
        dir: git_view.clone(),
        files,
    };
    let findings = lint_view(
        &target,
        &[
            RULE_AST_UNKNOWN_MACRO,
            RULE_AST_UNUSED_IMPORT,
            RULE_AST_UNKNOWN_FILTER,
            RULE_AST_UNKNOWN_ROUTE,
        ],
    );
    // No parse errors or unknown macros — the git view is the reference.
    let parse_errs: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == "ast-parse-error")
        .collect();
    let unknown_macros: Vec<_> = findings
        .iter()
        .filter(|f| f.rule == RULE_AST_UNKNOWN_MACRO)
        .collect();
    assert!(parse_errs.is_empty(), "parse errors: {:?}", parse_errs);
    assert!(
        unknown_macros.is_empty(),
        "unknown macros: {:?}",
        unknown_macros
    );
    // Print for baseline observation (shown with `cargo test -- --nocapture`).
    for f in &findings {
        eprintln!("[{}] {}:{} {}", f.rule, f.line, f.col, f.message);
    }
}

#[test]
fn stale_pack_ignores_unknown_pack_names() {
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
