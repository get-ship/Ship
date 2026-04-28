//! `textDocument/codeAction` — convert each diagnostic's structured
//! `DiagnosticFix` (round-tripped through `Diagnostic.data` by
//! `diagnostics::sail_to_lsp`) into an LSP `CodeAction` with a
//! `WorkspaceEdit`. Diagnostics with no fix produce no action; the
//! filter is the diagnostic's range overlapping the request range.

use std::collections::HashMap;

use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, Diagnostic,
    OneOf, Position, Range, TextEdit, Url, WorkspaceEdit,
};
use sail_compiler::DiagnosticFix;

/// Build the response for a `textDocument/codeAction` request. Walks
/// the diagnostics in the request payload, decodes each `data` payload
/// as a `DiagnosticFix`, and emits a quick-fix CodeAction per fix.
pub fn code_actions(params: &CodeActionParams) -> Vec<CodeActionOrCommand> {
    let uri = &params.text_document.uri;
    let req_range = params.range;

    params
        .context
        .diagnostics
        .iter()
        .filter(|d| ranges_overlap(d.range, req_range))
        .filter_map(|d| build_action(uri, d))
        .collect()
}

fn build_action(uri: &Url, diag: &Diagnostic) -> Option<CodeActionOrCommand> {
    let data = diag.data.as_ref()?;
    let fix: DiagnosticFix = serde_json::from_value(data.clone()).ok()?;
    let edits: Vec<TextEdit> = fix
        .edits
        .iter()
        .map(|e| TextEdit {
            range: span_to_range(&e.range),
            new_text: e.replacement.clone(),
        })
        .collect();
    if edits.is_empty() {
        return None;
    }
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), edits);
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: fix.title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    }))
}

fn span_to_range(s: &sail_compiler::Span) -> Range {
    Range {
        start: Position {
            line: s.line.saturating_sub(1),
            character: s.col.saturating_sub(1),
        },
        end: Position {
            line: s.end_line.saturating_sub(1),
            character: s.end_col.saturating_sub(1),
        },
    }
}

fn ranges_overlap(a: Range, b: Range) -> bool {
    !(a.end < b.start || b.end < a.start)
}

#[allow(dead_code)]
fn _force_response_type() -> Vec<CodeActionOrCommand> {
    Vec::new()
}

// `OneOf` isn't used directly but the LSP signature requires the type
// to be in scope for the matching server response shape.
#[allow(dead_code)]
type _OneOf<A, B> = OneOf<A, B>;

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{CodeActionContext, NumberOrString, TextDocumentIdentifier};
    use sail_compiler::{DiagnosticFix, FixEdit, Span};

    fn pos(line: u32, ch: u32) -> Position {
        Position { line, character: ch }
    }
    fn range(sl: u32, sc: u32, el: u32, ec: u32) -> Range {
        Range { start: pos(sl, sc), end: pos(el, ec) }
    }
    fn url() -> Url { Url::parse("file:///x/widget.set").unwrap() }

    fn diag_with_fix(code: &str, line: u32, fix: DiagnosticFix) -> Diagnostic {
        Diagnostic {
            range: range(line, 0, line, 10),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String(code.into())),
            source: Some("sail".into()),
            message: "test".into(),
            data: Some(serde_json::to_value(&fix).unwrap()),
            ..Default::default()
        }
    }

    #[test]
    fn diagnostic_with_fix_yields_quickfix_action() {
        let fix = DiagnosticFix {
            title: "add ()".into(),
            edits: vec![FixEdit {
                range: Span {
                    file: "widget.set".into(),
                    line: 5,
                    col: 10,
                    end_line: 5,
                    end_col: 15,
                },
                replacement: "count()".into(),
            }],
        };
        let d = diag_with_fix("SAIL/E0101", 4, fix);
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: url() },
            range: range(4, 0, 4, 20),
            context: CodeActionContext {
                diagnostics: vec![d],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        let actions = code_actions(&params);
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            CodeActionOrCommand::CodeAction(a) => {
                assert_eq!(a.title, "add ()");
                assert_eq!(a.kind, Some(CodeActionKind::QUICKFIX));
                let edit = a.edit.as_ref().expect("edit");
                let changes = edit.changes.as_ref().expect("changes");
                let text_edits = changes.get(&url()).expect("uri");
                assert_eq!(text_edits.len(), 1);
                assert_eq!(text_edits[0].new_text, "count()");
            }
            _ => panic!("expected CodeAction"),
        }
    }

    #[test]
    fn diagnostic_without_fix_yields_no_action() {
        let d = Diagnostic {
            range: range(0, 0, 0, 10),
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            data: None,
            ..Default::default()
        };
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: url() },
            range: range(0, 0, 0, 20),
            context: CodeActionContext {
                diagnostics: vec![d],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        assert!(code_actions(&params).is_empty());
    }

    #[test]
    fn diagnostic_outside_request_range_skipped() {
        let fix = DiagnosticFix {
            title: "x".into(),
            edits: vec![FixEdit {
                range: Span { file: "x".into(), line: 1, col: 1, end_line: 1, end_col: 5 },
                replacement: "y".into(),
            }],
        };
        let d = diag_with_fix("SAIL/E0101", 50, fix);
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri: url() },
            range: range(0, 0, 1, 0),
            context: CodeActionContext {
                diagnostics: vec![d],
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        assert!(code_actions(&params).is_empty());
    }
}
