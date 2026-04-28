//! R1 — signal_call_required.
//!
//! Bare signal reads in template **text** interpolation are written as
//! `{count}` instead of `{count()}`. The runtime emit path wraps known
//! reactives at codegen time (see `emit::runtime::wrap_reactive_calls`),
//! but the call-site form is the documented interface and the LSP
//! code-action surface — `{count}` and `{count()}` should round-trip
//! the same diagnostic-clean state.
//!
//! Scope per the v1 spec: `Node::Interpolation` and `Node::RawHtml`
//! only (see [`super::for_each_text_expr`]). Attribute interpolation
//! is not flagged — sail's emitter wraps attribute exprs and the
//! corpus relies on the bare form there deliberately.
//!
//! False-positive guard: skip when the signal is followed by `.` —
//! that's either a setter call (`count.set(0)`) or a member access on
//! a signal whose return is intentionally object-shaped. Identifiers
//! preceded by `.` (member RHS) and identifiers inside string/template
//! literals are skipped — same rules as
//! [`super::super::reactivity::extract_idents`].

use crate::analyze::symbols::SymbolTable;
use crate::parse::ast::{Expr, SailFile};
use crate::{Diagnostic, DiagnosticFix, FixEdit, Severity};

pub fn analyze(file: &SailFile, symbols: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
    let Some(tpl) = file.template.as_ref() else { return };
    super::for_each_text_expr(tpl, |expr| check_expr(expr, symbols, diagnostics));
}

fn check_expr(expr: &Expr, symbols: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
    let mut already: std::collections::HashSet<String> = Default::default();
    for (name, follower) in extract_top_idents_with_followers(&expr.raw) {
        if !symbols.is_signal(&name) { continue; }
        if matches!(follower, Some('(') | Some('.')) { continue; }
        if !already.insert(name.clone()) { continue; }
        let fixed = insert_signal_call(&expr.raw, &name);
        let fix = expr.span.as_ref().map(|span| DiagnosticFix {
            title: format!("add `()` to `{name}` call"),
            edits: vec![FixEdit { range: span.clone(), replacement: fixed }],
        });
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "SAIL/E0101".to_string(),
            message: format!("signal `{name}` referenced without call in `{}`", expr.raw),
            span: expr.span.clone(),
            help: Some(format!(
                "signal '{name}' must be called as '{name}()' to read the value reactively"
            )),
            fix,
        });
    }
}

/// Walk a JS expression and yield each top-level identifier paired with
/// the next non-whitespace character (or `None` at end). Mirrors the
/// `extract_idents` shape from `analyze::reactivity`, extended to
/// report what immediately follows the identifier so R1 can decide
/// between a bare signal read and a call/member access.
fn extract_top_idents_with_followers(expr: &str) -> Vec<(String, Option<char>)> {
    let bytes = expr.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut in_str: Option<u8> = None;
    let mut after_dot = false;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
            if b == q { in_str = None; }
            i += 1;
            continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' { in_str = Some(b); i += 1; continue; }
        if b == b'.' { after_dot = true; i += 1; continue; }
        if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            {
                i += 1;
            }
            if !after_dot {
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
                let next = bytes.get(j).copied().map(|c| c as char);
                if next != Some(':') {
                    out.push((expr[start..i].to_string(), next));
                }
            }
            after_dot = false;
            continue;
        }
        if !b.is_ascii_whitespace() { after_dot = false; }
        i += 1;
    }
    out
}

/// Replace bare occurrences of `name` with `name()` in a JS expression.
/// "Bare" means: not preceded by `.`, not followed by `(` or an
/// identifier character. Handles the case where the same signal appears
/// multiple times in one expression (all occurrences are fixed).
fn insert_signal_call(expr: &str, name: &str) -> String {
    let name_bytes = name.as_bytes();
    let bytes = expr.as_bytes();
    let mut result = String::with_capacity(expr.len() + 2);
    let mut i = 0;
    while i < bytes.len() {
        let remaining = bytes.len() - i;
        if remaining >= name_bytes.len() && &bytes[i..i + name_bytes.len()] == name_bytes {
            let end = i + name_bytes.len();
            let preceded_by_dot = i > 0 && bytes[i - 1] == b'.';
            let next = bytes.get(end).copied();
            let next_is_ident = matches!(next, Some(c) if c.is_ascii_alphanumeric() || c == b'_');
            let next_is_call = next == Some(b'(');
            if !preceded_by_dot && !next_is_ident && !next_is_call {
                result.push_str(name);
                result.push_str("()");
                i = end;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn run(src: &str) -> Vec<Diagnostic> {
        let file = parse::parse(src).expect("parse");
        let symbols = SymbolTable::from_file(&file);
        let mut d = Vec::new();
        analyze(&file, &symbols, &mut d);
        d
    }

    fn warns(d: &[Diagnostic]) -> Vec<&Diagnostic> {
        d.iter().filter(|x| x.code == "SAIL/E0101").collect()
    }

    #[test]
    fn bare_signal_warns() {
        let d = run("<script>state count = 0</script><template>{count}</template>");
        let w = warns(&d);
        assert_eq!(w.len(), 1, "got {d:?}");
        assert!(w[0].message.contains("count"));
        assert!(w[0].help.as_deref().unwrap().contains("count()"));
        assert_eq!(w[0].severity, Severity::Error);
    }

    #[test]
    fn called_signal_passes() {
        let d = run("<script>state count = 0</script><template>{count()}</template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn setter_method_passes() {
        let d = run("<script>state count = 0</script><template>{count.set(0)}</template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn non_signal_identifier_passes() {
        let d = run("<script>const PI = 3.14</script><template>{PI}</template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn bare_signal_in_arithmetic_warns() {
        let d = run("<script>state count = 0</script><template>{count + 1}</template>");
        assert_eq!(warns(&d).len(), 1, "got {d:?}");
    }

    #[test]
    fn signal_in_attribute_passes() {
        // Attribute interpolation is auto-wrapped by the runtime emit path.
        let d = run(
            "<script>state open = false</script><template><div hidden={open}></div></template>",
        );
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn signal_member_access_skipped() {
        // Accept the false-negative on `user.name` to keep noise low.
        let d = run("<script>state user = null</script><template>{user.name}</template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn diagnostic_carries_span_at_interpolation() {
        // The bare `{count}` interpolation appears on line 3, col 14
        // (the inner `count` after the `{`). Verify the diagnostic
        // anchors there so the LSP highlights the offending expression
        // instead of file 0:0.
        let src = "<script>\nstate count = 0\n</script>\n<template><span>{count}</span></template>";
        let d = run(src);
        let w = warns(&d);
        assert_eq!(w.len(), 1, "got {d:?}");
        let span = w[0].span.as_ref().expect("expected span on R1 diagnostic");
        // `{count}` opens at col 17 of line 4 ('<' of <template> is col
        // 1, span="<template><span>{" → `{` at col 17, inner at 18).
        // The interpolation Expr span covers the inner expression text
        // (between the braces), so col is 18.
        assert_eq!(span.line, 4, "{:?}", span);
        assert_eq!(span.col, 18, "{:?}", span);
        assert!(span.end_col > span.col, "{:?}", span);
    }

    #[test]
    fn fix_is_present_and_inserts_call() {
        let d = run("<script>state count = 0</script><template>{count}</template>");
        let w = warns(&d);
        assert_eq!(w.len(), 1);
        let fix = w[0].fix.as_ref().expect("fix should be present for E0101");
        assert!(fix.title.contains("count"), "title: {}", fix.title);
        assert_eq!(fix.edits.len(), 1);
        assert_eq!(fix.edits[0].replacement, "count()");
    }

    #[test]
    fn fix_range_matches_span() {
        let src = "<script>\nstate count = 0\n</script>\n<template><span>{count}</span></template>";
        let d = run(src);
        let w = warns(&d);
        assert_eq!(w.len(), 1);
        let span = w[0].span.as_ref().expect("span on diagnostic");
        let fix = w[0].fix.as_ref().expect("fix on diagnostic");
        let fix_range = &fix.edits[0].range;
        assert_eq!(fix_range.line, span.line, "fix range line matches span");
        assert_eq!(fix_range.col, span.col, "fix range col matches span");
    }

    #[test]
    fn fix_corrects_arithmetic_expression() {
        let d = run("<script>state count = 0</script><template>{count + 1}</template>");
        let w = warns(&d);
        assert_eq!(w.len(), 1);
        let fix = w[0].fix.as_ref().expect("fix present");
        assert_eq!(fix.edits[0].replacement, "count() + 1");
    }

    #[test]
    fn insert_signal_call_replaces_bare_ident() {
        assert_eq!(insert_signal_call("count", "count"), "count()");
        assert_eq!(insert_signal_call("count + 1", "count"), "count() + 1");
        assert_eq!(insert_signal_call("count.set(0)", "count"), "count.set(0)");
        assert_eq!(insert_signal_call("count()", "count"), "count()");
        assert_eq!(insert_signal_call("notcount", "count"), "notcount");
    }
}
