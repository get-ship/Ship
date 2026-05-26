//! `sui-controller-no-ambient-authority` — data flows through `ship`, not
//! raw network/eval.
//!
//! Flags, anywhere inside a `*.controller.ts` file, use of ambient authority:
//! - `fetch(...)`, `navigator.sendBeacon(...)`
//! - `new XMLHttpRequest()`, `new WebSocket(...)`, `new EventSource(...)`
//! - `eval(...)`, dynamic `import(...)`
//! - `document.cookie`
//!
//! Rationale (ADR unified-deck-router's sibling, the secure-execution model):
//! view code is untrusted (agent-authored) and runs on the highest-privilege
//! surface. All I/O must go through the `ship` SDK (the only sanctioned,
//! policy-gated path); ambient network/eval is exactly what the Deck SES
//! Compartment denies at runtime. This rule is the author-time mirror.
//!
//! **Severity: Warning (for now).** First-party controllers still call
//! `fetch('/api/v1/…')` against the REST API; those migrate to the Connect
//! client during REST removal. Once that lands, promote this to Error so the
//! build gate enforces it (the gate already only hard-fails on Error).

use std::path::{Path, PathBuf};

use crate::report::{Finding, Severity};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

pub const RULE_ID: &str = "sui-controller-no-ambient-authority";

const BANNED_CALLS: &[&str] = &["fetch", "eval"];
const BANNED_CTORS: &[&str] = &["XMLHttpRequest", "WebSocket", "EventSource"];

pub fn check(path: &Path, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::ts()).parse();
    if !ret.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = AmbientVisitor {
        source,
        file: path.to_path_buf(),
        findings: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    visitor.findings
}

struct AmbientVisitor<'src> {
    source: &'src str,
    file: PathBuf,
    findings: Vec<Finding>,
}

impl AmbientVisitor<'_> {
    fn report(&mut self, span: Span, what: &str) {
        let (line, col) = byte_offset_to_line_col(self.source, span.start as usize);
        self.findings.push(Finding {
            file: self.file.clone(),
            line,
            col,
            rule: RULE_ID,
            severity: Severity::Warn,
            message: format!(
                "controllers must not use ambient authority (`{what}`). All I/O goes \
                 through the `ship` SDK — the only sanctioned, policy-gated path. Raw \
                 network/eval is denied at runtime by the Deck secure-execution Compartment."
            ),
            suggest: Some("use a <data source=\"ship.<domain>.<op>()\"> subscription or a ship.<domain>.<op>() call".to_string()),
        });
    }
}

impl<'a> Visit<'a> for AmbientVisitor<'_> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        match &expr.callee {
            // bare `fetch(...)` / `eval(...)`
            Expression::Identifier(id) if BANNED_CALLS.contains(&id.name.as_str()) => {
                self.report(expr.span, id.name.as_str())
            }
            // `navigator.sendBeacon(...)` / `window.fetch(...)`
            Expression::StaticMemberExpression(m) => {
                let p = m.property.name.as_str();
                if p == "sendBeacon" || p == "fetch" {
                    self.report(expr.span, p)
                }
            }
            _ => {}
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e)
            }
        }
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        // dynamic import(...)
        self.report(expr.span, "import()");
        self.visit_expression(&expr.source);
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        if let Expression::Identifier(id) = &expr.callee
            && BANNED_CTORS.contains(&id.name.as_str())
        {
            self.report(expr.span, id.name.as_str());
        }
        self.visit_expression(&expr.callee);
        for arg in &expr.arguments {
            if let Some(e) = arg.as_expression() {
                self.visit_expression(e)
            }
        }
    }

    fn visit_static_member_expression(&mut self, m: &StaticMemberExpression<'a>) {
        // document.cookie
        if m.property.name.as_str() == "cookie" {
            if let Expression::Identifier(obj) = &m.object
                && obj.name.as_str() == "document"
            {
                self.report(m.span, "document.cookie");
            }
        }
        self.visit_expression(&m.object);
    }
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

    fn n(src: &str) -> usize {
        check(Path::new("x.controller.ts"), src).len()
    }

    #[test]
    fn flags_fetch() {
        assert_eq!(n("fetch('/api/v1/x')"), 1);
    }

    #[test]
    fn flags_new_xhr_and_websocket() {
        assert_eq!(n("const x = new XMLHttpRequest()"), 1);
        assert_eq!(n("const w = new WebSocket('wss://x')"), 1);
    }

    #[test]
    fn flags_dynamic_import_and_eval() {
        assert_eq!(n("const m = import('./x.js')"), 1);
        assert_eq!(n("eval('2+2')"), 1);
    }

    #[test]
    fn flags_document_cookie() {
        assert_eq!(n("const c = document.cookie"), 1);
    }

    #[test]
    fn allows_ship_calls() {
        assert_eq!(n("ship.git.overview()"), 0);
    }

    #[test]
    fn warning_severity_does_not_hard_fail() {
        let f = check(Path::new("x.controller.ts"), "fetch('/x')");
        assert_eq!(f[0].severity, Severity::Warn);
    }
}
