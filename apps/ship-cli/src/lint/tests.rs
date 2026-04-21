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
    ALL_RULES, RULE_INLINE_STYLE_BUDGET, RULE_NO_INLINE_SCRIPT, RULE_UNDEFINED_SIGNAL, ViewTarget,
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
