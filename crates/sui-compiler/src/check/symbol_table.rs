//! Cross-check symbol table built from a paired controller.
//!
//! The byte-walking scope-bound check answers "is this name declared?"
//! but stops short of "what kind of binding is it?". The
//! type-strengthening checks need the second answer: an arrow-shaped
//! scope action behaves differently from a ref-shaped scope handle,
//! and a derived getter is only a callable when invoked with `()`.
//!
//! This module owns the kind classifier so each downstream check
//! (`scope_member`, `bind_this_target`, `on_callable`) can ask
//! `kind_of_scope("foo")` instead of re-walking the controller source.

use std::collections::BTreeMap;

use crate::check::controller_parse::ParsedController;
use crate::check::js_scan::{is_ident_cont, is_ident_start, skip_balanced, skip_trivia};
use crate::ir::Program;

/// Author-declared shape of a scope key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// `key: ref(initial)` — DOM-handle slot; written by the runtime,
    /// read like a signal.
    Ref,
    /// `key: signal(initial)` — readable + writable cell.
    Signal,
    /// `key: derived(() => ...)` — read-only computed getter.
    Derived,
    /// `key: () => ...` or `key: (e) => ...` — bound action / handler.
    Action,
    /// Anything else (plain values, expressions we can't classify).
    Other,
}

impl ScopeKind {
    pub fn label(self) -> &'static str {
        match self {
            ScopeKind::Ref => "ref",
            ScopeKind::Signal => "signal",
            ScopeKind::Derived => "derived",
            ScopeKind::Action => "action",
            ScopeKind::Other => "value",
        }
    }

    pub fn is_callable(self) -> bool {
        matches!(self, ScopeKind::Action)
    }
}

/// Symbol table merged across the program's `<data>` / `<prop>`
/// directives and the paired controller's top-level sections.
#[derive(Debug, Default)]
pub struct SymbolTable {
    pub data: BTreeMap<String, KeyEntry>,
    pub props: BTreeMap<String, KeyEntry>,
    pub derived: BTreeMap<String, KeyEntry>,
    pub on: BTreeMap<String, KeyEntry>,
    /// `use:` keys — opaque type, treated as Other.
    pub use_keys: BTreeMap<String, KeyEntry>,
    /// Scope-factory return keys, with kinds.
    pub scope: BTreeMap<String, ScopeEntry>,
}

#[derive(Debug, Clone)]
pub struct KeyEntry {
    /// Byte offset into the controller source (or 0 if the key is
    /// declared in the `.sui`).
    pub offset: usize,
    /// Source kind label — surfaced in hover.
    pub kind_label: &'static str,
}

#[derive(Debug, Clone)]
pub struct ScopeEntry {
    pub offset: usize,
    pub kind: ScopeKind,
}

impl SymbolTable {
    pub fn build(program: &Program, controller: Option<&ParsedController>) -> Self {
        let mut t = Self::default();
        for d in &program.data {
            t.data.insert(
                d.name.clone(),
                KeyEntry { offset: 0, kind_label: "data" },
            );
        }
        for p in &program.props {
            let cm = crate::emit::util::camel_of(&p.name);
            t.props.insert(cm, KeyEntry { offset: 0, kind_label: "prop" });
        }
        let Some(c) = controller else {
            return t;
        };
        for sec in &c.sections {
            match sec.name.as_str() {
                "derived" => collect_section_keys(c, sec.value_span, "derived", &mut t.derived),
                "on" => collect_section_keys(c, sec.value_span, "on", &mut t.on),
                "use" => collect_use_keys(c, sec.value_span, &mut t.use_keys),
                "scope" => collect_scope_kinds(c, sec.value_span, &mut t.scope),
                _ => {}
            }
        }
        t
    }

    pub fn kind_of_scope(&self, name: &str) -> Option<ScopeKind> {
        self.scope.get(name).map(|e| e.kind)
    }

    /// Iterator of every (name, label) pair for completion.
    pub fn all_root_names(&self) -> Vec<(String, &'static str)> {
        let mut out: Vec<(String, &'static str)> = Vec::new();
        out.extend(self.data.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.props.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.derived.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.on.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.use_keys.iter().map(|(k, v)| (k.clone(), v.kind_label)));
        out.extend(self.scope.iter().map(|(k, v)| (k.clone(), v.kind.label())));
        out
    }
}

fn collect_section_keys(
    c: &ParsedController,
    span: crate::diagnostics::Span,
    label: &'static str,
    into: &mut BTreeMap<String, KeyEntry>,
) {
    let bytes = c.source.as_bytes();
    if span.start >= span.end || bytes[span.start] != b'{' {
        return;
    }
    let start = span.start + 1;
    let end = span.end.saturating_sub(1).min(bytes.len());
    walk_object_keys(bytes, start, end, |name, offset| {
        into.insert(name.to_string(), KeyEntry { offset, kind_label: label });
    });
}

fn collect_use_keys(
    c: &ParsedController,
    span: crate::diagnostics::Span,
    into: &mut BTreeMap<String, KeyEntry>,
) {
    let bytes = c.source.as_bytes();
    let mut i = span.start;
    let end = span.end.min(bytes.len());
    i = skip_trivia(bytes, i);
    if i < end && bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let after = skip_trivia(bytes, close);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            i = skip_trivia(bytes, after + 2);
        }
    }
    if i < end && bytes[i] == b'(' {
        i = skip_trivia(bytes, i + 1);
    }
    if i < end && bytes[i] == b'{' {
        let close = skip_balanced(bytes, i, end);
        walk_object_keys(bytes, i + 1, close.saturating_sub(1), |name, offset| {
            into.insert(name.to_string(), KeyEntry { offset, kind_label: "use" });
        });
    }
}

fn collect_scope_kinds(
    c: &ParsedController,
    span: crate::diagnostics::Span,
    into: &mut BTreeMap<String, ScopeEntry>,
) {
    let bytes = c.source.as_bytes();
    let start = span.start;
    let end = span.end.min(bytes.len());
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
    let obj_start = if let Some(s) = last_return_obj {
        s
    } else {
        // Expression body: skip params + `=>` + optional `(`.
        let mut k = skip_trivia(bytes, start);
        if k >= end {
            return;
        }
        if bytes[k] == b'(' {
            k = skip_balanced(bytes, k, end);
        } else if is_ident_start(bytes[k]) {
            while k < end && is_ident_cont(bytes[k]) {
                k += 1;
            }
        } else {
            return;
        }
        k = skip_trivia(bytes, k);
        if k + 1 >= end || bytes[k] != b'=' || bytes[k + 1] != b'>' {
            return;
        }
        k = skip_trivia(bytes, k + 2);
        while k < end && bytes[k] == b'(' {
            k = skip_trivia(bytes, k + 1);
        }
        if k >= end || bytes[k] != b'{' {
            return;
        }
        k
    };
    let obj_end = skip_balanced(bytes, obj_start, end);
    walk_object_entries(
        bytes,
        obj_start + 1,
        obj_end.saturating_sub(1),
        |name, key_offset, value_start, value_end| {
            let kind = classify_scope_value(bytes, value_start, value_end);
            into.insert(name.to_string(), ScopeEntry { offset: key_offset, kind });
        },
    );
}

fn classify_scope_value(bytes: &[u8], start: usize, end: usize) -> ScopeKind {
    let mut i = skip_trivia(bytes, start);
    if i >= end {
        return ScopeKind::Other;
    }
    // Arrow shapes: `(x) => …` / `x => …` / `() => …` => Action.
    if bytes[i] == b'(' {
        let close = skip_balanced(bytes, i, end);
        let after = skip_trivia(bytes, close);
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return ScopeKind::Action;
        }
    }
    if is_ident_start(bytes[i]) {
        let s = i;
        while i < end && is_ident_cont(bytes[i]) {
            i += 1;
        }
        let word = std::str::from_utf8(&bytes[s..i]).unwrap_or("");
        let after = skip_trivia(bytes, i);
        // `name => …` arrow shorthand.
        if after + 1 < end && bytes[after] == b'=' && bytes[after + 1] == b'>' {
            return ScopeKind::Action;
        }
        if after < end && bytes[after] == b'(' {
            return match word {
                "ref" => ScopeKind::Ref,
                "signal" => ScopeKind::Signal,
                "derived" | "computed" => ScopeKind::Derived,
                _ => ScopeKind::Other,
            };
        }
    }
    ScopeKind::Other
}

/// Visit `key: value` entries within an object literal slice. The
/// callback receives the key name, the key's byte offset, and the
/// value-span range.
fn walk_object_entries<F>(bytes: &[u8], start: usize, end: usize, mut f: F)
where
    F: FnMut(&str, usize, usize, usize),
{
    let mut i = start;
    while i < end {
        i = skip_trivia(bytes, i);
        if i >= end {
            break;
        }
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        let key_start = i;
        let key;
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            i += 1;
            let s = i;
            while i < end && bytes[i] != q {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            key = std::str::from_utf8(&bytes[s..i])
                .unwrap_or("")
                .to_string();
            if i < end {
                i += 1;
            }
        } else if is_ident_start(bytes[i]) {
            while i < end && is_ident_cont(bytes[i]) {
                i += 1;
            }
            key = std::str::from_utf8(&bytes[key_start..i])
                .unwrap_or("")
                .to_string();
        } else {
            i = next_entry(bytes, i, end);
            continue;
        }
        let after_key = i;
        i = skip_trivia(bytes, i);
        // Shorthand entry — value is the key itself (treat as "Other").
        if i < end && bytes[i] == b',' {
            f(&key, key_start, key_start, after_key);
            i += 1;
            continue;
        }
        if i >= end || bytes[i] != b':' {
            f(&key, key_start, key_start, after_key);
            i = next_entry(bytes, i, end);
            continue;
        }
        i += 1;
        i = skip_trivia(bytes, i);
        let value_start = i;
        let value_end = next_entry(bytes, i, end);
        f(&key, key_start, value_start, value_end);
        i = value_end;
        if i < end && bytes[i] == b',' {
            i += 1;
        }
    }
}

fn walk_object_keys<F>(bytes: &[u8], start: usize, end: usize, mut f: F)
where
    F: FnMut(&str, usize),
{
    walk_object_entries(bytes, start, end, |name, key_offset, _, _| {
        f(name, key_offset)
    });
}

/// Walk to the next top-level `,` or to `end`.
fn next_entry(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut k = start;
    let mut in_str: Option<u8> = None;
    while k < end {
        let c = bytes[k];
        if let Some(q) = in_str {
            if c == b'\\' {
                k += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            k += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => {
                in_str = Some(c);
                k += 1;
            }
            b'(' | b'{' | b'[' => {
                k = skip_balanced(bytes, k, end);
            }
            b',' => return k,
            _ => k += 1,
        }
    }
    end
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::controller_parse::parse;
    use std::path::PathBuf;

    fn ctrl(src: &str) -> ParsedController {
        parse(PathBuf::from("test.controller.ts"), src.to_string()).expect("parsed")
    }

    fn empty_program() -> Program {
        Program {
            data: vec![],
            props: vec![],
            styles: String::new(),
            template: vec![],
            source_len: 0,
            span: crate::diagnostics::Span::point(0),
        }
    }

    #[test]
    fn classifies_scope_kinds() {
        let c = ctrl(
            r#"export default controller({
                scope: () => ({
                    el: ref(null),
                    n: signal(0),
                    doubled: derived(() => 1),
                    handle: (e) => 1,
                    plain: 42,
                }),
            });"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert_eq!(t.kind_of_scope("el"), Some(ScopeKind::Ref));
        assert_eq!(t.kind_of_scope("n"), Some(ScopeKind::Signal));
        assert_eq!(t.kind_of_scope("doubled"), Some(ScopeKind::Derived));
        assert_eq!(t.kind_of_scope("handle"), Some(ScopeKind::Action));
        assert_eq!(t.kind_of_scope("plain"), Some(ScopeKind::Other));
    }

    #[test]
    fn collects_derived_and_on_keys() {
        let c = ctrl(
            r#"export default controller({
                derived: { total: () => 1 },
                on: { increment: () => {} },
            });"#,
        );
        let t = SymbolTable::build(&empty_program(), Some(&c));
        assert!(t.derived.contains_key("total"));
        assert!(t.on.contains_key("increment"));
    }
}
