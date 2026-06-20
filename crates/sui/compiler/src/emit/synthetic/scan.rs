//! Free-identifier scan for the synthetic-controller emit: does a template
//! (or a `<data source>`) reference a given manifest name? A word-boundary
//! match so `route` hits in `route.params` but not in `myroute`, and a
//! leading `.` is treated as a member access (not a free root).

use crate::ir::*;

/// Does any template expression or `<data source>` reference `name`?
pub(super) fn references_ident(program: &Program, name: &str) -> bool {
    program.data.iter().any(|d| expr_references_ident(&d.source.source, name))
        || template_references_ident(&program.template, name)
}

fn template_references_ident(instrs: &[Instr], name: &str) -> bool {
    instrs.iter().any(|i| instr_references_ident(i, name))
}

fn instr_references_ident(instr: &Instr, name: &str) -> bool {
    match instr {
        Instr::DynamicText { expr } => expr_references_ident(&expr.source, name),
        Instr::Element(el) => {
            el.attrs.iter().any(|a| attr_references_ident(a, name))
                || template_references_ident(&el.children, name)
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            branches.iter().any(|b| {
                expr_references_ident(&b.cond.source, name)
                    || template_references_ident(&b.body, name)
            }) || else_body.as_ref().is_some_and(|b| template_references_ident(b, name))
        }
        Instr::For(ForInstr { iterable, key, body, else_body, .. }) => {
            expr_references_ident(&iterable.source, name)
                || key.as_ref().is_some_and(|k| expr_references_ident(&k.source, name))
                || template_references_ident(body, name)
                || else_body.as_ref().is_some_and(|b| template_references_ident(b, name))
        }
        Instr::Match(MatchInstr { discriminant, cases, default, .. }) => {
            expr_references_ident(&discriminant.source, name)
                || cases.iter().any(|c| {
                    expr_references_ident(&c.test.source, name)
                        || template_references_ident(&c.body, name)
                })
                || default.as_ref().is_some_and(|d| template_references_ident(d, name))
        }
        Instr::WhileLoading(wl) => template_references_ident(&wl.body, name),
        Instr::WhileError(we) => template_references_ident(&we.body, name),
        Instr::Let(l) => {
            expr_references_ident(&l.value.source, name)
                || template_references_ident(&l.body, name)
        }
        Instr::StaticText { .. } | Instr::Comment { .. } => false,
    }
}

fn attr_references_ident(attr: &AttrInstr, name: &str) -> bool {
    match attr {
        AttrInstr::Expr { expr, .. }
        | AttrInstr::BindValue { expr, .. }
        | AttrInstr::BindChecked { expr, .. }
        | AttrInstr::Class { expr, .. }
        | AttrInstr::Style { expr, .. }
        | AttrInstr::Prop { expr, .. }
        | AttrInstr::Mutation { expr, .. } => expr_references_ident(&expr.source, name),
        AttrInstr::On { handler, .. } => expr_references_ident(&handler.source, name),
        AttrInstr::DynamicAttr { name_expr, expr, .. } => {
            expr_references_ident(&name_expr.source, name)
                || expr_references_ident(&expr.source, name)
        }
        AttrInstr::Transition { opts, .. } => {
            opts.as_ref().is_some_and(|e| expr_references_ident(&e.source, name))
        }
        AttrInstr::Static { .. }
        | AttrInstr::BindThis { .. }
        | AttrInstr::SlotReflectEmpty { .. } => false,
    }
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Word-boundary scan for `name` in `src`. A leading `.` disqualifies a
/// match (it would be a member access, not a free root identifier).
fn expr_references_ident(src: &str, name: &str) -> bool {
    let bytes = src.as_bytes();
    let target = name.as_bytes();
    let tlen = target.len();
    if tlen == 0 || bytes.len() < tlen {
        return false;
    }
    for i in 0..=bytes.len() - tlen {
        if &bytes[i..i + tlen] != target {
            continue;
        }
        let ok_before = i == 0 || (!is_ident_char(bytes[i - 1]) && bytes[i - 1] != b'.');
        let ok_after = i + tlen >= bytes.len() || !is_ident_char(bytes[i + tlen]);
        if ok_before && ok_after {
            return true;
        }
    }
    false
}
