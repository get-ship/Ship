//! Template grammar scan.
//!
//! Mirrors `packages/ship-set-dom/src/template/{tokenize,compile,
//! compile-blocks}.ts`. The flow:
//!
//!   parse children of <root> → flat token list
//!     → recursive parse into `Instr` tree
//!     → element children recurse
//!     → attribute classification (see `attrs::compile_attrs`)

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;
use crate::parse::{expr, html, html::Node};

mod attrs;
mod blocks;
mod tokenize;

pub use attrs::compile_attrs;
use blocks::ParseScope;
pub(crate) use tokenize::{tokenize_children, Discriminant, Token, TokenKind};

/// Compile a flat list of top-level template nodes.
pub fn compile_children(parent_children: &[Node]) -> Result<Vec<Instr>, CompileError> {
    let tokens = tokenize_children(parent_children);
    let result = parse_tokens(&tokens, 0, &[])?;
    Ok(result.instrs)
}

pub(crate) struct ParseResult {
    pub instrs: Vec<Instr>,
    pub end: usize,
}

pub(crate) fn parse_tokens(
    tokens: &[Token],
    start: usize,
    terminators: &[Discriminant],
) -> Result<ParseResult, CompileError> {
    let mut instrs = Vec::new();
    let mut i = start;
    while i < tokens.len() {
        let tok = &tokens[i];
        if terminators.contains(&tok.kind.discriminant()) {
            return Ok(ParseResult { instrs, end: i });
        }
        match &tok.kind {
            TokenKind::Text(s) => {
                if !s.is_empty() {
                    instrs.push(Instr::StaticText {
                        text: s.clone(),
                        span: tok.span,
                    });
                }
                i += 1;
            }
            TokenKind::Comment(data) => {
                instrs.push(Instr::Comment {
                    data: data.clone(),
                    span: tok.span,
                });
                i += 1;
            }
            TokenKind::Expr { src } => {
                expr::validate_at(src, false, "{expr}", tok.span)?;
                instrs.push(Instr::DynamicText {
                    expr: Expr::new(src.clone(), tok.span, false),
                });
                i += 1;
            }
            TokenKind::Element(node) => {
                instrs.push(Instr::Element(compile_element(node)?));
                i += 1;
            }
            TokenKind::OpenIf { .. } => {
                let scope = make_scope();
                let r = blocks::parse_if(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::OpenFor { .. } => {
                let scope = make_scope();
                let r = blocks::parse_for(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::OpenMatch { .. } => {
                let scope = make_scope();
                let r = blocks::parse_match(tokens, i, &scope)?;
                instrs.push(r.instr);
                i = r.end;
            }
            TokenKind::ElseIf { .. }
            | TokenKind::Else
            | TokenKind::Case { .. }
            | TokenKind::Default
            | TokenKind::CloseIf
            | TokenKind::CloseFor
            | TokenKind::CloseMatch => {
                return Err(CompileError::new(format!(
                    "unexpected token: {}",
                    tok.kind.label()
                ))
                .with_span(tok.span));
            }
        }
    }
    Ok(ParseResult { instrs, end: i })
}

fn make_scope() -> ParseScope {
    ParseScope {
        body: |tokens, start, terminators| parse_tokens(tokens, start, terminators),
    }
}

fn compile_element(el: &html::Element) -> Result<ElementInstr, CompileError> {
    let tag_lower = el.tag.as_str();
    let loc_prefix = format!("<{tag_lower}>");

    if tag_lower == "while-loading" {
        let bind = required_attr(el, "bind", "<while-loading>")?;
        return Ok(ElementInstr {
            tag: el.tag.clone(),
            attrs: Vec::new(),
            children: compile_children(&el.children)?,
            while_loading: Some(bind),
            while_error: None,
            span: el.span,
        });
    }
    if tag_lower == "while-error" {
        let bind = required_attr(el, "bind", "<while-error>")?;
        let mut error_var = "error".to_string();
        for a in &el.attributes {
            if let Some(rest) = a.name.strip_prefix("let:") {
                error_var = rest.to_string();
                break;
            }
        }
        return Ok(ElementInstr {
            tag: el.tag.clone(),
            attrs: Vec::new(),
            children: compile_children(&el.children)?,
            while_loading: None,
            while_error: Some(WhileErrorMeta { bind, error_var }),
            span: el.span,
        });
    }
    let attrs = compile_attrs(el, &loc_prefix)?;
    let children = compile_children(&el.children)?;
    Ok(ElementInstr {
        tag: el.tag.clone(),
        attrs,
        children,
        while_loading: None,
        while_error: None,
        span: el.span,
    })
}

fn required_attr(
    el: &html::Element,
    name: &str,
    label: &str,
) -> Result<String, CompileError> {
    el.get_attribute(name).ok_or_else(|| {
        CompileError::new(format!("{label} missing required attribute: {name}"))
            .with_span(el.span)
            .with_category(label.to_string())
    })
}

#[allow(dead_code)]
fn _force_use_span(_: Span) {}
