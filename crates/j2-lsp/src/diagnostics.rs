//! `lint::Finding` -> `lsp_types::Diagnostic` conversion.
//!
//! v1 emits a single-character span at `(line, col)`. Structural ranges
//! (whole-tag, whole-attr, whole-expression) are explicit v2 work; the
//! lint crate would have to surface span end positions first.

use lint::report::{Finding, Severity};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

pub const SOURCE: &str = "ship-lint";

/// Convert a slice of findings into LSP diagnostics, in the same order.
pub fn findings_to_diagnostics(findings: &[Finding]) -> Vec<Diagnostic> {
    findings.iter().map(finding_to_diagnostic).collect()
}

/// Convert one finding.
pub fn finding_to_diagnostic(f: &Finding) -> Diagnostic {
    // LSP positions are 0-based; lint findings are 1-based.
    let line = f.line.saturating_sub(1) as u32;
    let col = f.col.saturating_sub(1) as u32;
    let start = Position { line, character: col };
    let end = Position {
        line,
        character: col.saturating_add(1),
    };
    let mut message = f.message.clone();
    if let Some(suggest) = &f.suggest {
        message.push_str("\nsuggest: ");
        message.push_str(suggest);
    }
    Diagnostic {
        range: Range { start, end },
        severity: Some(map_severity(f.severity)),
        code: Some(NumberOrString::String(f.rule.to_string())),
        code_description: None,
        source: Some(SOURCE.to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

fn map_severity(s: Severity) -> DiagnosticSeverity {
    match s {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warn => DiagnosticSeverity::WARNING,
    }
}
