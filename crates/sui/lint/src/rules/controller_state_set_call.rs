//! `sui-state-set-call` — controllers do not write through `state.<key>.set(…)`.
//!
//! After #179 (proxy unwrap unified — `state.x` is the value everywhere, not
//! a callable signal), `state.x.set(…)` calls `.set` on the unwrapped value
//! and errors at runtime. The canonical write form is the assignment-style
//! `state.x = …` — the proxy's set-trap routes it through `signal.set(v)`.
//!
//! The escape hatch for the rare cases that genuinely need the signal
//! (updater function, handing a `Signal<T>` to a callback) is
//! `state.$signals.x.set(updater)`. The lint specifically allows that
//! shape — only `state.<key>.set(...)` (one identifier between `state.`
//! and `.set`) is flagged.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sui-state-set-call";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = SetCallVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct SetCallVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl<'src> SetCallVisitor<'src> {
    fn report(&mut self, span: Span, key: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Error,
            message: format!(
                "`state.{key}.set(…)` calls `.set` on the unwrapped value, not the \
                 signal. Write via the assignment form — the proxy routes it through \
                 `signal.set` for you."
            ),
            suggest: Some(format!(
                "Use `state.{key} = …` for the value write. For the rare updater-\
                 function case, use `state.$signals.{key}.set(updater)`."
            )),
        });
    }
}

impl<'a, 'src> Visit<'a> for SetCallVisitor<'src> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        if let Some(key) = state_member_set_call(&expr.callee) {
            self.report(expr.span, key);
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e);
            }
        }
    }
}

/// Match the callee shape `state.<key>.set` (one identifier between the
/// `state.` and `.set`). Returns the key on match.
///
/// Specifically allows the `state.$signals.<key>.set` escape hatch by
/// rejecting any nesting depth greater than `state.<key>.set` — see the
/// `does_not_flag_signals_escape_hatch` test for what slips through.
fn state_member_set_call<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let outer = match expr {
        Expression::StaticMemberExpression(m) => m,
        _ => return None,
    };
    if outer.property.name.as_str() != "set" {
        return None;
    }
    let inner = match &outer.object {
        Expression::StaticMemberExpression(m) => m,
        _ => return None,
    };
    let Expression::Identifier(id) = &inner.object else { return None };
    if id.name.as_str() != "state" {
        return None;
    }
    // Reject `state.$signals.x.set` — that's the explicit escape hatch.
    let key = inner.property.name.as_str();
    if key == "$signals" {
        return None;
    }
    Some(key)
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut line = 1usize;
    let mut col = 1usize;
    let cap = offset.min(bytes.len());
    let mut i = 0;
    while i < cap {
        if bytes[i] == b'\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
        i += 1;
    }
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn flags_state_x_set() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                state.count.set(5)
                return () => {}
              },
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, RULE_ID);
        assert!(findings[0].message.contains("count"));
        assert!(findings[0].suggest.as_ref().unwrap().contains("state.count = …"));
    }

    #[test]
    fn does_not_flag_direct_assign() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                state.count = 5
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn does_not_flag_signals_escape_hatch() {
        // state.$signals.x.set(updater) is the explicit signal-level access
        // for updater-function writes — passes the lint.
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                state.$signals.count.set((prev) => prev + 1)
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn does_not_flag_set_call_on_non_state() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                const local = signal(0)
                local.set(5)
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }
}
