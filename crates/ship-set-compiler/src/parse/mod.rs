//! Parse pipeline. Walks `.html.set` source and produces an IR.
//!
//! Stages, in order:
//!  1. Preprocess — same two transforms the interpreter does
//!     (quote unquoted brace attrs, expand non-void self-closers).
//!  2. HTML parse — produces a span-bearing DOM-ish tree.
//!  3. Directive lift — pluck `<data>`, `<state>`, `<component>` out
//!     of the top-level child list into the IR.
//!  4. Template grammar scan — text-node `{...}` markers become
//!     template instructions; element children recurse.
//!  5. Expression validate — every brace expression goes through the
//!     same forbidden-syntax checks the interpreter applies.

use crate::diagnostics::{CompileError, Span};
use crate::ir::*;

pub mod expr;
pub mod html;
pub mod preprocess;
pub mod template;

pub use html::{Attr, Node};

/// Parse a `.html.set` source string into an IR program.
pub fn parse(source: &str) -> Result<Program, CompileError> {
    let prepared = preprocess::preprocess(source);
    let nodes = html::parse_fragment(&prepared)?;

    let mut data: Vec<DataDecl> = Vec::new();
    let mut state: Vec<StateDecl> = Vec::new();
    let mut component: Option<ComponentDecl> = None;
    let mut template_nodes: Vec<Node> = Vec::with_capacity(nodes.len());

    for node in nodes {
        if let Node::Element(el) = &node {
            match el.tag.as_str() {
                "data" => {
                    data.push(parse_data(el)?);
                    continue;
                }
                "state" => {
                    state.push(parse_state(el)?);
                    continue;
                }
                "component" => {
                    if component.is_some() {
                        return Err(CompileError::new(
                            "only one <component> declaration allowed per file",
                        )
                        .with_span(el.span));
                    }
                    component = Some(parse_component(el)?);
                    continue;
                }
                _ => {}
            }
        }
        template_nodes.push(node);
    }

    let template = template::compile_children(&template_nodes)?;

    Ok(Program {
        data,
        state,
        component,
        template,
        source_len: source.len(),
        span: Span::new(0, source.len()),
    })
}

fn parse_data(el: &html::Element) -> Result<DataDecl, CompileError> {
    let name = required_attr(el, "name", "<data>")?;
    let source_attr = required_attr(el, "source", "<data>")?;
    let live = el.has_attribute("live");
    // The `source=` attribute is a JS expression evaluated against
    // (ship, route). We validate it as a normal expression.
    let span = attr_span(el, "source").unwrap_or(el.span);
    let expr_text = strip_brace_attr(&source_attr);
    expr::validate_at(&expr_text, false, &format!("<data name=\"{name}\"> source"), span)?;
    Ok(DataDecl {
        name,
        source: Expr::new(expr_text, span, false),
        live,
        span: el.span,
    })
}

fn parse_state(el: &html::Element) -> Result<StateDecl, CompileError> {
    let name = required_attr(el, "name", "<state>")?;
    let raw = required_attr(el, "value", "<state>")?;
    let span = attr_span(el, "value").unwrap_or(el.span);
    let expr_text = strip_brace_attr(&raw);
    expr::validate_at(&expr_text, false, &format!("<state name=\"{name}\"> value"), span)?;
    Ok(StateDecl {
        name,
        value: Expr::new(expr_text, span, false),
        span: el.span,
    })
}

fn parse_component(el: &html::Element) -> Result<ComponentDecl, CompileError> {
    let tag = required_attr(el, "name", "<component>")?;
    let shadow = el.has_attribute("shadow");
    let mut props = Vec::new();
    let mut style_parts: Vec<String> = Vec::new();
    for child in &el.children {
        if let Node::Element(child_el) = child {
            match child_el.tag.as_str() {
                "prop" => props.push(parse_prop(child_el)?),
                "style" => {
                    // `<style>` is a raw-text tag; the parser collapses
                    // its body into a single Text node.
                    for sub in &child_el.children {
                        if let Node::Text(t) = sub {
                            style_parts.push(t.data.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let styles = style_parts.join("\n");
    Ok(ComponentDecl { tag, props, shadow, styles, span: el.span })
}

fn parse_prop(el: &html::Element) -> Result<PropDecl, CompileError> {
    let name = required_attr(el, "name", "<prop>")?;
    let values_attr = el.get_attribute("values");
    let type_attr = el.get_attribute("type");
    let default = el.get_attribute("default").map(String::from);
    let reflect = el.has_attribute("reflect");
    let bindable = el.has_attribute("bindable");

    let kind = if let Some(values_raw) = values_attr {
        let values = values_raw
            .split('|')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        PropKind::Union(values)
    } else {
        match type_attr.as_deref() {
            Some("bool") => PropKind::Bool,
            Some("number") => PropKind::Number,
            None | Some("string") => PropKind::String,
            Some(other) => {
                return Err(CompileError::new(format!(
                    "<prop name=\"{name}\"> unknown type: {other}"
                ))
                .with_span(el.span));
            }
        }
    };

    Ok(PropDecl { name, kind, default, reflect, bindable, span: el.span })
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

fn attr_span(el: &html::Element, name: &str) -> Option<Span> {
    el.attributes.iter().find(|a| a.name == name).map(|a| a.value_span)
}

/// Authors may write `value={0}` (brace form) or `value="0"` (literal
/// expression string). Both compile to the same code.
pub fn strip_brace_attr(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}
