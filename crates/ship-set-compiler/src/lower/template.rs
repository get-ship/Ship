//! Template-level helpers consumed by the emitter.
//!
//! Walks the IR collecting which special HTML tags are used (e.g.
//! `<while-loading>`) so the emitter can decide what runtime helpers
//! to import. Beyond that, the emit step works directly off the IR.

use crate::ir::{Instr, ElementInstr};

/// Recursively true if any element under `instrs` is a custom
/// element (tag contains a hyphen). Used to flag whether the emitted
/// JS needs to wait for `customElements.whenDefined` at mount.
pub fn has_custom_element(instrs: &[Instr]) -> bool {
    instrs.iter().any(|i| match i {
        Instr::Element(e) => element_or_descendant_is_custom(e),
        Instr::If(b) => {
            b.branches.iter().any(|br| has_custom_element(&br.body))
                || b.else_body.as_ref().map_or(false, |e| has_custom_element(e))
        }
        Instr::For(f) => {
            has_custom_element(&f.body)
                || f.else_body.as_ref().map_or(false, |e| has_custom_element(e))
        }
        Instr::Match(m) => {
            m.cases.iter().any(|c| has_custom_element(&c.body))
                || m.default.as_ref().map_or(false, |d| has_custom_element(d))
        }
        _ => false,
    })
}

fn element_or_descendant_is_custom(el: &ElementInstr) -> bool {
    if el.tag.contains('-') && el.tag != "while-loading" && el.tag != "while-error" {
        return true;
    }
    has_custom_element(&el.children)
}
