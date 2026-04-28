//! unused_derived — diagnostic `SAIL/E0103`.
//!
//! Sister rule to [`super::unused_state`]. Flags `derived X = …`
//! declarations whose name is never read anywhere in the file.
//!
//! References are collected the same way unused_state collects them
//! (see [`super::collect_all_references`]). A derived's own expression
//! body counts toward references for *other* signals it reads, but a
//! self-reference inside the same derived's body would just mean the
//! file is malformed (derives can't read themselves) — we don't try to
//! distinguish, the heuristic is simply: name appears as a top-level
//! identifier somewhere → live.
//!
//! Severity::Error — same justification as unused_state. A derived
//! computed but never observed is dead reactivity; if you find a real
//! use case where this fires legitimately, suppress it by inlining the
//! computation at the call site.

use crate::analyze::symbols::SymbolTable;
use crate::parse::ast::SailFile;
use crate::{Diagnostic, Severity};

pub fn analyze(file: &SailFile, _symbols: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
    let Some(script) = file.script.as_ref() else { return };
    if script.derived.is_empty() { return; }

    let referenced = super::collect_all_references(file);

    for d in &script.derived {
        if referenced.contains(&d.name) { continue; }
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "SAIL/E0103".to_string(),
            message: format!("derived `{}` declared but never read", d.name),
            span: None,
            help: Some(format!(
                "remove `derived {0} = …` or read it via `{0}()` in a template, sibling derived, effect, or function body",
                d.name
            )),
        });
    }
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
        d.iter().filter(|x| x.code == "SAIL/E0103").collect()
    }

    #[test]
    fn declared_but_unread_errors() {
        let d = run("<script>\nstate count = 0\nderived doubled = count() * 2\n</script><template>{count()}</template>");
        let e = errs(&d);
        assert_eq!(e.len(), 1, "got {d:?}");
        assert!(e[0].message.contains("doubled"));
        assert_eq!(e[0].severity, Severity::Error);
    }

    #[test]
    fn read_in_text_passes() {
        let d = run("<script>\nstate count = 0\nderived doubled = count() * 2\n</script><template>{doubled()}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_attribute_passes() {
        let d = run("<script>\nstate open = false\nderived ariaExpanded = open() ? 'true' : 'false'\n</script><template><div aria-expanded={ariaExpanded()}></div></template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_sibling_derived_passes() {
        let d = run("<script>\nstate count = 0\nderived a = count() * 2\nderived b = a() + 1\n</script><template>{b()}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_effect_passes() {
        let d = run("<script>\nstate count = 0\nderived doubled = count() * 2\neffect on doubled { console.log(doubled()) }\n</script><template>x</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn one_unused_among_many_flagged() {
        let d = run(
            "<script>\nstate count = 0\nderived used = count() * 2\nderived dead = count() + 1\n</script><template>{used()}</template>",
        );
        let e = errs(&d);
        assert_eq!(e.len(), 1, "got {d:?}");
        assert!(e[0].message.contains("`dead`"));
    }

    #[test]
    fn no_derived_no_diagnostics() {
        let d = run("<script>\nstate count = 0\n</script><template>{count()}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }
}
