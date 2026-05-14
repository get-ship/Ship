//! Expression-aware LSP intelligence — hover, goto-definition, and
//! completion for `{expr}` regions inside `.sui` files.
//!
//! Strategy is symbol-level:
//!   1. Locate the `{...}` region containing the cursor.
//!   2. Parse the inner with `sui_compiler::parse::expr::parse_expression`.
//!   3. Walk the AST to find the innermost node at the cursor offset.
//!   4. Resolve the symbol against a `SymbolTable` built from the
//!      paired controller, plus any `<data>` / `<prop>` directives in
//!      the .sui itself.
//!
//! No TypeScript type inference. Cross-file inference beyond the
//! `.sui` ↔ `.controller.ts` pair is out of scope (see
//! `crates/sui/sui-compiler/EXPRESSIONS.md → LSP intelligence`).

use std::path::Path;

use lsp_types::{
    CompletionItem, CompletionItemKind, GotoDefinitionResponse, Hover, HoverContents,
    Location, MarkedString, Position, Range, Url,
};

use sui_compiler::check::controller_parse::{read_paired_controller, ParsedController};
use sui_compiler::check::symbol_table::{ScopeKind, SymbolTable};
use sui_compiler::ir::Program;
use sui_compiler::parse::expr::{parse_expression_with_arrows, Ast, AstKind};

/// Convert a 0-based (line, character) Position to a UTF-8 byte offset
/// in `text`. Saturates at the document end if the position is past it.
pub fn position_to_offset(text: &str, pos: Position) -> usize {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, c) in text.char_indices() {
        if line == pos.line && col == pos.character {
            return i;
        }
        if c == '\n' {
            if line == pos.line {
                return i;
            }
            line += 1;
            col = 0;
        } else {
            col += c.len_utf16() as u32;
        }
    }
    text.len()
}

/// Inverse of [`position_to_offset`]. Linear scan — fine for the
/// short documents the LSP handles per request.
pub fn offset_to_position(text: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;
    let clamp = offset.min(text.len());
    for (i, c) in text.char_indices() {
        if i >= clamp {
            return Position { line, character: col };
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += c.len_utf16() as u32;
        }
    }
    Position { line, character: col }
}

/// Resolved expression context at a cursor — the inner AST, the
/// innermost node under the cursor, plus the symbol table.
pub struct ExprContext<'a> {
    pub ast: Ast,
    pub source_path: &'a Path,
    pub controller: Option<ParsedController>,
    pub symbols: SymbolTable,
    pub cursor_byte: usize,
}

impl<'a> ExprContext<'a> {
    /// Innermost AST node whose span covers the cursor.
    pub fn node_at_cursor(&self) -> Option<&Ast> {
        self.ast.innermost_at(self.cursor_byte)
    }

    /// Innermost member chain (`head.prop`) ending at the cursor — used
    /// to resolve `scope.X` and friends.
    pub fn member_at_cursor(&self) -> Option<(&str, Vec<&str>)> {
        let node = self.node_at_cursor()?;
        match &node.kind {
            AstKind::Ident(name) => Some((name.as_str(), Vec::new())),
            AstKind::Member { .. } => node.member_chain(),
            _ => None,
        }
    }
}

/// Build a fresh `ExprContext` if the cursor is inside an `{expr}`
/// region of the .sui document. Returns `None` for cursors in plain
/// HTML / between elements.
pub fn at_cursor<'a>(
    source: &'a str,
    source_path: &'a Path,
    cursor_byte: usize,
) -> Option<ExprContext<'a>> {
    let (inner_start, inner_end) = enclosing_brace_region(source, cursor_byte)?;
    let inner = &source[inner_start..inner_end];
    // Block-header prefixes like `#if `, `#for x of `, `:case ` etc.
    // need stripping so we hand the parser a pure expression. Naive but
    // good enough for v1.
    let (offset_after_prefix, expr_src) = strip_block_prefix(inner);
    let absolute_base = inner_start + offset_after_prefix;
    let ast = parse_expression_with_arrows(expr_src, absolute_base).ok()?;

    let controller = read_paired_controller(Some(source_path));
    let program = parse_program_safe(source);
    let symbols = SymbolTable::build(&program, controller.as_ref());
    Some(ExprContext {
        ast,
        source_path,
        controller,
        symbols,
        cursor_byte,
    })
}

fn parse_program_safe(source: &str) -> Program {
    sui_compiler::parse::parse(source).unwrap_or_else(|_| Program {
        data: vec![],
        props: vec![],
        styles: String::new(),
        template: vec![],
        source_len: source.len(),
        span: sui_compiler::diagnostics::Span::point(0),
    })
}

/// Locate the `{...}` region whose interior contains `byte`. Returns
/// `(inner_start, inner_end)` — both relative to `source`. The braces
/// themselves are excluded from the range. Brace-counting is
/// string-aware, matching `parse::template::tokenize::find_matching_close`.
fn enclosing_brace_region(source: &str, byte: usize) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    if byte > bytes.len() {
        return None;
    }
    // Walk back to find a `{` whose matching `}` comes after `byte`.
    let mut i = byte;
    loop {
        if i == 0 {
            return None;
        }
        i -= 1;
        let c = bytes[i];
        if c == b'}' {
            // Skip past matching open: walk back another balanced pair.
            let opener = walk_back_to_opener(bytes, i)?;
            i = opener;
            continue;
        }
        if c == b'{' {
            // Confirm the matching close is past `byte`.
            let close = walk_forward_to_closer(bytes, i)?;
            if close >= byte {
                return Some((i + 1, close));
            }
            // Not the enclosing brace; keep walking back.
        }
    }
}

fn walk_forward_to_closer(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut q: Option<u8> = None;
    let mut i = open;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(quote) = q {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == quote {
                q = None;
            }
            i += 1;
            continue;
        }
        if c == b'"' || c == b'\'' || c == b'`' {
            q = Some(c);
            i += 1;
            continue;
        }
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn walk_back_to_opener(bytes: &[u8], close: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = close + 1;
    while i > 0 {
        i -= 1;
        let c = bytes[i];
        if c == b'}' {
            depth += 1;
        } else if c == b'{' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Strip `#if `, `#for X of `, `:case `, `:else if`, etc. prefixes.
/// Returns `(offset, remaining)` — `offset` is the byte advance into
/// the inner so absolute spans line up.
fn strip_block_prefix(inner: &str) -> (usize, &str) {
    let trimmed_start = inner.len() - inner.trim_start().len();
    let rest = &inner[trimmed_start..];
    for prefix in &["#if", "#match", "#while-loading", "#while-error", ":else if", ":case"] {
        if let Some(after) = rest.strip_prefix(prefix) {
            let body_start = inner.len() - after.len();
            // Skip whitespace after the keyword.
            let body_offset = body_start + (after.len() - after.trim_start().len());
            return (body_offset, &inner[body_offset..]);
        }
    }
    if let Some(after) = rest.strip_prefix("#for") {
        // `#for X of items …` — return the iterable expr only.
        if let Some(of_idx) = after.find(" of ") {
            let head_offset = inner.len() - after.len() + of_idx + 4;
            return (head_offset, &inner[head_offset..]);
        }
    }
    (trimmed_start, rest)
}

// ── Hover ────────────────────────────────────────────────────────

pub fn hover(
    source: &str,
    source_path: &Path,
    cursor_byte: usize,
) -> Option<Hover> {
    let ctx = at_cursor(source, source_path, cursor_byte)?;
    let (head, tail) = ctx.member_at_cursor()?;
    let summary = describe_symbol(head, &tail, &ctx.symbols)?;
    Some(Hover {
        contents: HoverContents::Scalar(MarkedString::String(summary)),
        range: ctx.node_at_cursor().map(|n| Range {
            start: offset_to_position(source, n.span.start),
            end: offset_to_position(source, n.span.end),
        }),
    })
}

fn describe_symbol(
    head: &str,
    tail: &[&str],
    symbols: &SymbolTable,
) -> Option<String> {
    if tail.is_empty() {
        // Bare identifier — figure out which namespace it lives in.
        if let Some(k) = symbols.kind_of_scope(head) {
            return Some(format!("`{head}` — scope {} (paired controller)", k.label()));
        }
        if symbols.derived.contains_key(head) {
            return Some(format!("`{head}` — derived getter (paired controller)"));
        }
        if symbols.on.contains_key(head) {
            return Some(format!("`{head}` — `on:` action (paired controller)"));
        }
        if symbols.data.contains_key(head) {
            return Some(format!("`{head}` — `<data>` binding"));
        }
        if symbols.props.contains_key(head) {
            return Some(format!("`{head}` — `<prop>` binding"));
        }
        if symbols.use_keys.contains_key(head) {
            return Some(format!("`{head}` — `use:` key (paired controller)"));
        }
        return None;
    }
    let first = tail[0];
    match head {
        "scope" => symbols
            .kind_of_scope(first)
            .map(|k| format!("`scope.{first}` — {} (paired controller)", k.label())),
        "derived" => symbols
            .derived
            .get(first)
            .map(|_| format!("`derived.{first}` — derived getter")),
        "data" => symbols
            .data
            .get(first)
            .map(|_| format!("`data.{first}` — `<data>` binding")),
        "props" => symbols
            .props
            .get(first)
            .map(|_| format!("`props.{first}` — `<prop>` binding")),
        _ => None,
    }
}

// ── Goto-definition ──────────────────────────────────────────────

pub fn goto_definition(
    source: &str,
    source_path: &Path,
    cursor_byte: usize,
) -> Option<GotoDefinitionResponse> {
    let ctx = at_cursor(source, source_path, cursor_byte)?;
    let (head, tail) = ctx.member_at_cursor()?;
    let (key_name, namespace) = if tail.is_empty() {
        (head, head)
    } else {
        (tail[0], head)
    };
    let (path, offset) = locate_definition(namespace, key_name, &ctx)?;
    let url = Url::from_file_path(&path).ok()?;
    let ctrl_text = std::fs::read_to_string(&path).ok()?;
    let pos = offset_to_position(&ctrl_text, offset);
    Some(GotoDefinitionResponse::Scalar(Location {
        uri: url,
        range: Range { start: pos, end: pos },
    }))
}

fn locate_definition(
    namespace: &str,
    key: &str,
    ctx: &ExprContext<'_>,
) -> Option<(std::path::PathBuf, usize)> {
    let c = ctx.controller.as_ref()?;
    let path = c.path.clone();
    let offset = match namespace {
        "scope" => ctx.symbols.scope.get(key).map(|e| e.offset)?,
        "derived" => ctx.symbols.derived.get(key).map(|e| e.offset)?,
        "data" => return None, // data lives in the .sui itself; future work
        "props" => return None,
        _ => {
            // Bare ident — flat-bound from one of the namespaces.
            ctx.symbols
                .scope
                .get(key)
                .map(|e| e.offset)
                .or_else(|| ctx.symbols.derived.get(key).map(|e| e.offset))
                .or_else(|| ctx.symbols.on.get(key).map(|e| e.offset))
                .or_else(|| ctx.symbols.use_keys.get(key).map(|e| e.offset))?
        }
    };
    Some((path, offset))
}

// ── Completion ───────────────────────────────────────────────────

pub fn completion(
    source: &str,
    source_path: &Path,
    cursor_byte: usize,
) -> Vec<CompletionItem> {
    // Completion has to work mid-typing — `scope.` typically doesn't
    // parse — so we resolve the symbol table without requiring a
    // successful AST parse, and only consult the AST for richer cases.
    let bytes = source.as_bytes();
    if !inside_brace(bytes, cursor_byte) {
        return Vec::new();
    }
    let symbols = build_symbols(source, source_path);
    if cursor_byte >= 1 && bytes[cursor_byte - 1] == b'.' {
        let head_end = cursor_byte - 1;
        let head_start = ident_start_back(bytes, head_end);
        let head = &source[head_start..head_end];
        return namespace_completions(head, &symbols);
    }
    root_completions(&symbols)
}

/// Cheap variant of `at_cursor` that only checks brace-enclosure.
fn inside_brace(bytes: &[u8], cursor_byte: usize) -> bool {
    enclosing_brace_region(
        std::str::from_utf8(bytes).unwrap_or(""),
        cursor_byte,
    )
    .is_some()
}

/// Build the symbol table for `source` at `path` without requiring
/// the expression at the cursor to parse cleanly.
fn build_symbols(source: &str, source_path: &Path) -> SymbolTable {
    let controller = read_paired_controller(Some(source_path));
    let program = parse_program_safe(source);
    SymbolTable::build(&program, controller.as_ref())
}

fn ident_start_back(bytes: &[u8], end: usize) -> usize {
    let mut i = end;
    while i > 0 {
        let c = bytes[i - 1];
        if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
            i -= 1;
        } else {
            break;
        }
    }
    i
}

fn namespace_completions(head: &str, symbols: &SymbolTable) -> Vec<CompletionItem> {
    match head {
        "scope" => symbols
            .scope
            .iter()
            .map(|(name, e)| {
                let kind = match e.kind {
                    ScopeKind::Action => CompletionItemKind::FUNCTION,
                    ScopeKind::Derived => CompletionItemKind::PROPERTY,
                    _ => CompletionItemKind::VARIABLE,
                };
                let mut item = CompletionItem::new_simple(
                    name.clone(),
                    format!("scope {}", e.kind.label()),
                );
                item.kind = Some(kind);
                item
            })
            .collect(),
        "derived" => symbols
            .derived
            .keys()
            .map(|name| {
                let mut item = CompletionItem::new_simple(name.clone(), "derived getter".into());
                item.kind = Some(CompletionItemKind::PROPERTY);
                item
            })
            .collect(),
        "data" => symbols
            .data
            .keys()
            .map(|name| {
                let mut item = CompletionItem::new_simple(name.clone(), "<data> binding".into());
                item.kind = Some(CompletionItemKind::VARIABLE);
                item
            })
            .collect(),
        "props" => symbols
            .props
            .keys()
            .map(|name| {
                let mut item = CompletionItem::new_simple(name.clone(), "<prop> binding".into());
                item.kind = Some(CompletionItemKind::VARIABLE);
                item
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn root_completions(symbols: &SymbolTable) -> Vec<CompletionItem> {
    symbols
        .all_root_names()
        .into_iter()
        .map(|(name, label)| {
            let mut item = CompletionItem::new_simple(name, label.into());
            item.kind = Some(CompletionItemKind::VARIABLE);
            item
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_pair(dir: &tempfile::TempDir, sui: &str, ts: &str) -> std::path::PathBuf {
        let view = dir.path().join("view.sui");
        let ctrl = dir.path().join("view.controller.ts");
        std::fs::write(&view, sui).unwrap();
        std::fs::write(&ctrl, ts).unwrap();
        view
    }

    #[test]
    fn position_round_trip() {
        let txt = "abc\ndef\nghij";
        let pos = Position { line: 1, character: 2 };
        let off = position_to_offset(txt, pos);
        assert_eq!(&txt[off..off + 1], "f");
        let pos2 = offset_to_position(txt, off);
        assert_eq!(pos, pos2);
    }

    #[test]
    fn hover_resolves_scope_member() {
        let dir = tempfile::tempdir().unwrap();
        let sui = "<p>{scope.count()}</p>";
        let ts = r#"import { controller, signal } from '@ship/sui-runtime'
export default controller({ scope: () => ({ count: signal(0) }) })
"#;
        let path = write_pair(&dir, sui, ts);
        // Cursor on `count` in `scope.count`.
        let cursor = sui.find("count").unwrap() + 2;
        let h = hover(sui, &path, cursor).expect("hover present");
        let txt = match h.contents {
            HoverContents::Scalar(MarkedString::String(s)) => s,
            other => panic!("unexpected: {other:?}"),
        };
        assert!(txt.contains("scope.count"), "{txt}");
        assert!(txt.contains("signal"), "{txt}");
    }

    #[test]
    fn goto_resolves_to_controller() {
        let dir = tempfile::tempdir().unwrap();
        let sui = "<p>{scope.count()}</p>";
        let ts = r#"import { controller, signal } from '@ship/sui-runtime'
export default controller({ scope: () => ({ count: signal(0) }) })
"#;
        let path = write_pair(&dir, sui, ts);
        let cursor = sui.find("count").unwrap() + 2;
        let go = goto_definition(sui, &path, cursor).expect("goto present");
        match go {
            GotoDefinitionResponse::Scalar(loc) => {
                assert!(loc.uri.path().ends_with("view.controller.ts"));
            }
            _ => panic!("expected scalar response"),
        }
    }

    #[test]
    fn completion_after_scope_dot_lists_keys() {
        let dir = tempfile::tempdir().unwrap();
        let sui = "<p>{scope.}</p>";
        let ts = r#"import { controller, signal } from '@ship/sui-runtime'
export default controller({ scope: () => ({ count: signal(0), inc: (e) => e }) })
"#;
        let path = write_pair(&dir, sui, ts);
        let cursor = sui.find("scope.").unwrap() + 6;
        let items = completion(sui, &path, cursor);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"count"), "labels: {labels:?}");
        assert!(labels.contains(&"inc"), "labels: {labels:?}");
    }

    #[test]
    fn hover_outside_brace_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let sui = "<p>plain text</p>";
        let ts = "export default null";
        let path = write_pair(&dir, sui, ts);
        assert!(hover(sui, &path, 5).is_none());
    }
}
