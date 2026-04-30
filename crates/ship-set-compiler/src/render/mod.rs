//! SSR render walker — Phase D-1 (scaffold + static rendering).
//!
//! Walks the lowered IR and emits HTML with `data-set-*` markers per
//! the marker contract ADR (`docs/ship-adrs/.../ship-set-marker-contract.md`).
//!
//! D-1 scope:
//! * Static text + comments + elements with static attributes render
//!   verbatim.
//! * Every dynamic attribute kind emits a `data-set-*` marker whose
//!   value is an opaque counter-allocated key (`b0`, `b1`, …). The real
//!   expression source is **not** emitted here; that lives in the JSON
//!   island / IR-side resolver added in D-2.
//! * Dynamic text (`{expr}`) emits the comment-marker shape from the
//!   ADR: `<!--set:text:bN--><!--/set-->`.
//! * Control flow (`{#if}`, `{#for}`, `{#match}`) emits a single empty
//!   `<template data-set-…="bN"></template>` placeholder; the body is
//!   filled in by D-3.
//!
//! No expression evaluation happens in this phase; every dynamic value
//! is a marker.

use crate::diagnostics::CompileError;
use crate::ir::{AttrInstr, ElementInstr, ForInstr, IfInstr, Instr, MatchInstr, Program};

mod escape;
mod walker;

#[cfg(test)]
mod tests;

pub use escape::{escape_attr_value, is_void_element};

/// Public entry point.
#[derive(Debug, Clone)]
pub struct RenderOpts {
    /// View name — used for diagnostics and (later) the JSON island id.
    pub view_name: String,
}

/// Output of a render pass.
#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub html: String,
}

/// Render a parsed program to HTML with `data-set-*` markers.
///
/// `D-1` does not evaluate any expressions; every dynamic binding
/// becomes an opaque-key marker. Later phases (D-2 evaluation, D-3
/// control-flow expansion) produce the full output.
pub fn render(program: &Program, opts: &RenderOpts) -> Result<RenderOutput, CompileError> {
    let _ = opts; // reserved for D-2 (JSON island id)
    let mut state = walker::Walker::new();
    state.walk_block(&program.template);
    Ok(RenderOutput { html: state.into_html() })
}

/// Allocates stable opaque marker keys (`b0`, `b1`, …) in source order.
#[derive(Debug, Default)]
pub(crate) struct KeyAllocator {
    next: usize,
}

impl KeyAllocator {
    pub(crate) fn alloc(&mut self) -> String {
        let key = format!("b{}", self.next);
        self.next += 1;
        key
    }
}

/// Re-export the shape of a single emit decision so the walker can
/// keep its match arms readable. Control-flow doesn't appear here —
/// it's classified by `control_flow_kind` on the `Instr` level, before
/// the walker reaches an element.
pub(crate) enum EmitDecision {
    /// Skip outright (e.g. `<state>`, `<data>`).
    Skip,
    /// Self-closing void element — emit `<tag …>` with no body / close.
    Void,
    /// Normal element — emit open, recurse, close.
    Normal,
}

/// Decide what to do with a top-level instruction kind. Pure helper —
/// keeps the walker's match readable.
pub(crate) fn decide_element(el: &ElementInstr) -> EmitDecision {
    let lower = el.tag.to_ascii_lowercase();
    if lower == "state" || lower == "data" {
        return EmitDecision::Skip;
    }
    if is_void_element(&lower) {
        return EmitDecision::Void;
    }
    EmitDecision::Normal
}

/// Map an attribute-instruction variant onto the `data-set-*` marker
/// name to emit. Returns `None` for `Static` (which is rendered
/// verbatim without a marker).
pub(crate) fn marker_attr_name(attr: &AttrInstr) -> Option<String> {
    match attr {
        AttrInstr::Static { .. } => None,
        AttrInstr::Expr { name, .. } => Some(format!("data-set-attr:{name}")),
        AttrInstr::DynamicAttr { .. } => Some("data-set-attr-dynamic".to_string()),
        AttrInstr::BindValue { .. } => Some("data-set-bind:value".to_string()),
        AttrInstr::BindChecked { .. } => Some("data-set-bind:checked".to_string()),
        AttrInstr::On { event, .. } => Some(format!("data-set-on:{event}")),
        AttrInstr::Class { name, .. } => Some(format!("data-set-class:{name}")),
        // Style is not in the ADR table; emit as a deliberate extension.
        AttrInstr::Style { name, .. } => Some(format!("data-set-style:{name}")),
    }
}

/// Marker name for control-flow instructions. Used for the
/// `<template data-set-…>` placeholder.
pub(crate) fn control_flow_kind(instr: &Instr) -> Option<&'static str> {
    match instr {
        Instr::If(IfInstr { .. }) => Some("if"),
        Instr::For(ForInstr { .. }) => Some("for"),
        Instr::Match(MatchInstr { .. }) => Some("match"),
        _ => None,
    }
}
