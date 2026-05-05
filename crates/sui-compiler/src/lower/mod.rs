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
use crate::CompileMode;
use std::collections::BTreeSet;

pub mod bindings;
pub mod directives;
pub mod template;

#[derive(Debug, Clone, Default)]
pub struct Lowered {
    /// Symbols imported from `@ship/ui-runtime`. Sorted for stable emit.
    pub runtime_imports: BTreeSet<&'static str>,
    /// Symbols imported from `@ship/ui-dom`. Sorted for stable emit.
    /// Phase 3b.3: views with depth-0 `DynamicText` import the hydrate
    /// helpers (`hydrateText`, `collectTextOpens`) — direct-emit replaces
    /// the `text` manifest entry that the runtime walker used to wire.
    pub dom_imports: BTreeSet<&'static str>,
    /// `true` if the user's setup file is referenced anywhere.
    pub uses_setup: bool,
}

/// Walk the program, build a `Lowered` summary.
pub fn lower(program: &Program, mode: CompileMode) -> Lowered {
    let mut out = Lowered::default();
    let is_view = mode == CompileMode::View;
    if mode == CompileMode::Component {
        out.runtime_imports.insert("component");
        if !program.props.is_empty() {
            out.runtime_imports.insert("prop");
        }
    } else {
        out.runtime_imports.insert("view");
    }
    if !program.data.is_empty() {
        out.runtime_imports.insert("data");
        out.runtime_imports.insert("live");
    }
    let mut depth = 0;
    walk_instrs(&program.template, &mut out, is_view, &mut depth);
    out
}

fn walk_instrs(instrs: &[Instr], out: &mut Lowered, is_view: bool, depth: &mut usize) {
    for instr in instrs {
        walk_instr(instr, out, is_view, depth);
    }
}

fn walk_instr(instr: &Instr, out: &mut Lowered, is_view: bool, depth: &mut usize) {
    match instr {
        Instr::StaticText { .. } | Instr::Comment { .. } => {}
        Instr::DynamicText { .. } => {
            out.runtime_imports.insert("bindText");
            // Phase 3b.3: depth-0 text bindings in views are direct-emit.
            // The compiler still calls `bindText` from `__mount` (legacy
            // CSR path), but `hydrate(root, ctx)` wires via these helpers.
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectTextOpens");
                out.dom_imports.insert("hydrateText");
            }
        }
        Instr::Element(el) => {
            for a in &el.attrs {
                bindings::collect_attr_imports(a, out, is_view, *depth);
            }
            walk_instrs(&el.children, out, is_view, depth);
        }
        Instr::WhileLoading(wl) => {
            out.runtime_imports.insert("bindWhileLoading");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateWhileLoading");
            }
            *depth += 1;
            walk_instrs(&wl.body, out, is_view, depth);
            *depth -= 1;
        }
        Instr::WhileError(we) => {
            out.runtime_imports.insert("bindWhileError");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
                out.dom_imports.insert("hydrateWhileError");
            }
            *depth += 1;
            walk_instrs(&we.body, out, is_view, depth);
            *depth -= 1;
        }
        Instr::If(IfInstr { branches, else_body, .. }) => {
            out.runtime_imports.insert("bindIf");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
            }
            *depth += 1;
            for b in branches {
                walk_instrs(&b.body, out, is_view, depth);
            }
            if let Some(eb) = else_body {
                walk_instrs(eb, out, is_view, depth);
            }
            *depth -= 1;
        }
        Instr::For(ForInstr { body, else_body, .. }) => {
            out.runtime_imports.insert("bindFor");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
            }
            *depth += 1;
            walk_instrs(body, out, is_view, depth);
            if let Some(eb) = else_body {
                walk_instrs(eb, out, is_view, depth);
            }
            *depth -= 1;
        }
        Instr::Match(MatchInstr { cases, default, .. }) => {
            out.runtime_imports.insert("bindIf");
            if is_view && *depth == 0 {
                out.dom_imports.insert("collectElementMarkers");
            }
            *depth += 1;
            for c in cases {
                walk_instrs(&c.body, out, is_view, depth);
            }
            if let Some(d) = default {
                walk_instrs(d, out, is_view, depth);
            }
            *depth -= 1;
        }
    }
}
