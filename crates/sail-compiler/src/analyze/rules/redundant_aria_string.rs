//! redundant_aria_string — diagnostic `SAIL/E0104`.
//!
//! Catches the ergonomics gap #4 pattern: `derived ariaFoo = X() ? 'true'
//! : 'false'` (or the swapped shape) where `X` is a known boolean
//! signal. The runtime auto-stringifies booleans in attribute position,
//! so `aria-foo={X()}` works the same — the derived is pure
//! boilerplate.
//!
//! Fires only when ALL of: the expr matches the ternary shape; the
//! operand is Prop/State/Data with `boolean` type annotation, or State
//! with a `true`/`false` literal initial; the derived's name has the
//! `aria` prefix; the derived is referenced in a template attribute
//! position somewhere (so it's actually doing boilerplate work).

use std::collections::HashSet;

use crate::analyze::reactivity::extract_idents;
use crate::analyze::symbols::{SignalKind, SymbolTable};
use crate::parse::ast::{SailFile, ScriptBlock};
use crate::{Diagnostic, Severity};

pub fn analyze(file: &SailFile, symbols: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
    let Some(script) = file.script.as_ref() else { return };
    if script.derived.is_empty() { return; }

    let mut attr_refs: HashSet<String> = HashSet::new();
    if let Some(tpl) = file.template.as_ref() {
        super::for_each_attr_expr(tpl, |expr| {
            for n in extract_idents(&expr.raw) { attr_refs.insert(n); }
        });
    }

    for d in &script.derived {
        if !is_aria_name(&d.name) { continue; }
        let Some(operand) = parse_redundant_aria_pattern(&d.expr) else { continue };
        if !is_boolean_signal(&operand, symbols, script) { continue; }
        if !attr_refs.contains(&d.name) { continue; }
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "SAIL/E0104".to_string(),
            message: format!(
                "derived `{}` is a redundant boolean→string wrapper around `{operand}`",
                d.name
            ),
            span: None,
            help: Some(format!(
                "drop the derived and write the binding inline: `aria-…={{{operand}()}}` — the runtime stringifies booleans in attribute position"
            )),
        });
    }
}

fn is_aria_name(name: &str) -> bool {
    let rest = name.strip_prefix("aria").unwrap_or("");
    matches!(rest.chars().next(), Some(c) if c.is_ascii_uppercase() || c == '_')
}

fn is_boolean_signal(name: &str, symbols: &SymbolTable, script: &ScriptBlock) -> bool {
    match symbols.signals.get(name) {
        Some(SignalKind::Prop { ty, .. }) | Some(SignalKind::Data { ty, .. }) => {
            ty.as_deref() == Some("boolean")
        }
        Some(SignalKind::State { ty }) => {
            if ty.as_deref() == Some("boolean") { return true; }
            if ty.is_some() { return false; }
            let Some(s) = script.state.iter().find(|s| s.name == name) else { return false };
            matches!(s.initial.trim(), "true" | "false")
        }
        Some(SignalKind::Derived) | None => false,
    }
}

/// Match `IDENT() ? 'true' : 'false'` or `IDENT() ? 'false' : 'true'`,
/// returning the identifier on success. Whitespace flexible; quote
/// style doesn't have to match between branches.
fn parse_redundant_aria_pattern(expr: &str) -> Option<String> {
    let s = expr.trim();
    let b = s.as_bytes();
    let mut i = 0;
    let id_end = scan_ident(b, i);
    let ident = &s[..id_end];
    if id_end == 0 || !(b[0].is_ascii_alphabetic() || b[0] == b'_' || b[0] == b'$') { return None; }
    i = id_end;
    for c in [b'(', b')', b'?'] {
        i = ws(b, i);
        if i >= b.len() || b[i] != c { return None; }
        i += 1;
    }
    i = ws(b, i);
    let a = read_str(b, &mut i)?;
    i = ws(b, i);
    if i >= b.len() || b[i] != b':' { return None; }
    i += 1;
    i = ws(b, i);
    let bv = read_str(b, &mut i)?;
    if ws(b, i) != b.len() { return None; }
    if !((a == "true" && bv == "false") || (a == "false" && bv == "true")) { return None; }
    Some(ident.to_string())
}

fn scan_ident(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'$') { i += 1; }
    i
}

fn ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && b[i].is_ascii_whitespace() { i += 1; }
    i
}

fn read_str(b: &[u8], i: &mut usize) -> Option<String> {
    let q = *b.get(*i)?;
    if q != b'\'' && q != b'"' { return None; }
    *i += 1;
    let start = *i;
    while *i < b.len() && b[*i] != q {
        *i += if b[*i] == b'\\' && *i + 1 < b.len() { 2 } else { 1 };
    }
    if *i >= b.len() { return None; }
    let s = std::str::from_utf8(&b[start..*i]).ok()?.to_string();
    *i += 1;
    Some(s)
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

    fn errs(d: &[Diagnostic]) -> Vec<&Diagnostic> {
        d.iter().filter(|x| x.code == "SAIL/E0104").collect()
    }

    #[test]
    fn redundant_pattern_with_state_bool_initial_errors() {
        let d = run("<script>\nstate open = false\nderived ariaExpanded = open() ? 'true' : 'false'\n</script><template><div aria-expanded={ariaExpanded()}></div></template>");
        let e = errs(&d);
        assert_eq!(e.len(), 1, "got {d:?}");
        assert!(e[0].message.contains("ariaExpanded"));
        assert!(e[0].help.as_deref().unwrap().contains("open()"));
    }

    #[test]
    fn redundant_pattern_with_typed_prop_errors() {
        let d = run("<script>\nprop disabled: boolean = false\nderived ariaDisabled = disabled() ? 'true' : 'false'\n</script><template><:host aria-disabled={ariaDisabled()} /></template>");
        assert_eq!(errs(&d).len(), 1, "got {d:?}");
    }

    #[test]
    fn swapped_branches_still_match() {
        let d = run("<script>\nstate open = false\nderived ariaHidden = open() ? 'false' : 'true'\n</script><template><div aria-hidden={ariaHidden()}></div></template>");
        assert_eq!(errs(&d).len(), 1, "got {d:?}");
    }

    #[test]
    fn double_quoted_strings_match() {
        let d = run("<script>\nstate open = false\nderived ariaExpanded = open() ? \"true\" : \"false\"\n</script><template><div aria-expanded={ariaExpanded()}></div></template>");
        assert_eq!(errs(&d).len(), 1, "got {d:?}");
    }

    #[test]
    fn non_aria_name_passes() {
        let d = run("<script>\nstate open = false\nderived openLabel = open() ? 'true' : 'false'\n</script><template><div data-x={openLabel()}></div></template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn non_boolean_operand_passes() {
        let d = run("<script>\nstate count = 0\nderived ariaPressed = count() ? 'true' : 'false'\n</script><template><div aria-pressed={ariaPressed()}></div></template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn non_redundant_strings_pass() {
        let d = run("<script>\nstate open = false\nderived ariaLabel = open() ? 'expanded' : 'collapsed'\n</script><template><div aria-label={ariaLabel()}></div></template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn aria_derived_unused_in_attrs_passes() {
        // Pattern matches but the derived isn't wired into an attribute —
        // unused_derived (E0103) covers it; we don't double-flag.
        let d = run("<script>\nstate open = false\nderived ariaExpanded = open() ? 'true' : 'false'\n</script><template>{ariaExpanded()}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }
}
