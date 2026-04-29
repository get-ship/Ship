//! Attribute classifier. Maps each attribute on an element to an
//! `AttrInstr`. Mirrors `packages/ship-set-dom/src/bindings/attr-compile.ts`.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;
use crate::parse::expr;
use crate::parse::html;

pub fn compile_attrs(
    el: &html::Element,
    loc_prefix: &str,
) -> Result<Vec<AttrInstr>, CompileError> {
    let mut out = Vec::new();
    for a in &el.attributes {
        out.push(compile_attr(&a.name, &a.value, a.value_span, &format!("{loc_prefix} {}", a.name))?);
    }
    Ok(out)
}

fn compile_attr(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if let Some(rest) = name.strip_prefix("on:") {
        return compile_on(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("bind:") {
        return compile_bind(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("class:") {
        return compile_class(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("style:") {
        return compile_style(rest, value, value_span, loc);
    }
    if let Some(rest) = name.strip_prefix("attr:") {
        return compile_dynamic_attr(rest, value, value_span, loc);
    }
    if is_expr_value(value) {
        let inner = strip_braces(value).to_string();
        expr::validate_at(&inner, false, loc, value_span)?;
        return Ok(AttrInstr::Expr {
            name: name.to_string(),
            expr: Expr::new(inner, value_span, false),
            span: value_span,
        });
    }
    Ok(AttrInstr::Static {
        name: name.to_string(),
        value: value.to_string(),
        span: value_span,
    })
}

fn is_expr_value(v: &str) -> bool {
    let t = v.trim();
    t.starts_with('{') && t.ends_with('}') && t.len() >= 2
}

fn strip_braces(v: &str) -> &str {
    let t = v.trim();
    if t.starts_with('{') && t.ends_with('}') && t.len() >= 2 {
        t[1..t.len() - 1].trim()
    } else {
        v
    }
}

fn compile_on(
    suffix: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    let mut parts = suffix.split('|');
    let event = parts.next().unwrap_or("").to_string();
    if event.is_empty() {
        return Err(CompileError::new("empty event name in on: binding")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let mut modifiers: Vec<Modifier> = Vec::new();
    for raw in parts {
        if raw.is_empty() {
            continue;
        }
        modifiers.push(parse_modifier(raw, value_span, loc)?);
    }
    if !is_expr_value(value) {
        return Err(CompileError::new("on: handler must be {expression}")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, true, loc, value_span)?;
    Ok(AttrInstr::On {
        event,
        handler: Expr::new(inner, value_span, true),
        modifiers,
        span: value_span,
    })
}

fn parse_modifier(raw: &str, span: Span, loc: &str) -> Result<Modifier, CompileError> {
    let (head, arg) = match raw.find(':') {
        Some(idx) => (&raw[..idx], Some(raw[idx + 1..].to_string())),
        None => (raw, None),
    };
    let key = head.to_ascii_lowercase();
    Ok(match key.as_str() {
        "preventdefault" | "prevent" => Modifier::PreventDefault,
        "stoppropagation" | "stop" => Modifier::StopPropagation,
        "stopimmediatepropagation" => Modifier::StopImmediatePropagation,
        "self" => Modifier::SelfMod,
        "trusted" => Modifier::Trusted,
        "key" => Modifier::Key(arg),
        "debounce" => Modifier::Debounce(arg),
        "throttle" => Modifier::Throttle(arg),
        _ => {
            return Err(CompileError::new(format!("unknown modifier \"{head}\""))
                .with_span(span)
                .with_category(loc.to_string()));
        }
    })
}

fn compile_bind(
    target: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if !is_expr_value(value) {
        return Err(CompileError::new(format!("bind:{target} expects {{signal}}"))
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, false, loc, value_span)?;
    let expr = Expr::new(inner, value_span, false);
    match target {
        "value" => Ok(AttrInstr::BindValue { expr, span: value_span }),
        "checked" => Ok(AttrInstr::BindChecked { expr, span: value_span }),
        other => Err(CompileError::new(format!("unknown bind:{other}"))
            .with_span(value_span)
            .with_category(loc.to_string())),
    }
}

fn compile_class(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if name.is_empty() {
        return Err(CompileError::new("class: requires a class name")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    if !is_expr_value(value) {
        return Err(CompileError::new(format!("class:{name} expects {{expr}}"))
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, false, loc, value_span)?;
    Ok(AttrInstr::Class {
        name: name.to_string(),
        expr: Expr::new(inner, value_span, false),
        span: value_span,
    })
}

fn compile_style(
    name: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if name.is_empty() {
        return Err(CompileError::new("style: requires a property name")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    if !is_expr_value(value) {
        return Err(CompileError::new(format!("style:{name} expects {{expr}}"))
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let inner = strip_braces(value).to_string();
    expr::validate_at(&inner, false, loc, value_span)?;
    Ok(AttrInstr::Style {
        name: name.to_string(),
        expr: Expr::new(inner, value_span, false),
        span: value_span,
    })
}

fn compile_dynamic_attr(
    raw_name_expr: &str,
    value: &str,
    value_span: Span,
    loc: &str,
) -> Result<AttrInstr, CompileError> {
    if !is_expr_value(raw_name_expr) {
        return Err(CompileError::new("attr: expects attr:{nameExpr}")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    if !is_expr_value(value) {
        return Err(CompileError::new("attr:{...} expects ={valueExpr}")
            .with_span(value_span)
            .with_category(loc.to_string()));
    }
    let name_inner = strip_braces(raw_name_expr).to_string();
    expr::validate_at(&name_inner, false, loc, value_span)?;
    let value_inner = strip_braces(value).to_string();
    expr::validate_at(&value_inner, false, loc, value_span)?;
    Ok(AttrInstr::DynamicAttr {
        name_expr: Expr::new(name_inner, value_span, false),
        expr: Expr::new(value_inner, value_span, false),
        span: value_span,
    })
}
