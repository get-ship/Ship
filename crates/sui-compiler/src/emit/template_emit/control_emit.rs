//! Control-flow emit: `{#if}`, `{#for}`, `{#match}`, `{#while-loading}`,
//! and `{#while-error}`.
//!
//! Phase 2 design: each `{#if}` / `{#for}` / `{#match}` boundary emits
//! a hoisted `__wireBoundary_b<N>` const inside `setup(ctx)` plus a
//! `__b<N>_attach` named property on the view's setup return. The
//! __mount path (legacy hand-mount) and `attach()` (SSR re-hydration)
//! both call into the same hoisted helper, with `attach()` first
//! swapping the SSR'd `<sui-{kind} key="b<N>">` wrapper for a Comment
//! anchor. `bindIf` / `bindFor`'s effect mounts the active branch
//! synchronously, replacing the SSR'd content client-side.
//!
//! `{#while-loading}` / `{#while-error}` keep their existing renderKey
//! shape — they're already client-side and don't go through the
//! deleted `/_sui/render-fragment/` endpoint.

use super::super::expr_rewrite::rewrite_for_scope;
use super::super::writer::{js_string, Writer};
use super::TemplateEmitter;
use crate::ir::*;

impl TemplateEmitter {
    pub(in crate::emit) fn emit_if(
        &mut self,
        w: &mut Writer,
        instr: &IfInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("ifAnchor");

        // __mount-side: anchor + invoke hoisted handler.
        w.line(&format!(
            "const {anchor} = document.createComment(\"if\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}));"
        ));

        // Hoisted handler — emitted into `boundary_hoists`, drained
        // into the setup body before the `return`.
        let mut h = Writer::new();
        h.line(&format!(
            "const __wireBoundary_{key} = (anchor) => bindIf({{"
        ));
        h.enter();
        h.line("anchor,");
        h.line("test: () => {");
        h.enter();
        for (i, b) in instr.branches.iter().enumerate() {
            let rw = rewrite_for_scope(&b.cond.source, &[]);
            h.line(&format!("if ({rw}) return {i};"));
        }
        let fallback = if instr.else_body.is_some() {
            instr.branches.len() as i32
        } else {
            -1
        };
        h.line(&format!("return {fallback};"));
        h.leave();
        h.line("},");
        h.line("branches: [");
        h.enter();
        for b in &instr.branches {
            self.emit_branch(&mut h, &b.body);
        }
        if let Some(eb) = &instr.else_body {
            self.emit_branch(&mut h, eb);
        }
        h.leave();
        h.line("],");
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());

        if self.is_view {
            self.emit_attach_handler(&key, "if");
        }
    }

    pub(in crate::emit) fn emit_for(
        &mut self,
        w: &mut Writer,
        instr: &ForInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("forAnchor");

        // __mount-side: anchor + invoke hoisted handler.
        w.line(&format!(
            "const {anchor} = document.createComment(\"for\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}));"
        ));

        let it_rw = rewrite_for_scope(&instr.iterable.source, &[]);
        let var = &instr.var_name;
        let idx_name = instr.index_name.as_deref().unwrap_or("__i");

        let mut h = Writer::new();
        h.line(&format!(
            "const __wireBoundary_{key} = (anchor) => bindFor({{"
        ));
        h.enter();
        h.line("anchor,");
        h.line(&format!("items: () => ({it_rw}),"));
        if let Some(k) = &instr.key {
            let k_rw = rewrite_for_scope(&k.source, &[var.as_str(), idx_name]);
            h.line(&format!(
                "key: ({var}, {idx_name}) => {{ const __child = Object.create($scope); __child.{var} = {var}; __child.{idx_name} = {idx_name}; const $scope = __child; return ({k_rw}); }},"
            ));
        }
        h.line(&format!("render: ({var}, {idx_name}) => {{"));
        h.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        let child_scope = self.fresh("childScope");
        h.line(&format!("const {child_scope} = Object.create($scope);"));
        h.line(&format!("{child_scope}.{var} = {var};"));
        h.line(&format!("{child_scope}.{idx_name} = {idx_name};"));
        h.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        h.line(&format!("const {cl} = [];"));
        // Per-iteration scope shadowing — nested expressions resolve
        // `item` against the iteration's child scope.
        h.line(&format!("const $scope = {child_scope};"));
        // Walk body once. Body keys are allocated inside this walk and
        // contribute to the registry's single-iteration key range.
        self.boundary_depth += 1;
        self.emit_children(&mut h, &instr.body, &frag, &cl);
        self.boundary_depth -= 1;
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        h.leave();
        h.line("},");
        if let Some(eb) = &instr.else_body {
            h.line("emptyRender: () => {");
            h.enter();
            let frag = self.fresh("frag");
            let cl = self.fresh("c");
            h.line(&format!(
                "const {frag} = document.createDocumentFragment();"
            ));
            h.line(&format!("const {cl} = [];"));
            self.boundary_depth += 1;
            self.emit_children(&mut h, eb, &frag, &cl);
            self.boundary_depth -= 1;
            h.line(&format!(
                "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
            ));
            h.leave();
            h.line("},");
        }
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());

        if self.is_view {
            self.emit_attach_handler(&key, "for");
        }
    }

    pub(in crate::emit) fn emit_match(
        &mut self,
        w: &mut Writer,
        instr: &MatchInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let key = self.next_key();
        let anchor = self.fresh("matchAnchor");

        w.line(&format!(
            "const {anchor} = document.createComment(\"match\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(__wireBoundary_{key}({anchor}));"
        ));

        let disc_rw = rewrite_for_scope(&instr.discriminant.source, &[]);
        let mut h = Writer::new();
        h.line(&format!(
            "const __wireBoundary_{key} = (anchor) => bindIf({{"
        ));
        h.enter();
        h.line("anchor,");
        h.line("test: () => {");
        h.enter();
        h.line(&format!("const __d = ({disc_rw});"));
        for (i, c) in instr.cases.iter().enumerate() {
            let c_rw = rewrite_for_scope(&c.test.source, &[]);
            h.line(&format!(
                "if (__d === ({c_rw})) return {i};"
            ));
        }
        let fallback = if instr.default.is_some() {
            instr.cases.len() as i32
        } else {
            -1
        };
        h.line(&format!("return {fallback};"));
        h.leave();
        h.line("},");
        h.line("branches: [");
        h.enter();
        for c in &instr.cases {
            self.emit_branch(&mut h, &c.body);
        }
        if let Some(d) = &instr.default {
            self.emit_branch(&mut h, d);
        }
        h.leave();
        h.line("],");
        h.leave();
        h.line("});");
        self.boundary_hoists.push_str(&h.into_string());

        if self.is_view {
            self.emit_attach_handler(&key, "match");
        }
    }

    /// Emit a `__b<N>_attach: (target) => …` named property on the
    /// view's setup return. `attach()` looks this up by walking SSR'd
    /// `<sui-{kind} key="b<N>">` elements, then calls the handler with
    /// the boundary element. The handler swaps the wrapper for a
    /// Comment anchor and invokes the hoisted helper, whose first
    /// effect-run mounts the active branch synchronously.
    fn emit_attach_handler(&mut self, key: &str, kind: &str) {
        let mut a = Writer::new();
        a.line(&format!("__{key}_attach: (target) => {{"));
        a.enter();
        a.line("const __doc = target.ownerDocument ?? document;");
        a.line(&format!(
            "const __anchor = (target.nodeType === 1) ? __doc.createComment({}) : target;",
            js_string(kind)
        ));
        a.line(
            "if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);",
        );
        a.line(&format!("return __wireBoundary_{key}(__anchor);"));
        a.leave();
        a.line("},");
        self.attach_handlers.push_str(&a.into_string());
    }

    /// Emit `{#while-loading of <bind>}…{/while-loading}`.
    ///
    /// Hoists `const __renderWhileLoading_<N> = () => …` inside
    /// `setup(ctx)` so the body is walked exactly once. Both `__mount`
    /// (legacy) and `attach()` paths reference the same hoisted const.
    /// Single walk = single registry-key consumption per body.
    pub(super) fn emit_while_loading_block(
        &mut self,
        w: &mut Writer,
        instr: &WhileLoadingInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        // Registry parity: while-loading consumes one b<N> for the
        // boundary itself. Body keys are consumed during the (single)
        // emit walk below.
        let _ = self.next_key();
        let idx = self.while_loading_idx;
        self.while_loading_idx += 1;
        let render_key = format!("__renderWhileLoading_{idx}");
        let bind = &instr.bind;

        // Hoist the render fn so it's a single-source-of-truth.
        self.emit_wl_render_hoist(&render_key, &instr.body);

        // __mount inline call references the hoisted const.
        let anchor = self.fresh("wlAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-loading\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(bindWhileLoading({{ anchor: {anchor}, signal: $scope.{bind}, render: {render_key} }}));"
        ));

        // Re-export the hoisted const as a named property so
        // `attach()` can find the render fn through `binding.renderKey`.
        self.attach_handlers.push_str(&format!(
            "{}: {render_key},\n",
            js_string(&render_key)
        ));
    }

    /// Emit `{#while-error <errVar> of <bind>}…{/while-error}`.
    pub(super) fn emit_while_error_block(
        &mut self,
        w: &mut Writer,
        instr: &WhileErrorInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let _ = self.next_key();
        let idx = self.while_error_idx;
        self.while_error_idx += 1;
        let render_key = format!("__renderWhileError_{idx}");
        let bind = &instr.bind;
        let var = &instr.error_var;

        self.emit_we_render_hoist(&render_key, var, &instr.body);

        let anchor = self.fresh("weAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-error\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(bindWhileError({{ anchor: {anchor}, signal: $scope.{bind}, render: {render_key} }}));"
        ));

        self.attach_handlers.push_str(&format!(
            "{}: {render_key},\n",
            js_string(&render_key)
        ));
    }

    fn emit_wl_render_hoist(&mut self, render_key: &str, body: &[Instr]) {
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        let mut h = Writer::new();
        h.line(&format!("const {render_key} = () => {{"));
        h.enter();
        h.line(&format!("const {frag} = document.createDocumentFragment();"));
        h.line(&format!("const {cl} = [];"));
        self.boundary_depth += 1;
        self.emit_children(&mut h, body, &frag, &cl);
        self.boundary_depth -= 1;
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        h.leave();
        h.line("};");
        self.boundary_hoists.push_str(&h.into_string());
    }

    fn emit_we_render_hoist(&mut self, render_key: &str, error_var: &str, body: &[Instr]) {
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        let child = self.fresh("childScope");
        let var = error_var;
        let mut h = Writer::new();
        h.line(&format!("const {render_key} = ({var}) => {{"));
        h.enter();
        h.line(&format!("const {child} = Object.create($scope);"));
        h.line(&format!("{child}.{var} = {var};"));
        h.line(&format!("const {frag} = document.createDocumentFragment();"));
        h.line(&format!("const {cl} = [];"));
        h.line(&format!("const $scope = {child};"));
        self.boundary_depth += 1;
        self.emit_children(&mut h, body, &frag, &cl);
        self.boundary_depth -= 1;
        h.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        h.leave();
        h.line("};");
        self.boundary_hoists.push_str(&h.into_string());
    }
}
