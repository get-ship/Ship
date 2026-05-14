//! Compile-time semantic checks that run after the IR has been built.
//!
//! These passes are additive — they do not mutate the IR, they only
//! emit diagnostics. The first pass (and currently only one) is the
//! renderable-expression subset enforced on every interpolation
//! expression in the program.

pub mod closed_sections;
pub mod controller_parse;
#[cfg(test)]
mod controller_parse_tests;
pub mod controller_purity;
pub mod css_token;
pub mod expr_subset;
pub mod handler_complexity;
pub(crate) mod js_scan;
pub mod pure_derived;
#[cfg(test)]
mod pure_derived_tests;
pub mod scope_bound;
mod scope_bound_expr;
#[cfg(test)]
mod scope_bound_expr_tests;
pub mod symbol_table;
pub mod scope_member;
pub mod key_identity;
pub mod bind_this_target;
pub mod on_callable;
pub mod for_iterable;

use std::path::Path;

use crate::diagnostics::CompileError;
use crate::ir::*;

/// Run every post-parse check. Returns the first error, or `Ok(())`.
///
/// Event handlers (`allow_arrow == true`) are excluded from the renderable
/// expr-subset check; they are checked separately by `handler_complexity`.
pub fn run(
    program: &Program,
    source: &str,
    source_path: Option<&Path>,
) -> Result<(), CompileError> {
    visit_program(program, &mut |e| {
        if e.allow_arrow {
            return Ok(());
        }
        expr_subset::check(e)
    })?;
    check_mutations(program)?;
    handler_complexity::check(program, source_path)?;
    controller_purity::check(source_path)?;
    css_token::check(source, source_path)?;
    let controller = controller_parse::read_paired_controller(source_path);
    if let Some(c) = controller.as_ref() {
        closed_sections::check(c)?;
        pure_derived::check(c)?;
    }
    // The scope-bound check needs to know about controller-declared
    // names (`derived` / `use`) — and we can only locate the paired
    // controller via `source_path`. When the caller hasn't supplied
    // one (synthetic fragments in unit tests), skip the check rather
    // than reject every reference as undeclared.
    if source_path.is_some() {
        scope_bound::check(program, controller.as_ref())?;
    }
    // Type-strengthening passes — see crates/sui-compiler/EXPRESSIONS.md.
    // These run after scope_bound so the user gets the "name not declared"
    // error first (it's the more common authoring mistake) before the
    // shape mismatches surface.
    key_identity::check(program)?;
    for_iterable::check(program)?;
    if source_path.is_some() {
        scope_member::check(program, controller.as_ref())?;
        bind_this_target::check(program, controller.as_ref())?;
        on_callable::check(program, controller.as_ref())?;
    }
    Ok(())
}

/// Validate that every `mutation=` attribute contains a valid `ship.X.Y` chain.
fn check_mutations(program: &Program) -> Result<(), CompileError> {
    check_mutations_instrs(&program.template)
}

fn check_mutations_instrs(instrs: &[Instr]) -> Result<(), CompileError> {
    for instr in instrs {
        if let Instr::Element(el) = instr {
            for attr in &el.attrs {
                if let AttrInstr::Mutation { expr, .. } = attr {
                    validate_mutation_expr(&expr.source, expr.span)?;
                }
            }
            check_mutations_instrs(&el.children)?;
        }
        match instr {
            Instr::If(i) => {
                for b in &i.branches { check_mutations_instrs(&b.body)?; }
                if let Some(e) = &i.else_body { check_mutations_instrs(e)?; }
            }
            Instr::For(f) => {
                check_mutations_instrs(&f.body)?;
                if let Some(e) = &f.else_body { check_mutations_instrs(e)?; }
            }
            Instr::Match(m) => {
                for c in &m.cases { check_mutations_instrs(&c.body)?; }
                if let Some(d) = &m.default { check_mutations_instrs(d)?; }
            }
            Instr::WhileLoading(wl) => check_mutations_instrs(&wl.body)?,
            Instr::WhileError(we) => check_mutations_instrs(&we.body)?,
            _ => {}
        }
    }
    Ok(())
}

/// `ship.X.Y` — must have ≥3 dot-separated segments, first is `ship`,
/// each segment is a non-empty ASCII identifier. No parens or args.
fn validate_mutation_expr(expr: &str, span: crate::diagnostics::Span) -> Result<(), CompileError> {
    let parts: Vec<&str> = expr.split('.').collect();
    let valid = parts.len() >= 3
        && parts[0] == "ship"
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        });
    if !valid {
        return Err(CompileError::new(format!(
            "mutation= must be ship.<namespace>.<method> (got `{expr}`)"
        ))
        .with_span(span));
    }
    Ok(())
}

/// Walk every `Expr` in the program and invoke `f` on each. Public so
/// the type-strengthening checks (`scope_member`, `key_identity`, ...)
/// can share the traversal instead of re-implementing it five times.
pub fn visit_exprs<F>(program: &Program, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    visit_program(program, f)
}

/// Walk every `Expr` in the program and invoke `f` on each.
fn visit_program<F>(program: &Program, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    for d in &program.data {
        f(&d.source)?;
    }
    visit_instrs(&program.template, f)?;
    Ok(())
}

fn visit_instrs<F>(instrs: &[Instr], f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    for instr in instrs {
        visit_instr(instr, f)?;
    }
    Ok(())
}

fn visit_instr<F>(instr: &Instr, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => {}
        Instr::DynamicText { expr } => f(expr)?,
        Instr::Element(el) => {
            for a in &el.attrs {
                visit_attr(a, f)?;
            }
            visit_instrs(&el.children, f)?;
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            for b in branches {
                f(&b.cond)?;
                visit_instrs(&b.body, f)?;
            }
            if let Some(eb) = else_body {
                visit_instrs(eb, f)?;
            }
        }
        Instr::For(ForInstr { iterable, key, body, else_body, .. }) => {
            f(iterable)?;
            if let Some(k) = key {
                f(k)?;
            }
            visit_instrs(body, f)?;
            if let Some(eb) = else_body {
                visit_instrs(eb, f)?;
            }
        }
        Instr::Match(MatchInstr { discriminant, cases, default, .. }) => {
            f(discriminant)?;
            for c in cases {
                f(&c.test)?;
                visit_instrs(&c.body, f)?;
            }
            if let Some(d) = default {
                visit_instrs(d, f)?;
            }
        }
        Instr::WhileLoading(wl) => visit_instrs(&wl.body, f)?,
        Instr::WhileError(we) => visit_instrs(&we.body, f)?,
    }
    Ok(())
}

fn visit_attr<F>(attr: &AttrInstr, f: &mut F) -> Result<(), CompileError>
where
    F: FnMut(&Expr) -> Result<(), CompileError>,
{
    match attr {
        AttrInstr::Static { .. } => {}
        AttrInstr::Expr { expr, .. } => f(expr)?,
        AttrInstr::DynamicAttr { name_expr, expr, .. } => {
            f(name_expr)?;
            f(expr)?;
        }
        AttrInstr::BindValue { expr, .. } | AttrInstr::BindChecked { expr, .. } => f(expr)?,
        AttrInstr::BindThis { .. } | AttrInstr::SlotReflectEmpty { .. } => {}
        AttrInstr::On { handler, .. } => f(handler)?,
        AttrInstr::Class { expr, .. }
        | AttrInstr::Style { expr, .. }
        | AttrInstr::Prop { expr, .. } => f(expr)?,
        AttrInstr::Mutation { .. } => {}
        AttrInstr::Transition { opts, .. } => {
            if let Some(e) = opts {
                f(e)?;
            }
        }
    }
    Ok(())
}
