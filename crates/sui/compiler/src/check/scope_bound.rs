//! Scope-bound expressions rule: every free identifier in a template
//! interpolation or `on:` handler must resolve to a name the runtime
//! actually has. Scope sources, in resolution order:
//!
//!  * `<data name="…">`, `<prop name="…">` declarations on the program.
//!  * `scope` factory return keys, `derived` keys, `on` keys, and
//!    `use` keys from the paired controller (when one exists).
//!  * The setup-context idents `ship`, `route`, `navigate`.
//!  * JS / browser globals from a small fixed allowlist.
//!  * Locals introduced by an arrow's parameter list (handlers can
//!    write `(e) => …`; `e` is in scope inside the body).
//!
//! Anything else triggers a compile error pointing at the unknown name.

use std::collections::BTreeSet;

use crate::check::controller_parse::ParsedController;
use crate::check::js_scan::{
    is_ident_cont, is_ident_start, next_top_separator, skip_balanced, skip_trivia,
};
use crate::diagnostics::{CompileError, Span};
use crate::ir::*;

use super::scope_bound_expr::check_expr;

/// JS/browser globals + framework-known ctx names that are always
/// available without an explicit declaration.
const BUILTINS: &[&str] = &[
    // Setup context.
    "ship", "route", "navigate", "ctx",
    // Namespace heads — the `check::scope_member` pass type-checks
    // member access through these so typos are flagged with a
    // useful "did you mean…" message instead of "scope is not
    // declared".
    "scope", "derived", "props",
    // JS standard objects.
    "Math", "JSON", "Object", "Array", "String", "Number", "Boolean", "Date",
    "Symbol", "Map", "Set", "WeakMap", "WeakSet", "Error", "RegExp",
    "Promise", "BigInt", "ArrayBuffer", "DataView",
    "Int8Array", "Uint8Array", "Uint8ClampedArray",
    "Int16Array", "Uint16Array", "Int32Array", "Uint32Array",
    "Float32Array", "Float64Array",
    // Standard literals/keywords-as-identifiers.
    "null", "undefined", "true", "false", "NaN", "Infinity",
    // Standard fns.
    "parseInt", "parseFloat", "isNaN", "isFinite", "encodeURIComponent",
    "decodeURIComponent", "encodeURI", "decodeURI",
    // Browser globals commonly used in handlers.
    "document", "window", "console", "globalThis",
    // Runtime helpers (unwrap variants are imported by the compiled
    // module; templates may also reference them via `use:` results).
    "unwrap", "unwrapOr",
    // Built-in transition factories — auto-imported when referenced by
    // `transition:` / `in:` / `out:` directives, so resolving them
    // without an explicit declaration is correct here.
    "fade", "slide", "scale",
];

pub fn check(
    program: &Program,
    controller: Option<&ParsedController>,
) -> Result<(), CompileError> {
    let scope = build_scope(program, controller);
    let mut visitor = Visitor { scope, error: None };
    visit_program(program, &mut visitor);
    match visitor.error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

#[derive(Debug)]
pub(super) struct Scope {
    pub(super) names: BTreeSet<String>,
}

impl Scope {
    pub(super) fn contains(&self, name: &str) -> bool {
        self.names.contains(name) || BUILTINS.contains(&name)
    }

    pub(super) fn list_for_diagnostic(&self) -> String {
        let mut declared: Vec<&str> = self
            .names
            .iter()
            .map(|s| s.as_str())
            .collect();
        declared.sort();
        if declared.is_empty() {
            "(none — only built-ins like Math, ship, route, navigate are available)".into()
        } else {
            declared.join(", ")
        }
    }
}

fn build_scope(program: &Program, controller: Option<&ParsedController>) -> Scope {
    let mut names = BTreeSet::new();
    for d in &program.data {
        names.insert(d.name.clone());
    }
    for p in &program.props {
        names.insert(super::super::emit::util::camel_of(&p.name));
    }
    if let Some(c) = controller {
        for sec in &c.sections {
            match sec.name.as_str() {
                "derived" | "on" => collect_object_keys(&c.source, sec.value_span, &mut names),
                "use" => collect_use_keys(&c.source, sec.value_span, &mut names),
                "scope" => collect_scope_return_keys(&c.source, sec.value_span, &mut names),
                _ => {}
            }
        }
    }
    Scope { names }
}

/// For `derived: { x: …, y: … }` — collect top-level keys.
fn collect_object_keys(src: &str, span: Span, into: &mut BTreeSet<String>) {
    let bytes = src.as_bytes();
    if span.start >= span.end {
        return;
    }
    if bytes[span.start] != b'{' {
        return;
    }
    let inner_start = span.start + 1;
    let inner_end = span.end.saturating_sub(1).min(bytes.len());
    let mut i = inner_start;
    while i < inner_end {
        i = skip_trivia(bytes, i);
        if i >= inner_end {
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        let key_start = i;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            let s = i;
            while i < inner_end && bytes[i] != q {
                if bytes[i] == b'\\' { i += 2; continue; }
                i += 1;
            }
            into.insert(src[s..i].to_string());
            if i < inner_end { i += 1; }
        } else if is_ident_start(bytes[i]) {
            while i < inner_end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            into.insert(src[key_start..i].to_string());
        } else {
            i = next_top_separator(bytes, i, inner_end);
            if i < inner_end && bytes[i] == b',' { i += 1; }
            continue;
        }
        i = next_top_separator(bytes, i, inner_end);
        if i < inner_end && bytes[i] == b',' { i += 1; }
    }
}

/// `use` may be either `{ key: factory(...), … }` (object form) or
/// `(s) => ({ key: factory(...), … })` (function form). Skip an
/// optional `(args) =>` prefix, then harvest keys from the object
/// literal that follows (unwrapping a parenthesized wrapper).
fn collect_use_keys(src: &str, span: Span, into: &mut BTreeSet<String>) {
    let bytes = src.as_bytes();
    let mut i = span.start;
    let end = span.end.min(bytes.len());
    i = skip_trivia(bytes, i);
    // Optional `(...)` parameter list.
    if i < end && bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let after = skip_trivia(bytes, close);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            i = skip_trivia(bytes, after + 2);
        }
        // If no `=>` follows, treat the original `(` as the start of
        // the object — but per `use` shape, that wouldn't be a key
        // bag; bail out without harvesting.
        if i == span.start {
            return;
        }
    }
    // Optional outer `(` wrapping the object literal: `({...})`.
    if i < end && bytes[i] == b'(' {
        i = skip_trivia(bytes, i + 1);
    }
    if i < end && bytes[i] == b'{' {
        let close = skip_balanced(bytes, i, end);
        collect_object_keys(src, Span::new(i, close), into);
    }
}

/// `scope:` factory — collect the keys of the object the factory returns.
///
/// Handles three author shapes:
///   - block body:    `scope: (init) => { ... return { x, y } }`
///   - expression body, parenthesised:  `scope: () => ({ x, y })`
///   - expression body, bare object:    `scope: () => ({x, y})` (same as above)
fn collect_scope_return_keys(src: &str, span: Span, into: &mut BTreeSet<String>) {
    let bytes = src.as_bytes();
    let start = span.start;
    let end = span.end.min(bytes.len());

    // Block body: walk for the last `return {…}` and lift its keys.
    let return_kw = b"return";
    let mut i = start;
    let mut last_return_obj: Option<usize> = None;
    while i + return_kw.len() < end {
        if bytes[i..].starts_with(return_kw) {
            let after = skip_trivia(bytes, i + return_kw.len());
            if after < end && bytes[after] == b'{' {
                last_return_obj = Some(after);
            }
        }
        i += 1;
    }
    if let Some(obj_start) = last_return_obj {
        let obj_end = skip_balanced(bytes, obj_start, end);
        collect_object_keys(src, Span::new(obj_start, obj_end), into);
        return;
    }

    // Expression-body arrow: skip past the parameter list + `=>` and lift
    // keys from the trailing object literal (with or without surrounding
    // parens).
    if let Some(obj_start) = expression_body_object(bytes, start, end) {
        let obj_end = skip_balanced(bytes, obj_start, end);
        collect_object_keys(src, Span::new(obj_start, obj_end), into);
    }
}

/// Locate the `{` that opens an arrow's expression-body object literal.
///
/// Walks past the param list (a `(...)` group, or a single bare ident),
/// the fat arrow `=>`, and any wrapping parens, then returns the offset
/// of the literal's opening `{`. Returns `None` if the value isn't an
/// arrow with an object expression body.
fn expression_body_object(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    let mut i = skip_trivia(bytes, start);
    if i >= end {
        return None;
    }
    // Step over the parameter list. Either `(...)` or a single ident.
    if bytes[i] == b'(' {
        i = skip_balanced(bytes, i, end);
    } else if is_ident_start(bytes[i]) {
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
    } else {
        return None;
    }
    i = skip_trivia(bytes, i);
    // Fat arrow.
    if i + 1 >= end || bytes[i] != b'=' || bytes[i + 1] != b'>' {
        return None;
    }
    i = skip_trivia(bytes, i + 2);
    // Strip wrapping parens (`() => ({...})`).
    while i < end && bytes[i] == b'(' {
        i = skip_trivia(bytes, i + 1);
    }
    if i < end && bytes[i] == b'{' {
        Some(i)
    } else {
        None
    }
}

pub(super) struct Visitor {
    pub(super) scope: Scope,
    /// First failure wins.
    pub(super) error: Option<CompileError>,
}

fn visit_program(program: &Program, v: &mut Visitor) {
    for d in &program.data {
        if v.error.is_some() { return; }
        check_expr(&d.source, v, &[]);
    }
    visit_instrs(&program.template, v);
}

fn visit_instrs(instrs: &[Instr], v: &mut Visitor) {
    for instr in instrs {
        if v.error.is_some() { return; }
        visit_instr(instr, v);
    }
}

fn visit_instr(instr: &Instr, v: &mut Visitor) {
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => {}
        Instr::DynamicText { expr } => check_expr(expr, v, &[]),
        Instr::Element(el) => {
            for a in &el.attrs {
                if v.error.is_some() { return; }
                visit_attr(a, v);
            }
            visit_instrs(&el.children, v);
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            for b in branches {
                check_expr(&b.cond, v, &[]);
                if v.error.is_some() { return; }
                visit_instrs(&b.body, v);
                if v.error.is_some() { return; }
            }
            if let Some(eb) = else_body {
                visit_instrs(eb, v);
            }
        }
        Instr::For(ForInstr { iterable, key, body, else_body, var_name, index_name, .. }) => {
            check_expr(iterable, v, &[]);
            if v.error.is_some() { return; }
            // Inside the loop body, the iteration var (and optional
            // index var) are bound.
            let mut locals: Vec<String> = vec![var_name.clone()];
            if let Some(idx) = index_name {
                locals.push(idx.clone());
            }
            if let Some(k) = key {
                check_expr(k, v, locals.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice());
                if v.error.is_some() { return; }
            }
            v.scope.names.extend(locals.iter().cloned());
            visit_instrs(body, v);
            for l in &locals {
                v.scope.names.remove(l);
            }
            if v.error.is_some() { return; }
            if let Some(eb) = else_body {
                visit_instrs(eb, v);
            }
        }
        Instr::Match(MatchInstr { discriminant, cases, default, .. }) => {
            check_expr(discriminant, v, &[]);
            for c in cases {
                if v.error.is_some() { return; }
                check_expr(&c.test, v, &[]);
                visit_instrs(&c.body, v);
            }
            if let Some(d) = default {
                visit_instrs(d, v);
            }
        }
        Instr::WhileLoading(wl) => visit_instrs(&wl.body, v),
        Instr::WhileError(we) => {
            // `{#while-error err of agents}` binds `err` inside the body.
            v.scope.names.insert(we.error_var.clone());
            visit_instrs(&we.body, v);
            v.scope.names.remove(&we.error_var);
        }
        Instr::Let(l) => {
            // `{#async X as v}` binds `v` inside the `{:ok}` body.
            check_expr(&l.value, v, &[]);
            if v.error.is_some() {
                return;
            }
            v.scope.names.insert(l.name.clone());
            visit_instrs(&l.body, v);
            v.scope.names.remove(&l.name);
        }
    }
}

fn visit_attr(attr: &AttrInstr, v: &mut Visitor) {
    match attr {
        AttrInstr::Static { .. } | AttrInstr::BindThis { .. } | AttrInstr::SlotReflectEmpty { .. } => {}
        AttrInstr::Expr { expr, .. }
        | AttrInstr::BindValue { expr, .. }
        | AttrInstr::BindChecked { expr, .. }
        | AttrInstr::Class { expr, .. }
        | AttrInstr::Style { expr, .. }
        | AttrInstr::Prop { expr, .. } => check_expr(expr, v, &[]),
        AttrInstr::DynamicAttr { name_expr, expr, .. } => {
            check_expr(name_expr, v, &[]);
            if v.error.is_none() {
                check_expr(expr, v, &[]);
            }
        }
        AttrInstr::On { handler, .. } => check_expr(handler, v, &[]),
        AttrInstr::Mutation { .. } => {}
        AttrInstr::Transition { opts, .. } => {
            if let Some(e) = opts {
                check_expr(e, v, &[]);
            }
        }
    }
}
