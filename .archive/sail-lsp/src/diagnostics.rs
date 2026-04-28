//! `sail_compiler::Diagnostic` -> `lsp_types::Diagnostic` conversion.
//!
//! Spans use the compiler's 1-based `(line, col)` and `(end_line,
//! end_col)`; LSP positions are 0-based. We collapse to a single-char
//! span at `(line, col)` when the compiler reports no end position.

use lsp_types::{
    Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range,
};
use sail_compiler::{Diagnostic as SailDiagnostic, Severity, Span};

pub const SOURCE: &str = "sail";

/// Compile `source` and convert each diagnostic into an LSP one.
/// Returns the converted list, or an empty vec if compilation panics
/// at I/O level (the compiler today only fails when the parse is fatal,
/// in which case it returns `Err`; we surface a single LSP diagnostic
/// at the file head so the user sees *something*).
pub fn compile_diagnostics(source: &str, tag: &str) -> Vec<Diagnostic> {
    match sail_compiler::compile_source(source, tag) {
        Ok(result) => result
            .diagnostics
            .iter()
            .map(sail_to_lsp)
            .collect(),
        Err(err) => vec![fatal_diagnostic(&err.to_string())],
    }
}

fn fatal_diagnostic(message: &str) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("SAIL/E0001".into())),
        code_description: None,
        source: Some(SOURCE.to_string()),
        message: format!("compile failed: {message}"),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn sail_to_lsp(d: &SailDiagnostic) -> Diagnostic {
    let range = d
        .span
        .as_ref()
        .map(span_to_range)
        .unwrap_or_else(zero_range);
    let mut message = d.message.clone();
    if let Some(help) = &d.help {
        message.push_str("\nhelp: ");
        message.push_str(help);
    }
    // Attach the structured fix as `data` so the codeAction handler
    // can read it back without re-running analysis.
    let data = d
        .fix
        .as_ref()
        .map(|f| serde_json::to_value(f).unwrap_or(serde_json::Value::Null));
    Diagnostic {
        range,
        severity: Some(map_severity(d.severity)),
        code: Some(NumberOrString::String(d.code.clone())),
        code_description: None,
        source: Some(SOURCE.to_string()),
        message,
        related_information: None,
        tags: None,
        data,
    }
}

fn span_to_range(s: &Span) -> Range {
    let start = Position {
        line: s.line.saturating_sub(1),
        character: s.col.saturating_sub(1),
    };
    let end = Position {
        line: s.end_line.saturating_sub(1),
        character: s.end_col.saturating_sub(1),
    };
    Range { start, end }
}

fn zero_range() -> Range {
    Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 0 },
    }
}

fn map_severity(s: Severity) -> DiagnosticSeverity {
    match s {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_produces_no_diagnostics() {
        let diags = compile_diagnostics("", "sail-x");
        assert!(diags.is_empty(), "{diags:?}");
    }

    #[test]
    fn fatal_diagnostic_is_zero_position_error() {
        let d = fatal_diagnostic("boom");
        assert_eq!(d.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(d.range.start.line, 0);
        assert_eq!(d.range.start.character, 0);
        assert!(d.message.contains("boom"));
    }

    #[test]
    fn span_round_trips_to_zero_based() {
        let s = Span {
            file: "x".into(),
            line: 4,
            col: 12,
            end_line: 4,
            end_col: 18,
        };
        let r = span_to_range(&s);
        assert_eq!(r.start.line, 3);
        assert_eq!(r.start.character, 11);
        assert_eq!(r.end.line, 3);
        assert_eq!(r.end.character, 17);
    }

    #[test]
    fn real_diagnostic_anchors_at_offending_line() {
        // End-to-end: a real fixture compiles, the R1 signal-call rule
        // fires, and the converted LSP range points at the
        // interpolation on the right line — not file 0:0.
        let source =
            "<script>\nstate count = 0\n</script>\n<template><span>{count}</span></template>\n";
        let diags = compile_diagnostics(source, "sail-counter");
        let r1 = diags
            .iter()
            .find(|d| matches!(&d.code, Some(NumberOrString::String(c)) if c == "SAIL/E0101"))
            .expect("expected R1 diagnostic");
        // 1-based line 4 → 0-based 3; both ends sit on the same line.
        assert_eq!(r1.range.start.line, 3, "{:?}", r1.range);
        assert!(r1.range.start.character > 0, "{:?}", r1.range);
        assert_eq!(r1.range.end.line, 3, "{:?}", r1.range);
        assert!(r1.range.end.character > r1.range.start.character, "{:?}", r1.range);
    }
}
