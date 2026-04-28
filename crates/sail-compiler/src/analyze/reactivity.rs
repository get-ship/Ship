//! Phase 5 — reactivity classification + identifier validation.
//!
//! Walks the parsed AST and reports diagnostics for:
//! - Identifiers in template expressions that don't match any
//!   declared name (state, derived, data, prop, function, or imported
//!   const) — emitted as warnings (could be a typo, could be a global)
//! - Unused reactive declarations (info)
//!
//! Token validation lives in [`super::tokens`] since it requires a
//! token source.
//!
//! Identifier extraction is regex-light: collect runs of identifier
//! characters from each expression, skipping string and template
//! literals. False positives on JS keywords and globals (`Math`,
//! `console`, etc.) are filtered against a small allowlist.

use crate::parse::ast::{Attr, AttrValue, Expr, Node, ScriptBlock, Template};
use crate::{Diagnostic, Severity};

/// Names always considered defined (JS globals, Sail magic).
const ALLOWED_GLOBALS: &[&str] = &[
    // JS globals
    "Math", "Date", "JSON", "Object", "Array", "String", "Number", "Boolean",
    "Promise", "Map", "Set", "RegExp", "Error", "console", "globalThis",
    "window", "document", "navigator", "location", "history", "fetch",
    "setTimeout", "setInterval", "clearTimeout", "clearInterval",
    "URL", "URLSearchParams", "FormData", "Headers", "Request", "Response",
    "Symbol", "BigInt", "Infinity", "NaN", "undefined", "null", "true", "false",
    "void", "typeof", "instanceof", "in", "of", "new", "this", "super",
    "function", "return", "if", "else", "for", "while", "do", "switch",
    "case", "default", "break", "continue", "try", "catch", "finally",
    "throw", "let", "const", "var", "async", "await", "yield",
    // Sail magic
    "el", "props", "cleanup",
];

/// Run reactivity analysis. Mutates `diagnostics` in place.
pub fn analyze(file: &crate::parse::ast::SailFile, diagnostics: &mut Vec<Diagnostic>) {
    let known = collect_known_names(file.script.as_ref());
    if let Some(tpl) = &file.template {
        check_template(tpl, &known, diagnostics);
    }
}

/// Names defined by the script block — anything an expression may
/// reference. Includes reactive decls, regular functions, and
/// best-effort `import`/`const`/`let` extraction from the raw script.
pub fn collect_known_names(script: Option<&ScriptBlock>) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for g in ALLOWED_GLOBALS { out.insert((*g).to_string()); }
    let Some(s) = script else { return out };
    for p in &s.props { out.insert(p.name.clone()); }
    for st in &s.state { out.insert(st.name.clone()); }
    for d in &s.derived { out.insert(d.name.clone()); }
    for d in &s.data { out.insert(d.name.clone()); }
    for f in &s.functions { out.insert(f.name.clone()); }
    // Best-effort imports + top-level consts.
    for line in s.raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("import ") {
            // Captures `import X from`, `import { A, B } from`, `import * as X from`.
            for name in extract_import_names(rest) { out.insert(name); }
        } else if let Some(rest) = trimmed.strip_prefix("const ")
            .or_else(|| trimmed.strip_prefix("let "))
            .or_else(|| trimmed.strip_prefix("var "))
        {
            if let Some(name) = extract_decl_name(rest) { out.insert(name); }
        }
    }
    out
}

fn extract_import_names(src: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(open) = src.find('{') {
        if let Some(close) = src[open..].find('}') {
            let inner = &src[open + 1..open + close];
            for raw in inner.split(',') {
                let n = raw.trim().split_whitespace().last().unwrap_or("");
                if is_ident(n) { names.push(n.to_string()); }
            }
        }
    } else if let Some(rest) = src.strip_prefix("* as ") {
        let n = rest.split_whitespace().next().unwrap_or("");
        if is_ident(n) { names.push(n.to_string()); }
    } else {
        // Default import: `import X from '...'`
        let n = src.split_whitespace().next().unwrap_or("");
        if is_ident(n) { names.push(n.to_string()); }
    }
    names
}

fn extract_decl_name(src: &str) -> Option<String> {
    let n = src
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '$')
        .collect::<String>();
    if is_ident(&n) { Some(n) } else { None }
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

fn check_template(
    tpl: &Template,
    known: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for n in &tpl.root { check_node(n, known, diagnostics); }
}

fn check_node(
    n: &Node,
    known: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match n {
        Node::Text(_) => {}
        Node::Interpolation(e) => check_expr(e, known, diagnostics),
        Node::RawHtml(e) => check_expr(e, known, diagnostics),
        Node::Element { attrs, children, .. } => {
            check_attrs(attrs, known, diagnostics);
            for c in children { check_node(c, known, diagnostics); }
        }
        Node::Component { props, events, children, .. } => {
            for p in props { check_expr(&p.value, known, diagnostics); }
            for e in events { check_expr(&e.handler, known, diagnostics); }
            for c in children { check_node(c, known, diagnostics); }
        }
        Node::If { cond, then_, elifs, else_ } => {
            check_expr(cond, known, diagnostics);
            for c in then_ { check_node(c, known, diagnostics); }
            for (e, body) in elifs {
                check_expr(e, known, diagnostics);
                for c in body { check_node(c, known, diagnostics); }
            }
            if let Some(b) = else_ { for c in b { check_node(c, known, diagnostics); } }
        }
        Node::For { iter, key, body, empty, .. } => {
            check_expr(iter, known, diagnostics);
            if let Some(k) = key { check_expr(k, known, diagnostics); }
            // Body lives in a scope with the loop var defined; skip
            // strict checking there for now (the emitter handles it
            // via the same `expr_references` helper).
            let _ = (body, empty);
        }
        Node::Await { promise, loading, then_, catch } => {
            check_expr(promise, known, diagnostics);
            for c in loading { check_node(c, known, diagnostics); }
            // then_/catch_ bodies similarly have a scoped binding.
            let _ = (then_, catch);
        }
        Node::Slot { props, fallback, .. } => {
            for (_, e) in props { check_expr(e, known, diagnostics); }
            for c in fallback { check_node(c, known, diagnostics); }
        }
        Node::SlotFill { body, .. } => for c in body { check_node(c, known, diagnostics); },
        Node::Host { attrs } => check_attrs(attrs, known, diagnostics),
    }
}

fn check_attrs(
    attrs: &[Attr],
    known: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for a in attrs {
        match &a.value {
            AttrValue::Static(_) => {}
            AttrValue::Interpolated(e) | AttrValue::BindTwoWay(e) => {
                check_expr(e, known, diagnostics);
            }
            AttrValue::OnEvent { handler, .. } => check_expr(handler, known, diagnostics),
        }
    }
}

fn check_expr(
    e: &Expr,
    known: &std::collections::HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for ident in extract_idents(&e.raw) {
        if !known.contains(&ident) {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: "SAIL/W0001".into(),
                message: format!("unknown identifier `{ident}` in expression `{}`", e.raw),
                span: e.span.clone(),
                help: Some(
                    "did you forget a `state`/`derived`/`prop` declaration, or import?".into(),
                ),
            });
        }
    }
}

/// Extract top-level identifiers from a JS expression, skipping
/// string/template literals, member access RHS (`.X`), and numeric
/// suffixes.
pub fn extract_idents(expr: &str) -> Vec<String> {
    let stripped = strip_js_comments(expr);
    let expr = stripped.as_str();
    let bytes = expr.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut in_str: Option<u8> = None;
    let mut after_dot = false;
    // Track when we're inside a template-literal `${…}` substitution so
    // identifiers there count as references (slider's `pct` is read via
    // `\`width: ${pct()}%\`` in fillStyle/thumbStyle — without this,
    // unused_derived false-flags it).
    let mut tmpl_subst_depth: u32 = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_str {
            if b == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
            if q == b'`' && b == b'$' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                in_str = None;
                tmpl_subst_depth = tmpl_subst_depth.saturating_add(1);
                i += 2;
                continue;
            }
            if b == q { in_str = None; }
            i += 1; continue;
        }
        if tmpl_subst_depth > 0 && b == b'}' {
            tmpl_subst_depth -= 1;
            if tmpl_subst_depth == 0 { in_str = Some(b'`'); }
            i += 1; continue;
        }
        if tmpl_subst_depth > 0 && b == b'{' {
            tmpl_subst_depth += 1;
            i += 1; continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' {
            in_str = Some(b); i += 1; continue;
        }
        if b == b'.' { after_dot = true; i += 1; continue; }
        if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            { i += 1; }
            if !after_dot {
                let ident = &expr[start..i];
                // Skip object-literal keys: identifier followed by `:`.
                let mut j = i;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() { j += 1; }
                let is_object_key = bytes.get(j) == Some(&b':');
                if !is_object_key {
                    out.push(ident.to_string());
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

/// Strip `//` and `/* … */` comments while preserving string literal
/// contents (so `'no // comment here'` stays untouched). Mirrors the
/// `strip_comments` helper in `analyze::rules::cleanup_pattern`; we
/// could share once a `parse::lex` utility lands. Without this,
/// comments with apostrophes (`can't`, `don't`) open phantom string
/// state that swallows real identifiers.
fn strip_js_comments(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'"' || c == b'\'' || c == b'`' {
            let q = c;
            out.push(c as char);
            i += 1;
            while i < b.len() {
                let x = b[i];
                if x == b'\\' && i + 1 < b.len() {
                    out.push(x as char);
                    out.push(b[i + 1] as char);
                    i += 2;
                    continue;
                }
                out.push(x as char);
                i += 1;
                if x == q { break; }
            }
            continue;
        }
        if c == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' { i += 1; }
            continue;
        }
        if c == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') { i += 1; }
            i = (i + 2).min(b.len());
            continue;
        }
        out.push(c as char);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn diag(src: &str) -> Vec<Diagnostic> {
        let f = parse::parse(src).unwrap();
        let mut d = Vec::new();
        analyze(&f, &mut d);
        d
    }

    #[test]
    fn known_state_passes() {
        let d = diag("<script>state x = 0</script><template><span>{x}</span></template>");
        assert!(d.is_empty(), "expected no diagnostics, got: {d:?}");
    }

    #[test]
    fn unknown_identifier_warns() {
        let d = diag("<template><span>{nope}</span></template>");
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].severity, Severity::Warning);
        assert!(d[0].message.contains("nope"));
    }

    #[test]
    fn member_access_only_checks_root() {
        let d = diag("<script>state user = null</script><template><span>{user.name.first}</span></template>");
        assert!(d.is_empty(), "got {d:?}");
    }

    #[test]
    fn import_names_recognized() {
        let d = diag(
            "<script>import { fmt } from './u'\nstate x = 0</script><template><span>{fmt(x)}</span></template>",
        );
        assert!(d.is_empty(), "got {d:?}");
    }

    #[test]
    fn const_declaration_recognized() {
        let d = diag(
            "<script>const PI = 3.14\nstate r = 1</script><template><span>{PI * r}</span></template>",
        );
        assert!(d.is_empty(), "got {d:?}");
    }

    #[test]
    fn js_globals_pass() {
        let d = diag("<template><span>{Math.PI}</span></template>");
        assert!(d.is_empty(), "got {d:?}");
    }

    #[test]
    fn event_handler_validated() {
        let d = diag("<template><button on:click={ghost}>x</button></template>");
        assert!(d.iter().any(|x| x.message.contains("ghost")));
    }

    #[test]
    fn extract_idents_skips_strings() {
        let ids = extract_idents("'foo' + bar");
        assert_eq!(ids, vec!["bar"]);
    }

    #[test]
    fn extract_idents_skips_member_rhs() {
        let ids = extract_idents("user.name.first");
        assert_eq!(ids, vec!["user"]);
    }

    #[test]
    fn extract_idents_skips_object_keys() {
        let ids = extract_idents("({ name: foo })");
        assert_eq!(ids, vec!["foo"]);
    }
}
