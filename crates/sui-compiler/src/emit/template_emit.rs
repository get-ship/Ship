//! Walk template `Instr`s and emit DOM-construction JS into a writer.
//!
//! Each instruction maps to a small chunk of JS that creates DOM
//! nodes, registers reactive bindings, and pushes their disposers
//! onto a `__cleanups` array.
//!
//! Expressions are not pasted inline. They go through an `ExprBank`
//! that pre-compiles each unique expression into a `new Function`
//! at module load. The DOM-time call site is `__E[idx]($scope)`.
//!
//! The emit logic is split across three modules:
//!  * this file — orchestration, `ExprBank`, the `emit_one` dispatcher
//!  * `dom_emit` — element + attribute emit
//!  * `control_emit` — `{#if}`, `{#for}`, `{#match}`, `{#while-*}`

use super::writer::{js_string, Writer};
use crate::ir::*;

mod control_emit;
mod dom_emit;

/// Bank of unique expression strings. The emit step adds entries via
/// `add(src)`; the JS module emitter pre-compiles each entry once.
#[derive(Debug, Default)]
pub struct ExprBank {
    pub entries: Vec<String>,
}

impl ExprBank {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, src: &str) -> usize {
        if let Some(idx) = self.entries.iter().position(|e| e == src) {
            return idx;
        }
        self.entries.push(src.to_string());
        self.entries.len() - 1
    }
}

pub struct TemplateEmitter<'a> {
    pub(super) bank: &'a mut ExprBank,
    next_id: usize,
    /// Sequential counters for while-loading/while-error render-callback names.
    /// Both the bindings pass and this emit pass walk the IR in the same source
    /// order, so the N-th while-loading gets the same `__renderWhileLoading_<N>`
    /// name in both.
    pub(super) while_loading_idx: usize,
    pub(super) while_error_idx: usize,
    /// Render callbacks for `{#while-loading}` / `{#while-error}` blocks,
    /// emitted as named properties on the setup return object (outside
    /// `__mount`) so `attach()` can look them up by `renderKey`.
    pub(super) render_callbacks: Writer,
}

impl<'a> TemplateEmitter<'a> {
    pub fn new(bank: &'a mut ExprBank) -> Self {
        Self {
            bank,
            next_id: 0,
            while_loading_idx: 0,
            while_error_idx: 0,
            render_callbacks: Writer::new(),
        }
    }

    /// Drain the render-callback buffer. Called by `emit_setup_body`
    /// after `emit_top_level` finishes writing `__mount`.
    pub fn take_render_callbacks(&mut self) -> String {
        std::mem::replace(&mut self.render_callbacks, Writer::new()).into_string()
    }

    pub(super) fn fresh(&mut self, prefix: &str) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("__{prefix}{id}")
    }

    pub fn emit_top_level(
        &mut self,
        w: &mut Writer,
        instrs: &[Instr],
        parent_var: &str,
        cleanups_var: &str,
    ) {
        self.emit_children(w, instrs, parent_var, cleanups_var);
    }

    pub(super) fn emit_children(
        &mut self,
        w: &mut Writer,
        instrs: &[Instr],
        parent_var: &str,
        cleanups_var: &str,
    ) {
        for instr in instrs {
            self.emit_one(w, instr, parent_var, cleanups_var);
        }
    }

    pub(super) fn emit_one(
        &mut self,
        w: &mut Writer,
        instr: &Instr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        match instr {
            Instr::StaticText { text, .. } => {
                let v = self.fresh("t");
                w.line(&format!(
                    "const {v} = document.createTextNode({});",
                    js_string(text)
                ));
                w.line(&format!("{parent_var}.appendChild({v});"));
            }
            Instr::Comment { data, .. } => {
                let v = self.fresh("c");
                w.line(&format!(
                    "const {v} = document.createComment({});",
                    js_string(data)
                ));
                w.line(&format!("{parent_var}.appendChild({v});"));
            }
            Instr::DynamicText { expr } => {
                let v = self.fresh("t");
                let idx = self.bank.add(&expr.source);
                w.line(&format!("const {v} = document.createTextNode(\"\");"));
                w.line(&format!("{parent_var}.appendChild({v});"));
                w.line(&format!(
                    "{cleanups_var}.push(bindText({v}, () => __E[{idx}]($scope)));"
                ));
            }
            Instr::Element(el) => self.emit_element(w, el, parent_var, cleanups_var),
            Instr::If(if_instr) => self.emit_if(w, if_instr, parent_var, cleanups_var),
            Instr::For(for_instr) => self.emit_for(w, for_instr, parent_var, cleanups_var),
            Instr::Match(match_instr) => {
                self.emit_match(w, match_instr, parent_var, cleanups_var)
            }
            // While-loading/while-error: the render callback is emitted as a
            // named property on the setup return object (via `render_callbacks`
            // buffer). `__mount` emits an inline call for the legacy path.
            Instr::WhileLoading(wl) => {
                self.emit_while_loading_block(w, wl, parent_var, cleanups_var)
            }
            Instr::WhileError(we) => {
                self.emit_while_error_block(w, we, parent_var, cleanups_var)
            }
        }
    }

    /// Emit a `{ nodes, cleanup }` branch result. Used by `bindIf`,
    /// `bindFor`, and the `<while-*>` helpers.
    pub(super) fn emit_branch(&mut self, w: &mut Writer, body: &[Instr]) {
        w.line("() => {");
        w.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        w.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        w.line(&format!("const {cl} = [];"));
        self.emit_children(w, body, &frag, &cl);
        w.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        w.leave();
        w.line("},");
    }
}
