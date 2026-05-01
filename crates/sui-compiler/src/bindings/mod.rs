//! Binding-manifest pre-pass.
//!
//! Walks the IR depth-first in the same source order the SSR render
//! walker uses today, allocating one `b<N>` key per binding-bearing
//! source position and recording a `Binding` entry that describes it.
//!
//! Two consumers downstream:
//! 1. `emit::js` writes the manifest into the compiled ESM as
//!    `export const bindings = [...]`. Phase E's attach-only client
//!    runtime imports it and resolves author-syntax markers by
//!    indexing into the array.
//! 2. The render walker (`render::walker`) shares the same allocator —
//!    its emitted marker keys are exactly the registry's keys, so the
//!    SSR HTML and the manifest agree on what `b<N>` refers to.
//!
//! ## Branch handling
//!
//! Control-flow bodies (`{#if}`, `{#match}`) are walked across **every**
//! branch (taken or not). The registry has no SSR scope, so it can't
//! tell which branch will render at SSR time — the marker contract
//! (`docs/ship-adrs/.../sui-marker-contract.md`) takes the position
//! that the manifest is the union of all bindings the IR could emit;
//! markers in the SSR'd HTML reference a subset.
//!
//! `{#for}` bodies are walked **once** — the walker's snapshot/restore
//! makes every iteration share the same key range, so the registry
//! mirrors that with a single body walk.

use crate::ir::*;
use crate::keys::{Binding, BindingKind, BindingRegistry, KeyAllocator};
use crate::render::eval::collect_idents;
use crate::render::is_void_element;

#[cfg(test)]
mod tests;

/// Build the binding manifest for a parsed program.
pub fn build(program: &Program) -> BindingRegistry {
    let mut b = Builder::default();
    b.walk_block(&program.template);
    BindingRegistry::from_bindings(b.bindings)
}

#[derive(Default)]
struct Builder {
    keys: KeyAllocator,
    bindings: Vec<Binding>,
}

impl Builder {
    fn push(&mut self, kind: BindingKind, expr_src: &str) {
        let key = self.keys.alloc();
        self.bindings.push(Binding {
            key,
            kind,
            expr: expr_src.to_string(),
            deps: collect_idents(expr_src),
        });
    }

    fn walk_block(&mut self, instrs: &[Instr]) {
        for i in instrs {
            self.walk_instr(i);
        }
    }

    fn walk_instr(&mut self, instr: &Instr) {
        match instr {
            Instr::StaticText { .. } | Instr::Comment { .. } => {}
            Instr::DynamicText { expr } => {
                self.push(BindingKind::Text, &expr.source);
            }
            Instr::Element(el) => self.walk_element(el),
            Instr::If(if_instr) => self.walk_if(if_instr),
            Instr::For(for_instr) => self.walk_for(for_instr),
            Instr::Match(match_instr) => self.walk_match(match_instr),
        }
    }

    fn walk_element(&mut self, el: &ElementInstr) {
        let lower = el.tag.to_ascii_lowercase();
        if lower == "state" || lower == "data" {
            return;
        }
        for attr in &el.attrs {
            self.walk_attr(attr);
        }
        if !is_void_element(&lower) {
            self.walk_block(&el.children);
        }
    }

    fn walk_attr(&mut self, attr: &AttrInstr) {
        match attr {
            AttrInstr::Static { .. } => {}
            AttrInstr::Expr { name, expr, .. } => {
                self.push(BindingKind::Attr(name.clone()), &expr.source);
            }
            AttrInstr::Class { name, expr, .. } => {
                self.push(BindingKind::Class(name.clone()), &expr.source);
            }
            AttrInstr::Style { name, expr, .. } => {
                self.push(BindingKind::Style(name.clone()), &expr.source);
            }
            AttrInstr::BindValue { expr, .. } => {
                self.push(BindingKind::BindValue, &expr.source);
            }
            AttrInstr::BindChecked { expr, .. } => {
                self.push(BindingKind::BindChecked, &expr.source);
            }
            AttrInstr::On { event, handler, .. } => {
                self.push(BindingKind::On(event.clone()), &handler.source);
            }
            AttrInstr::DynamicAttr { expr, .. } => {
                self.push(BindingKind::AttrDynamic, &expr.source);
            }
            AttrInstr::Mutation { expr, .. } => {
                self.push(BindingKind::Mutation, &expr.source);
            }
        }
    }

    /// `{#if}`: one binding for the boundary, then walk every branch
    /// body and the optional else body. The boundary's `expr` is the
    /// first branch's condition (the canonical discriminator);
    /// alternate-branch conditions live in the IR but the manifest's
    /// shape is one row per allocated key.
    fn walk_if(&mut self, instr: &IfInstr) {
        let cond_src = instr
            .branches
            .first()
            .map(|b| b.cond.source.as_str())
            .unwrap_or("");
        self.push(BindingKind::If, cond_src);
        for branch in &instr.branches {
            self.walk_block(&branch.body);
        }
        if let Some(else_body) = &instr.else_body {
            self.walk_block(else_body);
        }
    }

    /// `{#for}`: one binding for the boundary (expr = iterable source),
    /// body walked once, else body walked once.
    fn walk_for(&mut self, instr: &ForInstr) {
        self.push(BindingKind::For, &instr.iterable.source);
        self.walk_block(&instr.body);
        if let Some(else_body) = &instr.else_body {
            self.walk_block(else_body);
        }
    }

    /// `{#match}`: one binding for the boundary (expr = discriminant),
    /// each case body walked, default walked.
    fn walk_match(&mut self, instr: &MatchInstr) {
        self.push(BindingKind::Match, &instr.discriminant.source);
        for case in &instr.cases {
            self.walk_block(&case.body);
        }
        if let Some(default) = &instr.default {
            self.walk_block(default);
        }
    }
}
