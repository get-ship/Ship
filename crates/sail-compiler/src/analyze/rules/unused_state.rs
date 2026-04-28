//! unused_state — diagnostic `SAIL/E0102`.
//!
//! Flags `state X = …` declarations whose name is never read anywhere
//! in the file. A reference is any top-level identifier (per
//! [`crate::analyze::reactivity::extract_idents`]) appearing in:
//!
//! * any template expression — text interpolation, attribute
//!   interpolation, two-way binds, event handlers, prop bindings,
//!   `<:host …>` attrs, `if`/`for`/`await` heads, slot props;
//! * any script body — `derived` exprs, `effect` deps and bodies,
//!   `data` exprs, `function` bodies, `mount`/`unmount`,
//!   prop defaults, sibling state initials, form-lifecycle handlers.
//!
//! Setter calls (`x.set(…)`) count as references — `x` appears as a
//! top-level identifier with the member access on the RHS. State that
//!'s only ever written but never read is still dead in the reactive
//! graph and almost always a bug; flag it.
//!
//! Severity::Error — the corpus is currently clean and the pattern is
//! a real bug class (orphaned state from a half-finished port).

use crate::analyze::symbols::SymbolTable;
use crate::parse::ast::SailFile;
use crate::{Diagnostic, DiagnosticFix, FixEdit, Severity};

pub fn analyze(file: &SailFile, _symbols: &SymbolTable, diagnostics: &mut Vec<Diagnostic>) {
    let Some(script) = file.script.as_ref() else { return };
    if script.state.is_empty() { return; }

    let referenced = super::collect_all_references(file);

    for s in &script.state {
        if referenced.contains(&s.name) { continue; }
        let fix = s.span.as_ref().map(|span| DiagnosticFix {
            title: format!("remove unused `state {}`", s.name),
            edits: vec![FixEdit { range: span.clone(), replacement: String::new() }],
        });
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "SAIL/E0102".to_string(),
            message: format!("state `{}` declared but never read", s.name),
            span: s.span.clone(),
            help: Some(format!(
                "remove `state {0} = …` or read it via `{0}()` in a template, derived, effect, or function body",
                s.name
            )),
            fix,
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
        d.iter().filter(|x| x.code == "SAIL/E0102").collect()
    }

    #[test]
    fn declared_but_unread_errors() {
        let d = run("<script>state count = 0</script><template>x</template>");
        let e = errs(&d);
        assert_eq!(e.len(), 1, "got {d:?}");
        assert!(e[0].message.contains("count"));
        assert_eq!(e[0].severity, Severity::Error);
    }

    #[test]
    fn read_in_template_passes() {
        let d = run("<script>state count = 0</script><template>{count()}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_attribute_interpolation_passes() {
        // Attribute-position reads are auto-wrapped at emit time; they
        // still count as a read for liveness.
        let d = run(
            "<script>state open = false</script><template><div hidden={open}></div></template>",
        );
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_derived_passes() {
        let d = run("<script>\nstate count = 0\nderived doubled = count() * 2\n</script><template>{doubled()}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_effect_passes() {
        let d = run("<script>\nstate count = 0\neffect on count { console.log(count()) }\n</script><template>x</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_function_passes() {
        let d = run("<script>\nstate count = 0\nfunction inc() { count.set(count() + 1) }\n</script><template><button onclick={inc}>+</button></template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn write_only_via_setter_passes() {
        // `count.set(0)` includes a top-level `count` ident (member is
        // on RHS of `.`). That's a reference, so we don't flag it.
        let d = run("<script>\nstate count = 0\nfunction reset() { count.set(0) }\n</script><template><button onclick={reset}>r</button></template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_event_handler_passes() {
        let d = run("<script>\nstate count = 0\n</script><template><button onclick={() => console.log(count())}>x</button></template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn read_in_for_iter_passes() {
        let d = run("<script>\nstate items = []\n</script><template>{#for it of items()}<li>{it}</li>{/for}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn one_unused_among_many_flagged() {
        let d = run(
            "<script>\nstate a = 0\nstate b = 0\n</script><template>{a()}</template>",
        );
        let e = errs(&d);
        assert_eq!(e.len(), 1, "got {d:?}");
        assert!(e[0].message.contains("`b`"));
    }

    #[test]
    fn no_state_no_diagnostics() {
        let d = run("<script>\nprop label = ''\n</script><template>{label}</template>");
        assert!(errs(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn fix_is_present_with_delete_edit() {
        let d = run("<script>state count = 0</script><template>x</template>");
        let e = errs(&d);
        assert_eq!(e.len(), 1);
        let fix = e[0].fix.as_ref().expect("fix should be present for E0102");
        assert!(fix.title.contains("count"), "title: {}", fix.title);
        assert_eq!(fix.edits.len(), 1);
        assert_eq!(fix.edits[0].replacement, "", "replacement should be empty (delete)");
    }

    #[test]
    fn fix_range_matches_diagnostic_span() {
        let src = "<script>\nstate count = 0\n</script><template>x</template>";
        let d = run(src);
        let e = errs(&d);
        assert_eq!(e.len(), 1);
        let diag_span = e[0].span.as_ref().expect("span on diagnostic");
        let fix = e[0].fix.as_ref().expect("fix on diagnostic");
        let fix_range = &fix.edits[0].range;
        assert_eq!(fix_range.line, diag_span.line, "fix range line matches span");
        assert_eq!(fix_range.col, diag_span.col, "fix range col matches span");
    }
}
