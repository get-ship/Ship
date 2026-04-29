//! IR → emit-ready ops.
//!
//! The IR produced by `parse` is already structured by directive +
//! template instructions. The lower step's job is to compute the
//! information the emitter needs but doesn't want to recompute:
//!
//!  - which runtime imports the emitted JS will need
//!  - the flat list of bindings to register at mount time
//!  - the final list of decoder calls per component prop
//!
//! Mostly this is a walk over the IR collecting metadata. The emit
//! step uses the IR plus this `Lowered` struct to produce JS.

use crate::ir::*;
use std::collections::BTreeSet;

pub mod bindings;
pub mod directives;
pub mod template;

#[derive(Debug, Clone, Default)]
pub struct Lowered {
    /// Symbols imported from `@ship-set/runtime`. Sorted for stable emit.
    pub runtime_imports: BTreeSet<&'static str>,
    /// `true` if the user's setup file is referenced anywhere.
    pub uses_setup: bool,
}

/// Walk the program, build a `Lowered` summary.
pub fn lower(program: &Program) -> Lowered {
    let mut out = Lowered::default();
    if program.component.is_some() {
        out.runtime_imports.insert("component");
        out.runtime_imports.insert("prop");
    } else {
        out.runtime_imports.insert("view");
    }
    if !program.data.is_empty() {
        out.runtime_imports.insert("data");
        out.runtime_imports.insert("live");
    }
    if !program.state.is_empty() {
        out.runtime_imports.insert("signal");
    }
    walk_instrs(&program.template, &mut out);
    out
}

fn walk_instrs(instrs: &[Instr], out: &mut Lowered) {
    for instr in instrs {
        walk_instr(instr, out);
    }
}

fn walk_instr(instr: &Instr, out: &mut Lowered) {
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => {}
        Instr::DynamicText { .. } => {
            out.runtime_imports.insert("bindText");
        }
        Instr::Element(el) => {
            for a in &el.attrs {
                bindings::collect_attr_imports(a, out);
            }
            walk_instrs(&el.children, out);
            if el.while_loading.is_some() {
                out.runtime_imports.insert("bindWhileLoading");
            }
            if el.while_error.is_some() {
                out.runtime_imports.insert("bindWhileError");
            }
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            out.runtime_imports.insert("bindIf");
            for b in branches {
                walk_instrs(&b.body, out);
            }
            if let Some(eb) = else_body {
                walk_instrs(eb, out);
            }
        }
        Instr::For(ForInstr { body, else_body, .. }) => {
            out.runtime_imports.insert("bindFor");
            walk_instrs(body, out);
            if let Some(eb) = else_body {
                walk_instrs(eb, out);
            }
        }
        Instr::Match(MatchInstr { cases, default, .. }) => {
            out.runtime_imports.insert("bindIf");
            for c in cases {
                walk_instrs(&c.body, out);
            }
            if let Some(d) = default {
                walk_instrs(d, out);
            }
        }
    }
}
