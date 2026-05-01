//! Block parsers for `{#if}`, `{#for}`, `{#match}`. Mirrors
//! `packages/ui-dom/src/template/compile-blocks.ts`.
//!
//! `for_header.rs` owns the gnarly header tokenization
//! (`X of items key={…}`) so this file stays under the line cap.

use crate::diagnostics::CompileError;
use crate::ir::*;
use crate::parse::expr;

use super::tokenize::{Discriminant, Token, TokenKind};
use super::ParseResult;

mod for_header;
use for_header::parse_for_header;

/// Callback that re-enters the top-level parser. Avoids a circular
/// module dependency between blocks and the recursive parse driver.
pub(crate) struct ParseScope {
    pub body: fn(&[Token], usize, &[Discriminant]) -> Result<ParseResult, CompileError>,
}

pub(crate) struct InstrResult {
    pub instr: Instr,
    pub end: usize,
}

const IF_TERMS: &[Discriminant] = &[
    Discriminant::ElseIf,
    Discriminant::Else,
    Discriminant::CloseIf,
];
const FOR_TERMS: &[Discriminant] = &[Discriminant::Else, Discriminant::CloseFor];
const MATCH_TERMS: &[Discriminant] = &[
    Discriminant::Case,
    Discriminant::Default,
    Discriminant::CloseMatch,
];

pub(crate) fn parse_if(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenIf { src } => src.clone(),
        _ => unreachable!(),
    };
    expr::validate_at(&head_src, false, "{#if}", head_span)?;
    let mut branches: Vec<IfBranch> = Vec::new();
    let mut else_body: Option<Vec<Instr>> = None;
    let mut cur_cond = Expr::new(head_src, head_span, false);
    let mut i = start + 1;
    loop {
        let body = (scope.body)(tokens, i, IF_TERMS)?;
        branches.push(IfBranch { cond: cur_cond.clone(), body: body.instrs });
        i = body.end;
        let term = tokens
            .get(i)
            .ok_or_else(|| CompileError::new("missing {/if}").with_span(head_span))?;
        match &term.kind {
            TokenKind::CloseIf => {
                i += 1;
                break;
            }
            TokenKind::ElseIf { src } => {
                expr::validate_at(src, false, "{:else if}", term.span)?;
                cur_cond = Expr::new(src.clone(), term.span, false);
                i += 1;
            }
            TokenKind::Else => {
                i += 1;
                let eb = (scope.body)(tokens, i, &[Discriminant::CloseIf])?;
                else_body = Some(eb.instrs);
                i = eb.end;
                let closer = tokens.get(i).ok_or_else(|| {
                    CompileError::new("missing {/if}").with_span(head_span)
                })?;
                if !matches!(closer.kind, TokenKind::CloseIf) {
                    return Err(CompileError::new("missing {/if}").with_span(head_span));
                }
                i += 1;
                break;
            }
            _ => unreachable!(),
        }
    }
    Ok(InstrResult {
        instr: Instr::If(IfInstr { branches, else_body, span: head_span }),
        end: i,
    })
}

pub(crate) fn parse_for(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenFor { src } => src.clone(),
        _ => unreachable!(),
    };
    let header = parse_for_header(&head_src, head_span)?;
    let mut i = start + 1;
    let body = (scope.body)(tokens, i, FOR_TERMS)?;
    i = body.end;
    let mut else_body: Option<Vec<Instr>> = None;
    if let Some(t) = tokens.get(i) {
        if matches!(t.kind, TokenKind::Else) {
            i += 1;
            let eb = (scope.body)(tokens, i, &[Discriminant::CloseFor])?;
            else_body = Some(eb.instrs);
            i = eb.end;
        }
    }
    let closer = tokens
        .get(i)
        .ok_or_else(|| CompileError::new("missing {/for}").with_span(head_span))?;
    if !matches!(closer.kind, TokenKind::CloseFor) {
        return Err(CompileError::new("missing {/for}").with_span(head_span));
    }
    i += 1;
    Ok(InstrResult {
        instr: Instr::For(ForInstr {
            var_name: header.var_name,
            index_name: header.index_name,
            iterable: header.iterable,
            key: header.key,
            body: body.instrs,
            else_body,
            span: head_span,
        }),
        end: i,
    })
}

pub(crate) fn parse_match(
    tokens: &[Token],
    start: usize,
    scope: &ParseScope,
) -> Result<InstrResult, CompileError> {
    let head_span = tokens[start].span;
    let head_src = match &tokens[start].kind {
        TokenKind::OpenMatch { src } => src.clone(),
        _ => unreachable!(),
    };
    expr::validate_at(&head_src, false, "{#match}", head_span)?;
    let discriminant = Expr::new(head_src, head_span, false);
    let mut i = start + 1;
    while let Some(t) = tokens.get(i) {
        match &t.kind {
            TokenKind::Text(s) if s.trim().is_empty() => i += 1,
            _ => break,
        }
    }
    let mut cases: Vec<MatchCase> = Vec::new();
    let mut default: Option<Vec<Instr>> = None;
    while let Some(t) = tokens.get(i) {
        match &t.kind {
            TokenKind::CloseMatch => {
                i += 1;
                break;
            }
            TokenKind::Case { src } => {
                expr::validate_at(src, false, "{:case}", t.span)?;
                let test = Expr::new(src.clone(), t.span, false);
                i += 1;
                let body = (scope.body)(tokens, i, MATCH_TERMS)?;
                cases.push(MatchCase { test, body: body.instrs });
                i = body.end;
            }
            TokenKind::Default => {
                i += 1;
                let body = (scope.body)(tokens, i, &[Discriminant::CloseMatch])?;
                default = Some(body.instrs);
                i = body.end;
            }
            _ => {
                return Err(CompileError::new(format!(
                    "unexpected token in {{#match}}: {}",
                    t.kind.label()
                ))
                .with_span(t.span));
            }
        }
    }
    Ok(InstrResult {
        instr: Instr::Match(MatchInstr {
            discriminant,
            cases,
            default,
            span: head_span,
        }),
        end: i,
    })
}
