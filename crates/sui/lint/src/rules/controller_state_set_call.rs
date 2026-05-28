//! `sui-state-direct-assign` — controllers do not assign to `state.<key>`
//! directly; call `.set(…)` on the signal instead.
//!
//! From #178: two signal-write idioms lived in the codebase:
//! - `state.lineColor.set(v)` (sparkline.controller.ts)
//! - `state.caughtError = err` (sui-error-boundary.controller.ts)
//!
//! Both work at runtime — the proxy's `set` trap routes the assignment
//! through `signal.set(v)`. But two visible shapes for one underlying
//! operation is exactly the "two idioms in one codebase" trap an agent
//! falls into by copying the wrong exemplar. The proxy magic is for the
//! framework's benefit, not the author's — the call form is uniform with
//! every other signal in SUI (`x.set(v)` whether `x` came from a scope
//! factory, a `<state>` element, or a free `signal()`).
//!
//! Heuristic: flag `state.<key> = …` AssignmentExpressions where the LHS
//! object is the identifier `state` (the convention for the mount
//! callback's first parameter). Nested receivers (`state.foo.bar = …`)
//! are out of scope — those are usually field mutations on a non-signal
//! value, a different concern. Authors who name the state parameter
//! something other than `state` bypass the lint, which is acceptable: the
//! convention is the contract, and the rare deviation is an explicit
//! choice the lint shouldn't second-guess.

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sui-state-direct-assign";

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }

    let mut visitor = DirectAssignVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct DirectAssignVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl<'src> DirectAssignVisitor<'src> {
    fn report(&mut self, span: Span, key: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message: format!(
                "controllers do not assign `state.{key} = …` directly. Two visible \
                 shapes for one operation is the wrong-exemplar trap; pick the call \
                 form everywhere."
            ),
            suggest: Some(format!(
                "Use `state.{key}.set(…)` — same runtime, one shape across views, \
                 components, and free signals."
            )),
        });
    }
}

impl<'a, 'src> Visit<'a> for DirectAssignVisitor<'src> {
    fn visit_assignment_expression(&mut self, expr: &AssignmentExpression<'a>) {
        if let Some(key) = direct_state_member(&expr.left) {
            self.report(expr.span, key);
        }
        oxc_ast_visit::walk::walk_assignment_target(self, &expr.left);
        self.visit_expression(&expr.right);
    }

    fn visit_update_expression(&mut self, expr: &UpdateExpression<'a>) {
        // `state.count++` / `state.count--` route through the same set-trap
        // path; they're the same shape with a different syntax.
        if let SimpleAssignmentTarget::StaticMemberExpression(m) = &expr.argument
            && let Expression::Identifier(id) = &m.object
            && id.name.as_str() == "state"
        {
            self.report(expr.span, m.property.name.as_str());
        }
        oxc_ast_visit::walk::walk_simple_assignment_target(self, &expr.argument);
    }
}

/// Extract `<key>` from a `state.<key>` `AssignmentTarget` (lhs of `=`).
/// Returns `None` for other shapes (nested member, computed access, etc.).
fn direct_state_member<'a>(target: &'a AssignmentTarget<'a>) -> Option<&'a str> {
    let inner = target.as_member_expression()?;
    let MemberExpression::StaticMemberExpression(m) = inner else { return None };
    let Expression::Identifier(id) = &m.object else { return None };
    if id.name.as_str() != "state" {
        return None;
    }
    Some(m.property.name.as_str())
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
    fn flags_direct_assign() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                state.caughtError = err
                return () => {}
              },
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, RULE_ID);
        assert!(findings[0].message.contains("caughtError"));
        assert!(findings[0].suggest.as_ref().unwrap().contains("state.caughtError.set"));
    }

    #[test]
    fn flags_increment() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                state.count++
                state.count--
                return () => {}
              },
            })
        "#;
        let findings = check(Path::new("foo.controller.ts"), src);
        assert_eq!(findings.len(), 2);
    }

    #[test]
    fn does_not_flag_set_call() {
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                state.label.set('hi')
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn does_not_flag_nested_assignment() {
        // state.foo.bar = … is a field mutation on a non-signal value, not
        // a direct-signal-set. Out of scope; a different lint (or human review)
        // owns mutations like that.
        let src = r#"
            export default controller<Scope>({
              mount(state, host) {
                state.foo.bar = 42
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }

    #[test]
    fn does_not_flag_non_state_identifier() {
        // The lint is scoped to the conventional `state` name. Authors who
        // rename it bypass the lint — explicit opt-out is fine.
        let src = r#"
            export default controller<Scope>({
              mount(s, host) {
                s.caughtError = err
                return () => {}
              },
            })
        "#;
        assert_eq!(check(Path::new("foo.controller.ts"), src).len(), 0);
    }
}
